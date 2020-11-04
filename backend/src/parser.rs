use std::collections::BTreeSet;
use pest::Parser;
use pest::error::ErrorVariant;
use pest::iterators::Pair;

use crate::types::*;

pub type Error = pest::error::Error<Rule>;

fn make_error<S: Into<String>>(msg: S, span: pest::Span) -> Error {
    Error::new_from_span(ErrorVariant::CustomError{message: msg.into()}, span)
}

#[derive(Parser)]
#[grammar = "struct.pest"]
struct StructParser;

#[derive(Clone)] // TODO get rid of all the unnecessary copies somehow?
struct FileOptions {
    scope_name: String,
    endian: Endian,
}

struct ItemOptions<'a> {
    array_size_type: Option<Type<'a>>,

    endian: Endian,
}

fn default_file_options() -> FileOptions {
    FileOptions { scope_name: "main".to_owned(), endian: Endian::Little }
}

fn default_item_options(file_options: FileOptions) -> ItemOptions<'static> {
    ItemOptions { array_size_type: None, endian: file_options.endian }
}

// unwraps look spooky but the grammar says it's fine
fn parse_definition(pair: Pair<Rule>, file_options: FileOptions) -> Result<Struct, Error>{
    assert!(pair.as_rule() == Rule::definition, "expected definition");
    let mut inner_rules = pair.into_inner();
    // struct_name -> identifier -> as_str
    let name = inner_rules.next().unwrap().into_inner().next().unwrap().as_str();
    let mut items: Vec<Item> = vec![];
    // all other rules are for items
    for item_pair in inner_rules {
        let next_item = parse_item(item_pair, &items, file_options.clone())?;
        items.push(next_item);
    }
    Ok(Struct {name, items})
}

fn parse_item_type(type_name: &str) -> Type {
    match type_name {
        "u8" => Type::U8, "u16" => Type::U16, "u32" => Type::U32, "u64" => Type::U64,
        "i8" => Type::I8, "i16" => Type::I16, "i32" => Type::I32, "i64" => Type::I64,
        "f32" => Type::F32, "f64" => Type::F64,
        "byte" => Type::Byte, "string" => Type::String, "cstring" => Type::CString,
        _ => Type::User(type_name)
    }
}

fn parse_single_option(option: Pair<Rule>) -> (&str, &str) {
    assert!(option.as_rule() == Rule::option, "expected option");
    let mut inner = option.into_inner();
    let key = inner.next().unwrap().as_str();
    let value = inner.next().unwrap().as_str();
    (key, value)
}

fn parse_file_options(pair: Pair<Rule>, defaults: FileOptions) -> Result<FileOptions, Error> {
    let mut res = defaults;
    assert!(pair.as_rule() == Rule::file_options, "expected file options");
    let pair = pair.into_inner().next().unwrap();
    assert!(pair.as_rule() == Rule::multiline_options || pair.as_rule() == Rule::inline_options, "unexpected option type");
    // let multiline_options = pair.into_inner().into_inner();
    for option in pair.into_inner() {
        let err_span = option.as_span();
        let (key, value) = parse_single_option(option);
        match key {
            "scope" => {
                res.scope_name = value.to_owned();
            },
            "endian" => { // TODO put this endianness parsing in its own fn?
                res.endian = match value {
                    "big"    => { Endian::Big },
                    "little" => { Endian::Little },
                    _        => {
                        return Err(make_error(format!("unknown endianness {}", value), err_span))
                    }
                };
            }
            _ => return Err(make_error(format!("unknown option {}", key), err_span))
        }
    }
    Ok(res)
}

fn parse_item_options(pair: Pair<Rule>, file_options: FileOptions) -> Result<ItemOptions, Error> {
    let mut res = default_item_options(file_options);
    assert!(pair.as_rule() == Rule::inline_options, "expected options");
    for option in pair.into_inner() {
        let err_span = option.as_span();
        let (key, value) = parse_single_option(option);
        match key {
            // "max_array_size" => {
            //     let size = value.parse::<usize>().unwrap();
            //     res.max_array_size = Some(size);
            // }, // TODO bounded
            "array_size_type" => {
                let kind = parse_item_type(value);
                match kind {
                    Type::Byte | Type::String | Type::User(_) => {
                        return Err(make_error("array_size_type must be integer valued", err_span));
                    },
                    _ => {  },
                };
                res.array_size_type = Some(kind);
            },
            "endian" => {
                res.endian = match value {
                    "big"    => { Endian::Big },
                    "little" => { Endian::Little },
                    _        => {
                        return Err(make_error(format!("unknown endianness {}", value), err_span))
                    }
                };
            },
            _ => return Err(make_error(format!("unknown option {}", key), err_span))
        }

    }
    Ok(res)
}

