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
fn parse_definition(pair: Pair<Rule>) -> Definition {
    assert!(pair.as_rule() == Rule::definition, "expected definition");
    // println!("Parsing definition {}", pair.as_str());
    let mut inner_rules = pair.into_inner();
    // println!("Inner rules {:?}", inner_rules);
    // struct_name -> identifier -> as_str
    let name = inner_rules.next().unwrap().into_inner().next().unwrap().as_str();
    // println!("Name {:?}", name);
    let mut items: Vec<Item> = vec![];
    // all other rules are for items
    for item_pair in inner_rules {
        items.push(parse_item(item_pair));
    }
    Definition {name: name.to_string(), items}
}

fn parse_item_type(type_name: &str) -> Type {
    match type_name {
        "u8" => Type::U8, "u16" => Type::U16, "u32" => Type::U32, "u64" => Type::U64,
        "i8" => Type::I8, "i16" => Type::I16, "i32" => Type::I32, "i64" => Type::I64,
        "byte" => Type::Byte, "string" => Type::String,
        _ => Type::User(type_name.to_string())
    }
}

fn parse_item(pair: Pair<Rule>) -> Item {
    assert!(pair.as_rule() == Rule::struct_item, "expected struct item");
    // println!("Parsing item {}", pair.as_str());
    let mut inner_rules = pair.into_inner();
    let name = inner_rules.next().unwrap().as_str();
    // println!("Item name {:?}", name);
    let type_pair = inner_rules.next().unwrap();
    assert!(type_pair.as_rule() == Rule::type_decl, "expected type declaration");
    let array_size: Option<ArraySize>;
    let item_type: Type;
    let mut type_inner = type_pair.into_inner();
    let first_elem = type_inner.next().unwrap();
    match first_elem.as_rule() {
        Rule::array_brackets => {
            if let Some(array) = first_elem.into_inner().next() {
                let arr_str = array.as_str();
                array_size = Some(match arr_str.parse::<usize>() {
                    Ok(size) => ArraySize::Constant(size),
                    Err(_) => ArraySize::Variable(arr_str.to_string()),
                });
            } else {
                array_size = Some(ArraySize::Unknown);
            }
            // println!("Array Size {:?}", array_size);
            item_type = parse_item_type(type_inner.next().unwrap().as_str());
        },
        Rule::identifier => {
            array_size = None;
            item_type = parse_item_type(first_elem.as_str());
        },
        _ => unreachable!("expected array or identifier")
    };
    Item { name: name.to_string(), item_type, array_size }
}


pub fn parse_file(file_contents: &str) -> Result<Vec<Definition>, Error> {
    let parse_res = StructParser::parse(Rule::file, file_contents)?;

    let mut definitions = vec![];
    let mut defined_structs = BTreeSet::new();
    let mut defined_vars = BTreeSet::new();

    for def_pair in parse_res {
        if def_pair.as_rule() == Rule::EOI { break; }
        // println!("---------");
        let def = parse_definition(def_pair);
        // println!("{:#?}", def);
        for item in &def.items {
            defined_vars.insert(item.name.clone());
        }
        defined_structs.insert(def.name.clone());
        definitions.push(def);
    }
    for def in &definitions {
        for item in &def.items {
            // check for undefined types
            match &item.item_type {
                Type::User(typ) => {
                    if !defined_structs.contains(typ) {
                        let message = format!("{}: Undefined type '{}'", def.name, typ);
                        let error_span = pest::Span::new(&typ, 0, typ.len()).unwrap();
                        return Err(Error::new_from_span(ErrorVariant::CustomError{message}, error_span));
                    }
                },
                _ => (),
            }
            // check for undefined variables
            if let Some(array_size) = &item.array_size {
                match array_size {
                    ArraySize::Variable(var) => {
                        if !defined_vars.contains(var) {
                            let message = format!("{}: Undefined variable '{}'", def.name, var);
                            let error_span = pest::Span::new(var, 0, var.len()).unwrap();
                            return Err(Error::new_from_span(ErrorVariant::CustomError{message}, error_span));
                        }
                    },
                    _ => (),
                }
            }
        }
    }
    let names = defined_structs.iter().map(|s| s.clone()).collect::<Vec<String>>().join(", ");
    // println!("Got a total of {} definitions: {}", defined_structs.len(), names);

    Ok(definitions)
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
    // println!("{:?}", res);
    assert!(res.is_ok(), "ak.zs");
    let res = parse_file(test);
    // println!("{:?}", res);
    assert!(res.is_ok(), "ak.zs");

    let test = "struct player { hp u8  sp i16 }";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_ok(), "simplest test");
    let res = parse_file(test);
    assert!(res.is_ok(), "simplest test");

    let test = "struct player { hp [10]u8  sp []i16 }";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_ok(), "array test");
    let res = parse_file(test);
    assert!(res.is_ok(), "array test");

    let test = "struct player { hp u8[10]  sp []i16 }";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_err(), "wrong position of brackets");
    let res = parse_file(test);
    assert!(res.is_err(), "wrong position of brackets");

    let test = "struct player { hp [5]byte  sp []i16[] }";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_err(), "wrong position of empty brackets");
    let res = parse_file(test);
    assert!(res.is_err(), "wrong position of empty brackets");

    let test = "structplayer{ hp u8 sp i16 }";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_err(), "no spaces in name");
    let res = parse_file(test);
    assert!(res.is_err(), "no spaces in name");

    let test = "struct player {hp u8sp i16}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_err(), "no space between items");
    let res = parse_file(test);
    assert!(res.is_err(), "no space between items");

    let test = "struct player {hpu8 spi16}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_ok(), "no space between type and name - becomes single item, valid grammar");
    let res = parse_file(test);
    assert!(res.is_err(), "no space between type and name - becomes single item, invalid type");

    let test = "struct player { hp u8  sp i16 } struct ship {ang [3]u8spd string}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_err(), "no space between items on second definition");
    let res = parse_file(test);
    assert!(res.is_err(), "no space between items on second definition");

    let test = "struct player { hp []u8  sp [hp]i16 } struct ship {ang [3]u8 spd string}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_ok(), "variable as array size");
    let res = parse_file(test);
    assert!(res.is_ok(), "variable as array size");

    let test = "struct player { hp []u8  sp [asdf]i16 } struct ship {ang [3]u8 spd string}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_ok(), "invalid variable as array size");
    let res = parse_file(test);
    assert!(res.is_err(), "invalid variable as array size");

    let test = "struct player { hp []u8  sp []adf } struct ship {ang [3]u8 spd string}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_ok(), "invalid type");
    let res = parse_file(test);
    assert!(res.is_err(), "invalid type");
}
