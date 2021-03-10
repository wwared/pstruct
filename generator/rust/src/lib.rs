use std::{env, fs, io, io::Read, path};

use pstruct::parser;
use pstruct::types::*;

use proc_macro2::*;
use quote::quote;
use syn::{parse_macro_input, LitStr};

// copied from pest since i use the same strategy for getting the spec file path
// https://github.com/pest-parser/pest/blob/51fd1d49f1041f7839975664ef71fe15c7dcaf67/generator/src/generator.rs#L166-L178
// Needed because Cargo doesn't watch for changes in grammars.
fn generate_include(name: &Ident, path: &str) -> TokenStream {
    let const_name = Ident::new(&format!("_PSTRUCT_SPEC_{}", name), Span::call_site());
    // Need to make this relative to the current directory since the path to the file
    // is derived from the CARGO_MANIFEST_DIR environment variable
    let mut current_dir = std::env::current_dir().expect("Unable to get current directory");
    current_dir.push(path);
    let relative_path = current_dir.to_str().expect("path contains invalid unicode");
    quote! {
        #[allow(non_upper_case_globals)]
        const #const_name: &'static str = include_str!(#relative_path);
    }
}

fn read_file<P: AsRef<path::Path>>(path: P) -> io::Result<String> {
    let mut file = fs::File::open(path.as_ref())?;
    let mut string = String::new();
    file.read_to_string(&mut string)?;
    Ok(string)
}

// TODO: variant that takes in raw contents instead of filename

#[proc_macro]
pub fn pstruct(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as LitStr);

    let path = input.value();
    let root = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let path = path::Path::new(&root).join("src/").join(&path);

    // let include_specs = generate_include(&)

    let file_name = match path.file_name() {
        Some(file_name) => file_name,
        None => panic!("{} is not a file", path.to_string_lossy()),
    };
    let file_contents = match read_file(&path) {
        Ok(file_contents) => file_contents,
        Err(error) => panic!("error opening {:?}: {:?} {}", file_name, path, error),
    };

    let file = match parser::parse_file(file_contents.as_str()) {
        Ok(file) => file,
        Err(error) => {
            let error_message = format!("{}", error);
            return proc_macro::TokenStream::from(quote! {
                compile_error!(#error_message);
            });
        }
    };

    // println!("Scope: {}", file.scope); // XXX
    let scope = Ident::new(file.scope.as_str(), Span::call_site());
    let include_spec = generate_include(&scope, path.to_str().unwrap());

    let declarations = file.structs.iter().map(struct_declaration);

    // TODO: test with multiple pstruct! calls
    proc_macro::TokenStream::from(quote!(
        use pstruct_rs::Pstruct;
        mod #scope {
            #include_spec
            use pstruct_rs::*;
            use std::ffi::CString;
            #(#declarations)*
        }
    ))
}

fn struct_declaration(decl: &Struct) -> TokenStream {
    let struct_name = Ident::new(decl.name, Span::call_site());
    let fields = decl.items.iter().map(item_declaration);
    let defaults = decl.items.iter().map(item_default);
    let trait_impls = trait_impl(decl);
    quote!(
        #[derive(Debug)]
        pub struct #struct_name {
            #(#fields)*
        }

        impl Default for #struct_name {
            fn default() -> Self {
                Self {
                    #(#defaults)*
                }
            }
        }

        #trait_impls
    )
}

fn trait_impl(decl: &Struct) -> TokenStream {
    let struct_name = Ident::new(decl.name, Span::call_site());
    let size = size_fn(decl);
    let encode = encode_fn(decl);
    let decode = decode_fn(decl);
    quote!(
        impl Pstruct for #struct_name {
            #encode
            #decode
            #size
        }
    )
}

