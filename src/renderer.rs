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
                Type::CString         => { "string" }
                Type::User(user_type) => { (user_type) }
            }
        )
    }
}

impl fmt::Display for Array<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        wite!(
            f,
            match &self {
                Array::Constant(size)                    => { "[" (size) "]" }
                Array::Unknown(_) | Array::Variable(_,_) => { "[]" }
                _                                        => { } // TODO bounded case
            }
        )
    }
}

impl Type<'_> {
    fn alt(&self) -> String {
        fn some_kind_of_uppercase_first_letter(s: &str) -> String {
            let mut c = s.chars(); // TODO something better than this
            match c.next() {
                None    => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        }

        some_kind_of_uppercase_first_letter(
            fomat!(match &self {
                Type::Byte => { [Type::U8] }
                _          => { [self] }
            }).as_str(),
        )
    }
}

fn render_encode_item(item: &Item, var_name: &str) -> String {
    let item_kind = &item.kind.alt();
    if let Some(arr) = &item.array {
        fomat!(
            if item.kind != Type::CString {
                match &arr {
                    Array::Unknown(kind) => {
                        "\t" "stream.Write" (kind.alt()) "(" (kind) "(len(" (var_name) "." (item.name) ")))" "\n"
                    }
                    Array::Variable(size_name, kind) => {
                        "\t" "stream.Write" (kind.alt()) "(" (kind) "(" (var_name) "." (size_name) "))" "\n"
                    }
                    _ => {}
                }
                "\t" "for idx := 0; idx < int("
                    match &arr {
                        Array::Constant(size)         => { (size) }
                        Array::Unknown(_)             => { "len("(var_name)"."(item.name)")" }
                        Array::Variable(size_name, _) => { (var_name)"."(size_name) }
                        Array::Bounded(_, _)          => { "TODO bounded case" }
                    }
                    "); idx++ {" "\n"
            }
            match &item.kind {
                Type::User(_) => {
                    "\t\t" (var_name) "." (item.name) "[idx].EncodeStream(stream)" "\n"
                }
                Type::CString => {
                    match &arr {
                        Array::Constant(size) => {
                            "\t" "stream.WriteCString(" (var_name) "." (item.name) ", " (size) ")" "\n"
                        }
                        _ => { /*unimplemented!("cstrings with non-constant sizes not supported")*/ }
                    }
                }
                _ => {
                    "\t\t" "stream.Write" (item_kind) "(" (var_name) "." (item.name) "[idx])" "\n"
                }
            }
            if item.kind != Type::CString {
                "\t" "}" "\n"
            }
        )
    } else {
        fomat!(
            match &item.kind {
                Type::User(_) => {
                    "\t" (var_name) "." (item.name) ".EncodeStream(stream)" "\n"
                }
                _ => {
                    "\t" "stream.Write" (item_kind) "(" (var_name) "." (item.name) ")" "\n"
                }
            }
        )
    }
}

