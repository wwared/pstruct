#[macro_use]
extern crate pest_derive;

mod parser;
mod types;

use tera::{Context, Tera};

fn main() -> Result<(), parser::Error> {
    let tlv_test = include_str!("../specs/tlv.zs");
    let _res = parser::parse_file(tlv_test)?;

    let _tera = Tera::new("template/go/**/*").expect("wtf");
    let mut ctx = Context::new();
    ctx.insert("package_name", "channel");
    let structs = vec!["test", "channel"];
    ctx.insert("struct_definitions", &structs);

    // println!("{}", tera.render("struct.tera", &ctx).unwrap());

    Ok(())
}
