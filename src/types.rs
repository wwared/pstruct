#[derive(PartialEq, Clone, Debug)]
pub enum Type<'a> {
    U8, U16, U32, U64,
    I8, I16, I32, I64,
    Byte,
    String,
    CString,
    User(&'a str),
}

#[derive(Debug)]
pub enum Array<'a> {
    Constant(usize),
    Unknown(Type<'a>),
    Variable(&'a str, Type<'a>),
    Bounded(usize, Type<'a>), // TODO syntax and parser code
}

#[derive(Debug)]
pub enum Endian {
    Little,
    Big,
}

#[derive(Debug)]
pub struct Item<'a> {
    pub name: &'a str,
    pub kind: Type<'a>,
    pub array: Option<Array<'a>>,
    pub byte_order: Endian, // TODO syntax and parser code
}

#[derive(Debug)]
pub struct Struct<'a> {
    pub name: &'a str,
    pub items: Vec<Item<'a>>,
}

#[derive(Debug)]
pub struct File<'a> {
    pub name: &'a str,
    pub structs: Vec<Struct<'a>>,
}
