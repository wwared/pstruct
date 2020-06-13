#[derive(Debug)]
pub enum Type {
    U8, U16, U32, U64,
    I8, I16, I32, I64,
    Byte,
    String,
    User(String),
}

#[derive(Debug)]
pub enum ArraySize {
    NotArray,
    Unknown,
    Constant(usize),
    Variable(String),
}

#[derive(Debug)]
pub struct Item {
    pub name: String,
    pub item_type: Type,
    pub array_size: ArraySize,
}

#[derive(Debug)]
pub struct Definition {
    pub name: String,
    pub items: Vec<Item>,
}
