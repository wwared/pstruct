#[macro_use]
extern crate pest_derive;

mod parser;
mod renderer;
mod types;

fn main() -> Result<(), parser::Error> {
    let tlv_zs = include_str!("../specs/simple.zs");
    let tlv_defs = parser::parse_file(tlv_zs)?;

    println!("{}", renderer::render_file("main", &tlv_defs));

    Ok(())
}
