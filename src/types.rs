#[derive(Debug)]
pub enum Type<'a> {
    U8, U16, U32, U64,
    I8, I16, I32, I64,
    Byte, String, User(&'a str),
}

#[derive(Debug)]
pub enum ArraySize<'a> {
    Unknown,
    Constant(usize),
    Variable(&'a str),
}

#[derive(Debug)]
pub struct Item<'a> {
    pub name: &'a str,
    pub item_type: Type<'a>,
    pub array_size: Option<ArraySize<'a>>,
}

#[derive(Debug)]
pub struct Definition<'a> {
    pub name: &'a str,
    pub items: Vec<Item<'a>>,
}
