#[macro_use]
extern crate pest_derive;

mod parser;
mod types;

use tera::{Context, Tera, Value};
use std::collections::HashMap;

struct TeraArray;
impl tera::Filter for TeraArray {
    fn filter(&self, value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
        use types::ArraySize;
        // println!("value {:?} {} args {:?}", value, value, args);
        let opt_array: Option<ArraySize> = serde_json::from_value(value.clone())?;
        Ok(match opt_array {
            None => Value::Null,
            Some(sz) => {
                Value::String(match sz {
                    ArraySize::Constant(size) => format!("[{}]", size),
                    ArraySize::Unknown | ArraySize::Variable(_) => "[]".to_string(),
                })
            }
        })
    }
}

struct TeraType;
impl tera::Filter for TeraType {
    fn filter(&self, value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
        use types::Type;
        // println!("value {:?} {} args {:?}", value, value, args);
        let item_type: Type = serde_json::from_value(value.clone())?;
        Ok(Value::String(match item_type {
            Type::U8 => "uint8".to_string(),
            Type::U16 => "uint16".to_string(),
            Type::U32 => "uint32".to_string(),
            Type::U64 => "uint64".to_string(),
            Type::I8 => "int8".to_string(),
            Type::I16 => "int16".to_string(),
            Type::I32 => "int32".to_string(),
            Type::I64 => "int64".to_string(),
            Type::Byte => "byte".to_string(),
            Type::String => "string".to_string(),
            Type::User(user_type) => user_type,
        }))
    }
}

struct TeraVarname;
impl tera::Filter for TeraVarname {
    fn filter(&self, value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
        let mut str_value: String = serde_json::from_value(value.clone())?;
        str_value.truncate(1);
        str_value.make_ascii_lowercase();
        Ok(Value::String(str_value))
    }
}

fn main() -> Result<(), parser::Error> {
    let tlv_zs = include_str!("../specs/simple.zs");
    let tlv_defs = parser::parse_file(tlv_zs)?;

    let mut tera = Tera::new("template/go/**/*").expect("wtf");
    tera.register_filter("array", TeraArray);
    tera.register_filter("type", TeraType);
    tera.register_filter("varname", TeraVarname);
    let mut ctx = Context::new();
    ctx.insert("struct_definitions", &tlv_defs);
    ctx.insert("package_name", "main");
    println!("{}", tera.render("file.tera", &ctx).unwrap());

    Ok(())
}
