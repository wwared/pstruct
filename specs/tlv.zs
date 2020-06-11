struct TLV {
    Tag   u16
    Value []byte
}

struct BatchTLV {
    List  []TLV
}
