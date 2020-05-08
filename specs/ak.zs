struct Login {
    Username string
    Password string
    Version  string
}

struct ChannelList {
    Count    u16
    Channels []Channel Count
}

struct Channel {
    Idx  u16
    Name string
    unk  []byte 18
}

struct ChannelConnect {
    Key1 []byte 8
    Addr []byte 4
    Port u16
    Key2 []byte 8
}

struct SelectChannel {
    Idx u16
    Username string
    Hash string
}