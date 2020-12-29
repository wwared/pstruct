options endian:little scope:pstruct
struct Test {
       a    u8
       b    u16
       c    u32
       d    u64
       e    i8
       f    i16
       g    i32
       h    i64
       i    byte
       j    f32
       k    f64
       l    string
//       m    [40]cstring
       n    Wow
       o    [16]u8
       u    [3]string
       v    [2]Wow
       w    [g]byte
       x    [c]Wow
       y    []byte
       z    []string
       //z    []cstring
}

struct Wow {
       amazing      []byte
}

struct S {
       t    string
}