fn decode_item(item: &Item) -> TokenStream {
    let var_id = Ident::new(item.name, Span::call_site());
    let var = if item.array.is_none() {
        quote!(#var_id)
    } else {
        quote!(#var_id[idx])
    };
    let size = if item.array.is_none() {
        item_size(item)
    } else {
        type_size(&item.kind, &var)
    };
    let mut decode_fn = match item.byte_order {
        Endian::Little => {
            quote!(decode_le)
        }
        Endian::Big => {
            quote!(decode_be)
        }
    };
    if let Type::User(_) = item.kind {
        decode_fn = quote!(decode);
    }
    let single_item = match item.kind {
        Type::String => {
            quote!(
                let mut tmp_len: u16 = 0;
                tmp_len.#decode_fn(&data[..2])?;
                data = &data[2..];
                let mut tmp_buf: Vec<u8> = vec![0; tmp_len as usize];
                tmp_buf.#decode_fn(&data[..(tmp_len as usize)])?;
                self.#var = String::from_utf8(tmp_buf).unwrap();
                data = &data[(tmp_len as usize)..];
            )
        }
        Type::CString => {
            todo!("strings")
        }
        Type::User(_) => {
            // here we can't reliably use .size() before reading, since
            // the user type might have variable-sized elements that we haven't
            // filled in yet. for now, just pass in the rest of the buffer for
            // reading, then update the buffer by truncating based on what was read
            quote!(
                self.#var.#decode_fn(data)?;
                data = &data[#size..];
            )
        }
        _ => {
            quote!(
                let size = #size; // TODO: this is because #decode_fn borrows mutably, and #size might borrow immutably
                self.#var.#decode_fn(&data[..size])?;
                data = &data[size..];
            )
        }
    };
    if item.array.is_none() {
        single_item
    } else {
        let prefix_len = match &item.array {
            Some(Array::Constant(_)) => {
                quote!()
            }
            Some(Array::Variable(name, _)) => {
                let arr_len = Ident::new(name, Span::call_site());
                // NOTE: in this case, we already decoded the length previously, so
                // just initialize the vec
                // TODO: use .push and stuff instead of this gross stuff
                quote!(
                    self.#var_id = Vec::with_capacity(self.#arr_len as usize);
                    for idx in 0..(self.#arr_len as usize) {
                        self.#var_id.push(Default::default());
                    }
                )
            }
            Some(Array::Unknown(ty)) => {
                let arr_ty = quote_type(&ty);
                let arr_sz = type_size(&ty, &quote!(compile_error!("SHOULD NEVER HAPPEN")));
                // TODO: use .push and stuff instead of this gross stuff
                quote!(
                    let mut tmp_len: #arr_ty = 0;
                    tmp_len.#decode_fn(&data[..#arr_sz])?;
                    data = &data[#arr_sz..];
                    self.#var_id = Vec::with_capacity(tmp_len as usize);
                    for idx in 0..(tmp_len as usize) {
                        self.#var_id.push(Default::default());
                    }
                )
            }
            _ => {
                unreachable!()
            }
        };
        let array_size = match item.array {
            Some(Array::Constant(sz)) => {
                quote!(#sz)
            }
            Some(Array::Unknown(_)) => {
                quote!(tmp_len)
            }
            Some(Array::Variable(name, _)) => {
                let var = Ident::new(name, Span::call_site());
                quote!(self.#var)
            }
            _ => {
                unreachable!()
            }
        };
        quote!(
            #prefix_len
            for idx in 0..(#array_size as usize) {
                #single_item
            }
        )
    }
}

fn decode_fn(decl: &Struct) -> TokenStream {
    let items = decl.items.iter().map(|i| decode_item(i));
    quote!(
        fn decode_new(data: &[u8]) -> Result<Self> {
            let mut res = Self::default();
            res.decode(data)?;
            Ok(res)
        }
        fn decode(&mut self, data: &[u8]) -> Result<()> {
            assert!(data.len() >= self.size(), "todo improve errors");
            let mut data = data;
            #(#items)*
            Ok(())
        }
    )
}

fn encode_item(item: &Item) -> TokenStream {
    let var_id = Ident::new(item.name, Span::call_site());
    let var = if item.array.is_none() {
        quote!(#var_id)
    } else {
        quote!(#var_id[idx])
    };
    let size = if item.array.is_none() {
        item_size(item)
    } else {
        type_size(&item.kind, &var)
    };
    let mut encode_fn = match item.byte_order {
        Endian::Little => {
            quote!(encode_le)
        }
        Endian::Big => {
            quote!(encode_be)
        }
    };
    if let Type::User(_) = item.kind {
        encode_fn = quote!(encode_buf)
    }
    let single_item = match item.kind {
        Type::String => {
            // TODO implement for &[T] somehow so this doesn't need a clone?
            quote!(
                (self.#var.len() as u16).#encode_fn(&mut buf[..2])?;
                buf = &mut buf[2..];
                self.#var.as_bytes().to_vec().#encode_fn(&mut buf[..self.#var.len()])?;
                buf = &mut buf[self.#var.len()..];
            )
        }
        Type::CString => {
            todo!("strings")
        }
        _ => {
            quote!(
                self.#var.#encode_fn(&mut buf[..#size])?;
                buf = &mut buf[#size..];
            )
        }
    };
    if item.array.is_none() {
        single_item
    } else {
        let prefix_len = match &item.array {
            Some(Array::Constant(_)) => {
                quote!()
            }
            Some(Array::Variable(name, _)) => {
                let arr_len = Ident::new(name, Span::call_site());
                // NOTE: in this case, we already encoded the length previously, so
                // just assert the vec length matches
                quote!(
                    assert!(self.#var_id.len() == self.#arr_len as usize, "todo improve errors");
                )
            }
            Some(Array::Unknown(ty)) => {
                let arr_ty = quote_type(&ty);
                let arr_sz = type_size(&ty, &quote!(compile_error!("SHOULD NEVER HAPPEN")));
                quote!(
                    (self.#var_id.len() as #arr_ty).#encode_fn(&mut buf[..#arr_sz])?;
                    buf = &mut buf[#arr_sz..];
                )
            }
            _ => {
                unreachable!()
            }
        };
        quote!(
            #prefix_len
            for idx in 0..self.#var_id.len() {
                #single_item
            }
        )
    }
}

fn encode_fn(decl: &Struct) -> TokenStream {
    let items = decl.items.iter().map(|i| encode_item(i));
    quote!(
        fn encode(&self) -> Result<Vec<u8>> {
            let mut res = vec![0; self.size()];
            self.encode_buf(&mut res)?;
            Ok(res)
        }

        fn encode_buf(&self, buf: &mut [u8]) -> Result<()> {
            assert!(buf.len() >= self.size(), "todo improve errors");
            let mut buf = buf;
            #(#items)*
            Ok(())
        }
    )
}

fn type_size(ty: &Type, var: &TokenStream) -> TokenStream {
    match ty {
        Type::U8 | Type::Byte => {
            quote!(1)
        }
        Type::U16 => {
            quote!(2)
        }
        Type::U32 => {
            quote!(4)
        }
        Type::U64 => {
            quote!(8)
        }
        Type::I8 => {
            quote!(1)
        }
        Type::I16 => {
            quote!(2)
        }
        Type::I32 => {
            quote!(4)
        }
        Type::I64 => {
            quote!(8)
        }
        Type::F32 => {
            quote!(4)
        }
        Type::F64 => {
            quote!(8)
        }
        // These are all variable sized types
        // TODO: treat them differently so it's easier to tell when a type is what
        Type::String => {
            quote!(self.#var.len())
        }
        Type::CString => {
            // quote!(self.#var.as_bytes().len())
            // TODO: technically this is incorrect, the size is whatever is set
            // in the array field -- not sure if worth passing it in here
            // or just asserting that this never gets used
            quote!(compile_error!("OOPS LOL"))
        }
        Type::User(_) => {
            quote!(self.#var.size())
        }
    }
}

fn item_size(item: &Item) -> TokenStream {
    let var = Ident::new(item.name, Span::call_site());
    let var = quote!(#var);
    let mut size = type_size(&item.kind, &var);
    if item.kind == Type::String {
        // NOTE: this is always prefixed with u16 length, to keep implementation parity
        size = quote!(#size + 2);
    }
    // TODO: refactor this bit
    let prefix_len = match &item.array {
        Some(Array::Unknown(ty)) => {
            let arr_sz = type_size(&ty, &quote!(compile_error!("SHOULD NEVER HAPPEN")));
            quote!(#arr_sz + )
        }
        _ => {
            quote!()
        }
    };
    match &item.array {
        Some(arr) => {
            // this handles variable-sized elements; is there a better way?
            match item.kind {
                Type::User(_) => {
                    quote!((#prefix_len self.#var.iter().map(|i| i.size()).sum::<usize>()))
                }
                Type::CString => {
                    todo!("strings")
                }
                Type::String => {
                    quote!((#prefix_len self.#var.iter().map(|i| i.len() + 2).sum::<usize>()))
                }
                _ => {
                    // Constant-sized elements
                    match arr {
                        Array::Constant(sz) => {
                            quote!((#sz * (#size)))
                        }
                        Array::Variable(idx, _) => {
                            let idx = Ident::new(idx, Span::call_site());
                            quote!((self.#idx as usize * (#size)))
                        }
                        Array::Unknown(ty) => {
                            // TODO: validate if this is correct (i feel like it is)
                            let var = Ident::new(item.name, Span::call_site());
                            let arr_sz =
                                type_size(&ty, &quote!(compile_error!("SHOULD NEVER HAPPEN")));
                            quote!(self.#var.len() * (#size) + #arr_sz)
                        }
                    }
                }
            }
        }
        None => {
            quote!(#size)
        }
    }
}

fn size_fn(decl: &Struct) -> TokenStream {
    let sizes = decl.items.iter().map(item_size);
    quote!(
        fn size(&self) -> usize {
            #(#sizes)+*
        }
    )
}

fn item_default(item: &Item) -> TokenStream {
    let name = Ident::new(item.name, Span::call_site());
    let def = quote!(Default::default());
    let def = match item.array {
        Some(Array::Constant(sz)) => {
            // NOTE: this is gross, but i don't want to implement copy/clone
            let mut inner = quote!(#def);
            for _ in 1..sz {
                inner = quote!(#inner, #def)
            }
            quote!([#inner])
        }
        Some(_) => {
            quote!(vec![])
        }
        None => def,
    };
    quote!(#name: #def,)
}

fn quote_type(ty: &Type) -> TokenStream {
    match ty {
        Type::U8 | Type::Byte => {
            quote!(u8)
        }
        Type::U16 => {
            quote!(u16)
        }
        Type::U32 => {
            quote!(u32)
        }
        Type::U64 => {
            quote!(u64)
        }
        Type::I8 => {
            quote!(i8)
        }
        Type::I16 => {
            quote!(i16)
        }
        Type::I32 => {
            quote!(i32)
        }
        Type::I64 => {
            quote!(i64)
        }
        Type::F32 => {
            quote!(f32)
        }
        Type::F64 => {
            quote!(f64)
        }
        Type::String => {
            quote!(String)
        }
        Type::CString => {
            quote!(CString)
        }
        Type::User(user_ty) => {
            let user_ty = Ident::new(user_ty, Span::call_site());
            quote!(#user_ty)
        }
    }
}

fn item_declaration(item: &Item) -> TokenStream {
    let name = Ident::new(item.name, Span::call_site());
    let ty = quote_type(&item.kind);
    let ty = match item.array {
        Some(Array::Constant(sz)) => {
            quote!([#ty; #sz])
        }
        Some(_) => {
            quote!(Vec<#ty>)
        }
        None => ty,
    };
    quote!(pub #name: #ty,)
}
