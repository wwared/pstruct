#[macro_use]
extern crate pest_derive;

mod parser;
mod renderer;
mod types;

use std::{env, fs};

fn main() -> Result<(), parser::Error> {
    let file_name = env::args().nth(1).expect("give file name");
    let file_contents = fs::read_to_string(file_name).expect("error reading file");
    let file = parser::parse_file(file_contents.as_str())?;

    println!("{}", renderer::render_file(&file));

    Ok(())
}
