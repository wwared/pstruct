use crate::types::*;

use fomat_macros::{fomat, wite};

use std::fmt;

impl fmt::Display for Type<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        wite!(
            f,
            match &self {
                Type::U8              => { "uint8" }
                Type::U16             => { "uint16" }
                Type::U32             => { "uint32" }
                Type::U64             => { "uint64" }
                Type::I8              => { "int8" }
                Type::I16             => { "int16" }
                Type::I32             => { "int32" }
                Type::I64             => { "int64" }
                Type::Byte            => { "byte" }
                Type::String          => { "string" }
                Type::User(user_type) => { (user_type) }
            }
        )
    }
}

impl fmt::Display for ArraySize<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        wite!(
            f,
            match &self {
                ArraySize::Constant(size)                   => { "[" (size) "]" }
                ArraySize::Unknown | ArraySize::Variable(_) => { "[]" }
                _                                           => { }
            }
        )
    }
}

fn some_kind_of_uppercase_first_letter(s: &str) -> String {
    let mut c = s.chars(); // TODO something better than this
    match c.next() {
        None    => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

#[allow(unreachable_code)]
fn render_encode_item(item: &Item, var_name: &str) -> String {
    let item_type = some_kind_of_uppercase_first_letter(
        fomat!(match &item.item_type {
            Type::Byte => { [Type::U8] }
            _          => { [item.item_type] }
        }).as_str(),
    );
    if let ArraySize::NotArray = item.array_size {
        fomat!(
            match &item.item_type {
                Type::User(_) => {
                    "\t" (var_name) "." (item.name) ".EncodeStream(stream)" "\n"
                }
                _ => {
                    "\t" "stream.Write" (item_type) "(" (var_name) "." (item.name) ")" "\n"
                }
            }
        )
    } else {
        fomat!(
            match &item.array_size { // FIXME by default sizes are u32s
                ArraySize::Unknown => { "\t" "stream.WriteU32(uint32(len(" (var_name) "." (item.name) ")))" "\n" }
                ArraySize::Variable(size_name) => { "\t" "stream.WriteU32(uint32(" (var_name) "." (size_name) "))" "\n" }
                _ => {}
            }
            "\t" "for idx := 0; idx < int("
                match &item.array_size {
                    ArraySize::Constant(array_size) => { (array_size) }
                    ArraySize::Unknown              => { "len("(var_name)"."(item.name)")" }
                    ArraySize::Variable(size_name)  => { (var_name)"."(size_name) }
                    _                               => { [unreachable!("unexpected NotArray")] }
                }
                "); idx++ {" "\n"
            match &item.item_type {
                Type::User(_) => {
                    "\t\t" (var_name) "." (item.name) "[idx].EncodeStream(stream)" "\n"
                }
                _ => {
                    "\t\t" "stream.Write" (item_type) "(" (var_name) "." (item.name) "[idx])" "\n"
                }
            }
            "\t" "}" "\n"
        )
    }
}

#[allow(unreachable_code)]
fn render_decode_item(item: &Item, var_name: &str) -> String {
    let item_type = some_kind_of_uppercase_first_letter(
        fomat!(match &item.item_type {
            Type::Byte => {[Type::U8]}
            _ => {[item.item_type]}
        })
        .as_str(),
    );
    if let ArraySize::NotArray = item.array_size {
        fomat!(
            match &item.item_type {
                Type::User(_) => {
                    "\t" "err = " (var_name) "." (item.name) ".DecodeStream(stream)" "\n"
                }
                _ => {
                    "\t" (var_name) "." (item.name) ", err = stream.Read" (item_type) "()" "\n"
                }
            }
            "\t" "if err != nil {" "\n"
            "\t\t" "return err" "\n"
            "\t" "}" "\n"
        )
    } else {
        fomat!(
            match &item.array_size {
                ArraySize::Unknown => { // FIXME by default sizes are u32s
                    "\t" (var_name) "_" (item.name) "_size, err := stream.ReadU32()" "\n"
                    "\t" "if err != nil {" "\n"
                    "\t\t" "return err" "\n"
                    "\t" "}" "\n"
                    "\t" (var_name) "." (item.name) " = make([]" (item.item_type) ", " (var_name) "_" (item.name) "_size)" "\n"
                }
                ArraySize::Variable(size_name) => { // FIXME by default sizes are u32s
                    "\t" (var_name) "_" (size_name) "_tmp, err := stream.ReadU32()" "\n"
                    "\t" "if err != nil {" "\n"
                    "\t\t" "return err" "\n"
                    "\t" "}" "\n"
                    "\t" (var_name) "." (size_name) " = " (item.item_type) "(" (var_name) "_" (size_name) "_tmp)" "\n"
                    "\t" (var_name) "." (item.name) " = make([]" (item.item_type) ", " (var_name) "." (size_name) ")" "\n"
                }
                _ => {}
            }
            "\t" "for idx := 0; idx < int("
                match &item.array_size {
                    ArraySize::Constant(array_size) => { (array_size) }
                    ArraySize::Unknown              => { (var_name) "_" (item.name) "_size" }
                    ArraySize::Variable(size_name)  => { (var_name) "." (size_name) }
                    _                               => {[unreachable!("unexpected NotArray")]}
                }
                "); idx++ {" "\n"
            match &item.item_type {
                Type::User(_) => {
                    "\t\terr = " (var_name) "." (item.name) "[idx].DecodeStream(stream)\n"
                }
                _ => {
                    "\t\t" (var_name) "." (item.name) "[idx], err = stream.Read" (item_type) "()\n"
                }
            }
            "\t\t" "if err != nil {" "\n"
            "\t\t\t" "return err" "\n"
            "\t\t" "}" "\n"
            "\t" "}" "\n"
        )
    }
}

impl fmt::Display for Definition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let var_name = self .name .get(0..1) .map(|x| x.to_lowercase()) .ok_or(fmt::Error)?;
        let item_type = &self.name;
        wite!(
            f,
            "type " (item_type) " struct {" "\n"
            for item in &self.items {
                "\t" (item.name) "\t" (item.array_size) (item.item_type) "\n"
            }
            "}" "\n\n"
            "func New" (item_type) "() " (item_type) " {" "\n"
            "\t" "return " (item_type) "{}" "\n"
            "}" "\n\n"
            "func (" (var_name) " *" (item_type) ") Encode() []byte {" "\n"
            "\t" "stream := ps.NewStream()" "\n"
            "\t" (var_name) ".EncodeStream(stream)" "\n"
            "\t" "return stream.GetData()" "\n"
            "}" "\n\n"
            "func (" (var_name) " *" (item_type) ") Decode(data []byte) error {" "\n"
            "\t" "return " (var_name) ".DecodeStream(ps.NewStreamReader(data))" "\n"
            "}" "\n\n"
            "func (" (var_name) " *" (item_type) ") EncodeStream(stream *ps.Stream) {" "\n"
            for item in &self.items {
                (render_encode_item(item, var_name.as_str()))
            }
            "}" "\n\n"
            "func (" (var_name) " *" (item_type) ") DecodeStream(stream *ps.Stream) error {" "\n"
            "\t" "var err error" "\n"
            for item in &self.items {
                (render_decode_item(item, var_name.as_str()))
            }
            "\t" "return nil" "\n"
            "}" "\n\n"
        )
    }
}

pub fn render_file(package_name: &str, file_definitions: &Vec<Definition>) -> String {
    fomat!(
        "package " (package_name.to_lowercase()) "\n\n"
        r#"import ps "github.com/wwared/pstruct/runtime""# "\n\n"
        for definition in file_definitions {
            (definition)
        }
    )
}
