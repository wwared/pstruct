// use pstruct_derive::pstruct;

// pstruct!("test.zs");

mod test;
use pstruct_rs::Pstruct;
use test::*;

fn main() {
    use std::ffi::CString;
    let test = pstruct::Test {
        a: 1,
        b: 2,
        c: 3,
        d: 4,
        e: -100,
        f: -150,
        g: 3,
        h: -420,
        i: 255,
        j: 6.9,
        k: 4.2,
        l: "AAAAA".to_string(),
        m: CString::new("YAAAY").unwrap(),
        n: pstruct::Wow {
            amazing: vec![0xCA, 0xFE, 0xF0, 0x0D],
        },
        o: [0x41; 16],
        u: ["A".to_string(), "BB".to_string(), "CCC".to_string()],
        v: [
            pstruct::Wow {
                amazing: vec![1, 2, 3],
            },
            pstruct::Wow {
                amazing: vec![4, 5, 6],
            },
        ],
        w: vec![1, 2, 3],
        x: vec![
            pstruct::Wow {
                amazing: vec![0xa, 0xb],
            },
            pstruct::Wow {
                amazing: vec![0xff],
            },
            pstruct::Wow {
                amazing: vec![0x41; 16],
            },
        ],
        y: vec![0xff, 0xaa, 0xbb],
        z: CString::new("OK").unwrap(),
    };
    println!("{:#?}", test);
    println!("Size: {}", test.size());
    let encoded = test.encode().unwrap();
    println!("{:x?}", encoded);
    let decoded = pstruct::Test::decode_new(&encoded).unwrap();
    println!("{:#?}", decoded);
}