fn render_decode_item(item: &Item, var_name: &str) -> String {
    let item_kind = &item.kind.alt();
    if let Some(arr) = &item.array {
        fomat!(
            if item.kind != Type::CString {
                match &arr {
                    Array::Unknown(kind) => {
                        "\t" (var_name) "_" (item.name) "_size, err := stream.Read" (kind.alt()) "()" "\n"
                        "\t" "if err != nil {" "\n"
                        "\t\t" "return err" "\n"
                        "\t" "}" "\n"
                        "\t" (var_name) "." (item.name) " = make([]" (item.kind) ", " (var_name) "_" (item.name) "_size)" "\n"
                    }
                    Array::Variable(size_name, kind) => {
                        "\t" (var_name) "_" (size_name) "_tmp, err := stream.Read" (kind.alt()) "()" "\n"
                        "\t" "if err != nil {" "\n"
                        "\t\t" "return err" "\n"
                        "\t" "}" "\n"
                        "\t" (var_name) "." (size_name) " = " (item.kind) "(" (var_name) "_" (size_name) "_tmp)" "\n"
                        "\t" (var_name) "." (item.name) " = make([]" (item.kind) ", " (var_name) "." (size_name) ")" "\n"
                    }
                    _ => {}
                }
                "\t" "for idx := 0; idx < int("
                    match &arr {
                        Array::Constant(size)        => { (size) }
                        Array::Unknown(_)            => { (var_name) "_" (item.name) "_size" }
                        Array::Variable(size_name,_) => { (var_name) "." (size_name) }
                        Array::Bounded(_, _)         => { "TODO Bounded" }
                    }
                    "); idx++ {" "\n"
            }
            match &item.kind {
                Type::User(_) => {
                    "\t\terr = " (var_name) "." (item.name) "[idx].DecodeStream(stream)\n"
                }
                Type::CString => {
                    match &arr {
                        Array::Constant(size) => {
                            "\t" (var_name) "." (item.name) ", err = stream.ReadCString(" (size) ")" "\n"
                        }
                        _ => { /*unimplemented!("cstrings with non-constant sizes not supported")*/ }
                    }
                }
                _ => {
                    "\t\t" (var_name) "." (item.name) "[idx], err = stream.Read" (item_kind) "()\n"
                }
            }
            "\t\t" "if err != nil {" "\n"
            "\t\t\t" "return err" "\n"
            "\t\t" "}" "\n"
            if item.kind != Type::CString {
                "\t" "}" "\n"
            }
        )
    } else {
        fomat!(
            match &item.kind {
                Type::User(_) => {
                    "\t" "err = " (var_name) "." (item.name) ".DecodeStream(stream)" "\n"
                }
                _ => {
                    "\t" (var_name) "." (item.name) ", err = stream.Read" (item_kind) "()" "\n"
                }
            }
            "\t" "if err != nil {" "\n"
            "\t\t" "return err" "\n"
            "\t" "}" "\n"
        )
    }
}

impl fmt::Display for Struct<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let var_name = self.name.get(0..1) .map(|x| x.to_lowercase()) .ok_or(fmt::Error)?;
        wite!(
            f,
            "type " (self.name) " struct {" "\n"
            for item in &self.items {
                match &item.array {
                    Some(arr) => {
                        if item.kind == Type::CString {
                            "\t" (item.name) "\t" (item.kind) "\n"
                        } else {
                            "\t" (item.name) "\t" (arr) (item.kind) "\n"
                        }
                    }
                    None => { "\t" (item.name) "\t" (item.kind) "\n" }
                } // TODO do something better?
            }
            "}" "\n\n"
            "func New" (self.name) "() " (self.name) " {" "\n"
            "\t" "return " (self.name) "{}" "\n"
            "}" "\n\n"
            "func (" (var_name) " *" (self.name) ") Encode() []byte {" "\n"
            "\t" "stream := ps.NewStream()" "\n"
            "\t" (var_name) ".EncodeStream(stream)" "\n"
            "\t" "return stream.GetData()" "\n"
            "}" "\n\n"
            "func (" (var_name) " *" (self.name) ") Decode(data []byte) error {" "\n"
            "\t" "return " (var_name) ".DecodeStream(ps.NewStreamReader(data))" "\n"
            "}" "\n\n"
            "func (" (var_name) " *" (self.name) ") EncodeStream(stream *ps.Stream) {" "\n"
            for item in &self.items {
                (render_encode_item(item, var_name.as_str()))
            }
            "}" "\n\n"
            "func (" (var_name) " *" (self.name) ") DecodeStream(stream *ps.Stream) error {" "\n"
            "\t" "var err error" "\n"
            for item in &self.items {
                (render_decode_item(item, var_name.as_str()))
            }
            "\t" "return nil" "\n"
            "}" "\n\n"
        )
    }
}

pub fn render_file(file: &File) -> String {
    fomat!(
        "package " (file.name.to_lowercase()) "\n\n"
        r#"import ps "github.com/wwared/pstruct/runtime""# "\n\n"
        for definition in &file.structs {
            (definition)
        }
    )
}
