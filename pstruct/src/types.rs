#[derive(Debug, PartialEq, Clone)]
pub enum Type<'a> {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
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
}

#[derive(Debug, Clone, Copy)]
pub enum Endian {
    Little,
    Big,
}

#[derive(Debug)]
pub struct Item<'a> {
    pub name: &'a str,
    pub kind: Type<'a>,
    pub array: Option<Array<'a>>,
    pub byte_order: Endian,
    pub type_alias: Option<&'a str>,
}

#[derive(Debug)]
pub struct Struct<'a> {
    pub name: &'a str,
    pub items: Vec<Item<'a>>,
}

#[derive(Debug)]
pub struct File<'a> {
    pub scope: String,
    pub raw_imports: Vec<&'a str>,
    pub structs: Vec<Struct<'a>>,
}
