#[derive(PartialEq, Clone, Debug)]
pub enum Type<'a> {
    U8, U16, U32, U64,
    I8, I16, I32, I64,
    F32, F64,
    Byte,
    String,
    CString,
    User(&'a str),
}

pub enum Array<'a> {
    Constant(usize),
    Unknown(Type<'a>),
    Variable(&'a str, Type<'a>),
}

#[derive(Clone, Copy)]
pub enum Endian {
    Little,
    Big,
}

pub struct Item<'a> {
    pub name: &'a str,
    pub kind: Type<'a>,
    pub array: Option<Array<'a>>,
    pub byte_order: Endian,
}

pub struct Struct<'a> {
    pub name: &'a str,
    pub items: Vec<Item<'a>>,
}

pub struct File<'a> {
    pub scope: String,
    pub structs: Vec<Struct<'a>>,
}
