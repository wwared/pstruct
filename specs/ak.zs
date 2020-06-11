struct Login {
    Username string
    Password string
    Version  string
}

struct ChannelList {
    Count    u16
    Channels [Count]Channel
}

struct Channel {
    Idx  u16
    Name string
    unk  [18]byte
}

struct ChannelConnect {
    Key1 [8]byte
    Addr [4]byte
    Port u16
    Key2 [8]byte
}

struct SelectChannel {
    Idx u16
    Username string
    Hash string
}