fn parse_item<'a>(pair: Pair<'a, Rule>, environment: &[Item<'a>], file_options: FileOptions) -> Result<Item<'a>, Error> {
    assert!(pair.as_rule() == Rule::struct_item, "expected struct item");
    let mut inner_rules = pair.into_inner();
    let name = inner_rules.next().unwrap().as_str();
    let type_pair = inner_rules.next().unwrap();
    assert!(type_pair.as_rule() == Rule::type_decl, "expected type declaration");

    let item_options = if let Some(opts_pair) = inner_rules.next() {
        parse_item_options(opts_pair, file_options)?
    } else {
        default_item_options(file_options)
    };

    let array: Option<Array>;
    let item_type: Type;
    let mut type_inner = type_pair.into_inner();
    let first_elem = type_inner.next().unwrap();
    let err_span = first_elem.as_span();
    match first_elem.as_rule() {
        Rule::array_brackets => {
            if let Some(arr_pair) = first_elem.into_inner().next() {
                let arr_str = arr_pair.as_str();
                array = match arr_str.parse::<usize>() {
                    Ok(size) => Some(Array::Constant(size)),
                    Err(_) => {
                        // find the type of previously declared variable
                        if item_options.array_size_type.is_some() {
                            return Err(make_error("cannot declare type for array with known size", err_span));
                        }
                        let other_item = environment.iter().find(|i| i.name == arr_str);
                        if let Some(other_item) = other_item {
                            Some(Array::Variable(arr_str, other_item.kind.clone()))
                        } else {
                            return Err(make_error(format!("undeclared identifier {}", arr_str), err_span))
                        }
                    },
                };
            } else {
                array = Some(Array::Unknown(item_options.array_size_type.unwrap_or(Type::I32)));
            }
            item_type = parse_item_type(type_inner.next().unwrap().as_str());
        },
        Rule::item_identifier => {
            array = None;
            item_type = parse_item_type(first_elem.as_str());
        },
        _ => {
            return Err(make_error("expected array specifier or struct item", err_span))
        }
    };
    Ok(Item { name, kind: item_type, array, byte_order: item_options.endian, })
}


pub fn parse_file(file_contents: &str) -> Result<File, Error> {
    let parse_res = StructParser::parse(Rule::file, file_contents)?;

    let mut definitions = vec![];
    let mut defined_structs = BTreeSet::new();
    let mut defined_vars = BTreeSet::new();

    let mut file_options = default_file_options();

    for pair in parse_res {
        if pair.as_rule() == Rule::EOI { break; }

        if pair.as_rule() == Rule::file_options {
            file_options = parse_file_options(pair, file_options)?;
            continue;
        }

        let def = parse_definition(pair, file_options.clone())?;

        if def.items.is_empty() {
            eprintln!("Ignoring empty struct definition '{}'", def.name);
            continue;
        }

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
                    let error_span = pest::Span::new(&typ, 0, typ.len()).unwrap(); // TODO improve message?
                    return Err(make_error(format!("{}: undefined type {}", def.name, typ), error_span));
                }
            }
            // check for undefined variables
            if let Some(Array::Variable(var, _)) = &item.array {
                if !defined_vars.contains(var) {
                    let error_span = pest::Span::new(var, 0, var.len()).unwrap(); // TODO improve message?
                    return Err(make_error(format!("{}: undefined variable {}", def.name, var), error_span));
                }
            }
        }
    }
    let names = defined_structs.iter().cloned().collect::<Vec<&str>>().join(", ");
    println!("{} definitions: {}", defined_structs.len(), names);

    Ok(File { scope: file_options.scope_name, structs: definitions })
}

#[cfg(test)]
#[test]
fn parser_tests() {
    // TODO: check for an easy way to remove the repetition
    // let test = include_str!("../specs/simple.zs");
    // let res = StructParser::parse(Rule::file, test);
    // assert!(res.is_ok(), "simple.zs");
    // let res = parse_file(test);
    // assert!(res.is_ok(), "simple.zs");

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

    let test = "
struct player
{
    hpCount u64
    hp [hpCount]u8
    sp [spCount]u16
    spCount i32
}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_ok(), "can't put count after array");
    let res = parse_file(test);
    assert!(res.is_err(), "can't put count after array");

    let test = "
struct player
{
    hpCount u64
    hp [hpCount]u8  array_size_type:u8
    sp []u16
}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_ok(), "can't declare array size twice");
    let res = parse_file(test);
    assert!(res.is_err(), "can't declare array size twice");

    let test = "
struct   player
{
    hp u8  endian:big
    sp u16  endian:little
}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_ok(), "inline options with :");
    let res = parse_file(test);
    assert!(res.is_ok(), "inline options with :");

    let test = "
struct   player
{
    hp u8  endian big
    sp u16  endian little
}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_ok(), "inline options with space");
    let res = parse_file(test);
    assert!(res.is_ok(), "inline options with space");

    let test = "
struct   player
{
    hp u8  endian big endian:little
    sp u16
}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_ok(), "mixed inline options");
    let res = parse_file(test);
    assert!(res.is_ok(), "mixed inline options");

    let test = "
options scope test endian:big
struct   player
{
    hp u8
    sp u16 endian little
}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_ok(), "inline file options");
    let res = parse_file(test);
    assert!(res.is_ok(), "inline file options");

    let test = "
options {
    scope test
    endian:big
}
struct   player
{
    hp u8
    sp u16 endian little
}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_ok(), "multiline file options");
    let res = parse_file(test);
    assert!(res.is_ok(), "multiline file options");

    let test = "
options {
    scope test  endian:big
}
struct   player
{
    hp u8
    sp u16 endian little
}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_err(), "cannot use inline file options in block");
    let res = parse_file(test);
    assert!(res.is_err(), "cannot use inline file options in block");

    let test = "
options {
    scope test
    endian:big
}
options scope test endian:big
struct   player
{
    hp u8
    sp u16 endian little
}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_err(), "only one file option block");
    let res = parse_file(test);
    assert!(res.is_err(), "only one file option block");

    let test = "
struct cstringtest {
  s1 cstring
  s2 []cstring
  s3 [5]cstring
}";
    let res = StructParser::parse(Rule::file, test);
    assert!(res.is_ok(), "all kinds of cstrings supported");
    let res = parse_file(test);
    assert!(res.is_ok(), "all kinds of cstrings supported");
}
