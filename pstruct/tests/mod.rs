#[test]
fn parser_tests() {
    //     let test = "
    // struct player {
    //     hp u8
    //     sp i16
    // }";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_ok(), "simplest test");

    //     let test = "
    // struct player {
    //     hp [10]u8
    //     sp []i16
    // }";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_ok(), "array test");

    //     let test = "
    // struct player {
    //     hp u8[10]
    //     sp []i16
    // }";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_err(), "wrong position of brackets");

    //     let test = "
    // struct player {
    //     hp [5]byte
    //     sp []i16[]
    // }";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_err(), "wrong position of empty brackets");

    //     let test = "
    // structplayer{
    //     hp u8
    //     sp i16
    // }";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_err(), "no spaces in name");

    //     let test = "
    // struct player {
    //     hp u8sp i16
    // }";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_err(), "no space between items");

    //     let test = "
    // struct player {
    //     hpu8 spi16
    // }";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_err(), "no space between type and name - becomes single item, invalid type");

    //     let test = "
    // struct player {
    //     hp u8
    //     sp i16
    // }
    // struct ship {
    //     ang [3]
    //     u8spd string
    // }";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_err(), "no space between items on second definition");

    //     let test = "
    // struct player {
    //     hp []u8
    //     sp [hp]i16
    // }
    // struct ship {
    //     ang [3]u8
    //     spd string
    // }";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_ok(), "variable as array size");

    //     let test = "
    // struct player {
    //     hp []u8
    //     sp [asdf]i16
    // }

    // struct ship {
    //     ang [3]u8
    //     spd string
    // }";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_err(), "invalid variable as array size");

    //     let test = "
    // struct player {
    //     hp []u8
    //     sp []adf
    // }

    // struct ship {
    //     ang [3]u8
    //     spd string
    // }";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_err(), "invalid type");

    //     let test = "struct player {  hp []u8   sp []u16  }";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_err(), "needs line endings");

    //     let test = "struct player {  hp []u8
    // sp []u16  }";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_err(), "needs line endings on both ends");

    //     let test = "struct player {  hp []u8
    // sp []u16
    // }";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_err(), "needs line endings at beginning of struct");

    //     let test = "struct player {
    // hp []u8
    // sp []u16 }";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_err(), "needs line endings at end of struct");

    //     let test = "struct player {
    //     hp
    //  []u8
    //     sp
    // []u16
    // }";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_err(), "no line endings in middle of items");

    //     let test = "
    // struct
    // player {
    //     hp []u8
    //     sp []u16
    // }";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_err(), "no line ending in struct name");

    //     let test = "
    // struct   player
    // {
    //     hp []u8
    //     sp []u16
    // }";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_ok(), "line ending after name is ok, also spaces between struct and name");

    //     let test = "
    // /* hey look */ struct player // comments
    // {  // work fine
    //     hp []u8 /* real cool! */
    // /* multi
    // line
    // */ sp []u16 // all the way to the end of the line
    // /* wow */} // amazing
    // ";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_ok(), "comments, wow");

    //     let test = "
    // struct /* can't comment everywhere though */ player
    // {
    //     hp /* here doesn't work either */ []u8
    //     sp []u16
    // }";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_err(), "no comments between struct and name");

    //     let test = "
    // struct player
    // {
    //     hp []u8 /* multi line comments
    // can eat your newlines */  sp []u16
    // }";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_err(), "newline removed by multiline comment");

    //     let test = "
    // struct player
    // {
    //     hpCount u64
    //     hp [hpCount]u8
    //     sp [spCount]u16
    //     spCount i32
    // }";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_err(), "can't put count after array");

    //     let test = "
    // struct player
    // {
    //     hpCount u64
    //     hp [hpCount]u8  array_size_type:u8
    //     sp []u16
    // }";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_err(), "can't declare array size twice");

    //     let test = "
    // struct   player
    // {
    //     hp u8  endian:big
    //     sp u16  endian:little
    // }";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_ok(), "inline options with :");

    //     let test = "
    // struct   player
    // {
    //     hp u8  endian big
    //     sp u16  endian little
    // }";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_ok(), "inline options with space");

    //     let test = "
    // struct   player
    // {
    //     hp u8  endian big endian:little
    //     sp u16
    // }";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_ok(), "mixed inline options");

    //     let test = "
    // options scope test endian:big
    // struct   player
    // {
    //     hp u8
    //     sp u16 endian little
    // }";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_ok(), "inline file options");

    //     let test = "
    // options {
    //     scope test
    //     endian:big
    // }
    // struct   player
    // {
    //     hp u8
    //     sp u16 endian little
    // }";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_ok(), "multiline file options");

    //     let test = "
    // options {
    //     scope test  endian:big
    // }
    // struct   player
    // {
    //     hp u8
    //     sp u16 endian little
    // }";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_err(), "cannot use inline file options in block");

    //     let test = "
    // options {
    //     scope test
    //     endian:big
    // }
    // options scope test endian:big
    // struct   player
    // {
    //     hp u8
    //     sp u16 endian little
    // }";
    //     let res = parser::parse_file(test);
    //     assert!(res.is_err(), "only one file option block");
}
