use std::collections::BTreeSet;
use pest::Parser;
use pest::error::ErrorVariant;
use pest::iterators::Pair;

use crate::types::*;

pub type Error = pest::error::Error<Rule>;

#[derive(Parser)]
#[grammar = "struct.pest"]
struct StructParser;

// unwraps look spooky but the grammar says it's fine
fn parse_definition(pair: Pair<Rule>) -> Struct {
    assert!(pair.as_rule() == Rule::definition, "expected definition");
    eprintln!("Parsing definition {}", pair.as_str());
    let mut inner_rules = pair.into_inner();
    eprintln!("Inner rules {:?}", inner_rules);
    // struct_name -> identifier -> as_str
    let name = inner_rules.next().unwrap().into_inner().next().unwrap().as_str();
    eprintln!("Name {:?}", name);
    let mut items: Vec<Item> = vec![];
    // all other rules are for items
    for item_pair in inner_rules {
        items.push(parse_item(item_pair));
    }
    Struct {name, items}
}

fn parse_item_type(type_name: &str) -> Type {
    match type_name {
        "u8" => Type::U8, "u16" => Type::U16, "u32" => Type::U32, "u64" => Type::U64,
        "i8" => Type::I8, "i16" => Type::I16, "i32" => Type::I32, "i64" => Type::I64,
        "byte" => Type::Byte, "string" => Type::String,
        _ => Type::User(type_name)
    }
}

fn parse_item(pair: Pair<Rule>) -> Item {
    assert!(pair.as_rule() == Rule::struct_item, "expected struct item");
    eprintln!("Parsing item {}", pair.as_str());
    let mut inner_rules = pair.into_inner();
    let name = inner_rules.next().unwrap().as_str();
    eprintln!("Item name {:?}", name);
    let type_pair = inner_rules.next().unwrap();
    assert!(type_pair.as_rule() == Rule::type_decl, "expected type declaration");
    let array: Option<Array>;
    let item_type: Type;
    let mut type_inner = type_pair.into_inner();
    let first_elem = type_inner.next().unwrap();
    match first_elem.as_rule() {
        Rule::array_brackets => {
            if let Some(arr_pair) = first_elem.into_inner().next() {
                let arr_str = arr_pair.as_str();
                array = match arr_str.parse::<usize>() {
                    Ok(size) => Some(Array::Constant(size)),
                    Err(_) => Some(Array::Variable(arr_str, Type::I32)), // FIXME type here
                };
            } else {
                array = Some(Array::Unknown(Type::I32)); // FIXME
            }
            eprintln!("{:?}", array);
            item_type = parse_item_type(type_inner.next().unwrap().as_str());
        },
        Rule::identifier => {
            array = None;
            item_type = parse_item_type(first_elem.as_str());
        },
        _ => unreachable!("expected array or identifier")
    };
    Item { name, kind: item_type, array, byte_order: Endian::Little, }
}


pub fn parse_file(file_contents: &str) -> Result<File, Error> {
    let parse_res = StructParser::parse(Rule::file, file_contents)?;

    let mut definitions = vec![];
    let mut defined_structs = BTreeSet::new();
    let mut defined_vars = BTreeSet::new();

    for def_pair in parse_res {
        if def_pair.as_rule() == Rule::EOI { break; }
        eprintln!("---------");
        let def = parse_definition(def_pair);
        eprintln!("{:#?}", def);
        for item in &def.items {
            defined_vars.insert(item.name);
        }
        defined_structs.insert(def.name);
        definitions.push(def);
    }
    for def in &definitions {
        for item in &def.items {
            // check for undefined types
            if let Type::User(typ) = &item.kind {
                if !defined_structs.contains(typ) {
                    let message = format!("{}: Undefined type '{}'", def.name, typ);
                    let error_span = pest::Span::new(&typ, 0, typ.len()).unwrap();
                    return Err(Error::new_from_span(ErrorVariant::CustomError{message}, error_span));
                }
            }
            // check for undefined variables
            if let Some(Array::Variable(var, _)) = &item.array {
                if !defined_vars.contains(var) {
                    let message = format!("{}: Undefined variable '{}'", def.name, var);
                    let error_span = pest::Span::new(var, 0, var.len()).unwrap();
                    return Err(Error::new_from_span(ErrorVariant::CustomError{message}, error_span));
                }
            }
        }
    }
    let names = defined_structs.iter().cloned().collect::<Vec<&str>>().join(", ");
    eprintln!("Got a total of {} definitions: {}", defined_structs.len(), names);

    // TODO package name
    Ok(File { name: "main", structs: definitions })
}

