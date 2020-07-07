#[macro_use]
extern crate pest_derive;

mod parser;
mod renderer;
mod types;

fn main() -> Result<(), parser::Error> {
    let file_contents = include_str!("../specs/simple.zs");
    let file = parser::parse_file(file_contents)?;

    println!("{}", renderer::render_file(&file));

    Ok(())
}
