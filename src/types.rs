extern crate serde;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Type {
    U8, U16, U32, U64,
    I8, I16, I32, I64,
    Byte, String, User(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ArraySize {
    Unknown,
    Constant(usize),
    Variable(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Item {
    pub name: String,
    pub item_type: Type,
    pub array_size: Option<ArraySize>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Definition {
    pub name: String,
    pub items: Vec<Item>,
}