#[cfg(test)]
#[test]
fn parser_tests() {
    // TODO: check for an easy way to remove the repetition
    let test = include_str!("../specs/tlv.zs");
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_ok(), "tlv.zs");
    let res = parse_file(test);
    assert!(res.is_ok(), "tlv.zs");

    let test = include_str!("../specs/ak.zs");
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_ok(), "ak.zs");
    let res = parse_file(test);
    assert!(res.is_ok(), "ak.zs");

    let test = "
struct player {
    hp u8
    sp i16
}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_ok(), "simplest test");
    let res = parse_file(test);
    assert!(res.is_ok(), "simplest test");

    let test = "
struct player {
    hp [10]u8
    sp []i16
}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_ok(), "array test");
    let res = parse_file(test);
    assert!(res.is_ok(), "array test");

    let test = "
struct player {
    hp u8[10]
    sp []i16
}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_err(), "wrong position of brackets");
    let res = parse_file(test);
    assert!(res.is_err(), "wrong position of brackets");

    let test = "
struct player {
    hp [5]byte
    sp []i16[]
}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_err(), "wrong position of empty brackets");
    let res = parse_file(test);
    assert!(res.is_err(), "wrong position of empty brackets");

    let test = "
structplayer{
    hp u8
    sp i16
}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_err(), "no spaces in name");
    let res = parse_file(test);
    assert!(res.is_err(), "no spaces in name");

    let test = "
struct player {
    hp u8sp i16
}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_err(), "no space between items");
    let res = parse_file(test);
    assert!(res.is_err(), "no space between items");

    let test = "
struct player {
    hpu8 spi16
}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_ok(), "no space between type and name - becomes single item, valid grammar");
    let res = parse_file(test);
    assert!(res.is_err(), "no space between type and name - becomes single item, invalid type");

    let test = "
struct player {
    hp u8
    sp i16
}
struct ship {
    ang [3]
    u8spd string
}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_err(), "no space between items on second definition");
    let res = parse_file(test);
    assert!(res.is_err(), "no space between items on second definition");

    let test = "
struct player {
    hp []u8
    sp [hp]i16
}
struct ship {
    ang [3]u8
    spd string
}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_ok(), "variable as array size");
    let res = parse_file(test);
    assert!(res.is_ok(), "variable as array size");

    let test = "
struct player {
    hp []u8
    sp [asdf]i16
}

struct ship {
    ang [3]u8
    spd string
}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_ok(), "invalid variable as array size");
    let res = parse_file(test);
    assert!(res.is_err(), "invalid variable as array size");

    let test = "
struct player {
    hp []u8
    sp []adf
}

struct ship {
    ang [3]u8
    spd string
}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_ok(), "invalid type");
    let res = parse_file(test);
    assert!(res.is_err(), "invalid type");

    let test = "struct player {  hp []u8   sp []u16  }";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_err(), "needs line endings");
    let res = parse_file(test);
    assert!(res.is_err(), "needs line endings");

    let test = "struct player {  hp []u8
sp []u16  }";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_err(), "needs line endings on both ends");
    let res = parse_file(test);
    assert!(res.is_err(), "needs line endings on both ends");

    let test = "struct player {  hp []u8
sp []u16
}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_err(), "needs line endings at beginning of struct");
    let res = parse_file(test);
    assert!(res.is_err(), "needs line endings at beginning of struct");

    let test = "struct player {
hp []u8
sp []u16 }";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_err(), "needs line endings at end of struct");
    let res = parse_file(test);
    assert!(res.is_err(), "needs line endings at end of struct");


    let test = "struct player {
    hp
 []u8
    sp
[]u16
}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_err(), "no line endings in middle of items");
    let res = parse_file(test);
    assert!(res.is_err(), "no line endings in middle of items");

    let test = "
struct
player {
    hp []u8
    sp []u16
}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_err(), "no line ending in struct name");
    let res = parse_file(test);
    assert!(res.is_err(), "no line ending in struct name");

    let test = "
struct   player
{
    hp []u8
    sp []u16
}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_ok(), "line ending after name is ok, also spaces between struct and name");
    let res = parse_file(test);
    assert!(res.is_ok(), "line ending after name is ok, also spaces between struct and name");

    let test = "
/* hey look */ struct player // comments
{  // work fine
    hp []u8 /* real cool! */
/* multi
line
*/ sp []u16 // all the way to the end of the line
/* wow */} // amazing
";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_ok(), "comments, wow");
    let res = parse_file(test);
    assert!(res.is_ok(), "comments, wow");

    let test = "
struct /* can't comment everywhere though */ player
{
    hp /* here doesn't work either */ []u8
    sp []u16
}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_err(), "no comments between struct and name");
    let res = parse_file(test);
    assert!(res.is_err(), "no comments between struct and name");

    let test = "
struct player
{
    hp []u8 /* multi line comments
can eat your newlines */  sp []u16
}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_err(), "newline removed by multiline comment");
    let res = parse_file(test);
    assert!(res.is_err(), "newline removed by multiline comment");
}
