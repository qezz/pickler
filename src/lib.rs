pub mod unpickle;

pub use unpickle::unpickle;

pub mod op {
    pub const PROTO: u8 = 0x80;
    pub const FRAME: u8 = 0x95;
    pub const STOP: u8 = 0x2e;
    pub const NONE: u8 = 0x4e;
    pub const TRUE: u8 = 0x88;
    pub const FALSE: u8 = 0x89;
    pub const BININT1: u8 = 0x4b;
    pub const BININT2: u8 = 0x4d;
    pub const BININT: u8 = 0x4a;
    pub const LONG1: u8 = 0x8a;
    pub const BINFLOAT: u8 = 0x47;
    pub const SHORT_BINUNICODE: u8 = 0x8c;
    pub const BINUNICODE: u8 = 0x58;
    pub const SHORT_BINBYTES: u8 = 0x43;
    pub const BINBYTES: u8 = 0x42;
    pub const EMPTY_LIST: u8 = 0x5d;
    pub const EMPTY_TUPLE: u8 = 0x29;
    pub const EMPTY_DICT: u8 = 0x7d;
    pub const EMPTY_SET: u8 = 0x8f;
    pub const TUPLE1: u8 = 0x85;
    pub const TUPLE2: u8 = 0x86;
    pub const TUPLE3: u8 = 0x87;
    pub const MARK: u8 = 0x28;
    pub const MEMOIZE: u8 = 0x94;
    pub const BINGET: u8 = 0x68;
    pub const LONG_BINGET: u8 = 0x6a;
    pub const SETITEM: u8 = 0x73;
    pub const SETITEMS: u8 = 0x75;
    pub const APPENDS: u8 = 0x65;
    pub const ADDITEMS: u8 = 0x90;
    pub const FROZENSET: u8 = 0x91;
    pub const GLOBAL: u8 = 0x63;
    pub const STACK_GLOBAL: u8 = 0x93;
    pub const REDUCE: u8 = 0x52;
    pub const BUILD: u8 = 0x62;
    pub const NEWOBJ: u8 = 0x81;
    pub const TUPLE: u8 = 0x74;
    pub const BINPUT: u8 = 0x71;
    pub const LONG_BINPUT: u8 = 0x72;
    pub const APPEND: u8 = 0x61;
    pub const DUP: u8 = 0x32;
    pub const POP: u8 = 0x30;
    pub const POP_MARK: u8 = 0x31;
    pub const NEWOBJ_EX: u8 = 0x92;
    pub const BINPERSID: u8 = 0x51;

    pub const PROTO_LEN: usize = 1;
    pub const FRAME_LEN: usize = 8;
    pub const BININT1_LEN: usize = 1;
    pub const BININT2_LEN: usize = 2;
    pub const BININT4_LEN: usize = 4;
    pub const BINFLOAT_LEN: usize = 8;
    pub const BINUNICODE_LEN: usize = 4;
    pub const SHORT_BINBYTES_LEN: usize = 1;
    pub const BINGET_LEN: usize = 1;
    pub const LONG_BINGET_LEN: usize = 4;
    pub const BINPUT_LEN: usize = 1;
    pub const LONG_BINPUT_LEN: usize = 4;
}

#[derive(Debug, Clone, PartialEq)]
pub enum PickleValue<'a> {
    None,
    Bool(bool),
    Int(i64),
    BigInt(&'a [u8]),
    Float(f64),
    String(&'a str),
    Bytes(&'a [u8]),
    List(Vec<PickleValue<'a>>),
    Tuple(Vec<PickleValue<'a>>),
    Dict(Vec<(PickleValue<'a>, PickleValue<'a>)>),
    Set(Vec<PickleValue<'a>>),
    FrozenSet(Vec<PickleValue<'a>>),
    Object {
        module: &'a str,
        attr: &'a str,
        args: Vec<PickleValue<'a>>,
        state: Box<PickleValue<'a>>,
    },
    Global {
        module: &'a str,
        attr: &'a str,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct PickleData<'a> {
    pub proto: u8,
    pub frame: [u8; 8],
    pub root: PickleValue<'a>,
}
