struct Inner {
  U8 u8
  U16 u16
}

struct Test {
  U8 u8
  U16 u16
  U32 u32
  U64 u64
  I8 i8
  I16 i16
  I32 i32
  I64 i64
  Byte byte
  Str string
  U8Arr [5]u8
  StrArr [3]string
  UnsizedArr []u8
  Child Inner
  VarArr [Byte]byte
  End u64
}
