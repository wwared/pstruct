use crate::types::*;

use fomat_macros::{fomat, wite};

use std::fmt;

impl fmt::Display for Type<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        wite!(
            f,
            match &self {
                Type::U8 => {"uint8"}
                Type::U16 => {"uint16"}
                Type::U32 => {"uint32"}
                Type::U64 => {"uint64"}
                Type::I8 => {"int8"}
                Type::I16 => {"int16"}
                Type::I32 => {"int32"}
                Type::I64 => {"int64"}
                Type::Byte => {"byte"}
                Type::String => {"string"}
                Type::User(user_type) => {(user_type)}
            }
        )
    }
}

impl fmt::Display for ArraySize<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        wite!(
            f,
            match &self {
                ArraySize::Constant(size) => {"[" (size) "]"}
                ArraySize::Unknown | ArraySize::Variable(_) => {"[]"}
                _ => {}
            }
        )
    }
}

fn some_kind_of_uppercase_first_letter(s: &str) -> String {
    let mut c = s.chars(); // TODO something better than this
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn render_encode_item(item: &Item, var_name: &str) -> String {
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
                    "\t" (var_name) "." (item.name) ".EncodeStream(stream)\n"
                }
                _ => {
                    "\tstream.Write" (item_type) "(" (var_name) "." (item.name) ")\n"
                }
            }
        )
    } else {
        fomat!()
    }
}

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
                    "\terr = " (var_name) "." (item.name) ".DecodeStream(stream)\n"
                }
                _ => {
                    "\t" (var_name) "." (item.name) ", err = stream.Read" (item_type) "()\n"
                }
            }
            "\tif err != nil {\n"
                "\t\treturn err\n"
            "\t}\n"
        )
    } else {
        fomat!()
    }
}

impl fmt::Display for Definition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let var_name = self .name .get(0..1) .map(|x| x.to_lowercase()) .ok_or(fmt::Error)?;
        let item_type = &self.name;
        wite!(
            f,
            "type " (item_type) " struct {\n"
                for item in &self.items {
                    "\t" (item.name) "\t" (item.array_size) (item.item_type) "\n"
                }
            "}\n\n"
            "func New" (item_type) "() " (item_type) " {\n"
                "\treturn " (item_type) "{}\n"
            "}\n\n"
            "func (" (var_name) " *" (item_type) ") Encode() []byte {\n"
                "\tstream := ps.NewStream()\n"
                "\t" (var_name) ".EncodeStream(stream)\n"
                "\treturn stream.GetData()\n"
            "}\n\n"
            "func (" (var_name) " *" (item_type) ") Decode(data []byte) error {\n"
                "\treturn " (var_name) ".DecodeStream(ps.NewStreamReader(data))\n"
            "}\n\n"
            "func (" (var_name) " *" (item_type) ") EncodeStream(stream *ps.Stream) {\n"
                for item in &self.items {
                    (render_encode_item(item, var_name.as_str()))
                }
            "}\n\n"
            "func (" (var_name) " *" (item_type) ") DecodeStream(stream *ps.Stream) error {\n"
                "\tvar err error\n"
                for item in &self.items {
                    (render_decode_item(item, var_name.as_str()))
                }
                "\treturn nil\n"
            "}\n\n"
        )
    }
}

pub fn render_file(package_name: &str, file_definitions: &Vec<Definition>) -> String {
    fomat!(
        "package " (package_name.to_lowercase())
        "\n\nimport ps \"github.com/wwared/pstruct\"\n\n"
        for definition in file_definitions {
            (definition)
        }
    )
}
