use crate::{PickleData, PickleValue, op::*};

#[derive(Debug)]
pub enum Error {
    TruncatedOpcode(u8),
    TruncatedData { op: u8, expected_len: usize },
    Eof,
    EmptyStack,
    UnsupportedOpcode(u8),
    InvalidUtf8,
    IndexOutOfRange { op: u8 },
    StackUnderflow { op: u8 },
    NoValueOnStack,
    NoMarkFound { op: u8 },
    SetitemWithoutDict { op: u8 },
    SetitemsWithoutDict { op: u8 },
    AppendsWithoutList { op: u8 },
    AdditemsWithoutSet { op: u8 },
    ReduceWithoutCallable { op: u8 },
    BuildWithoutObject { op: u8 },
    InvalidGlobal { op: u8 },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedOpcode(op) => write!(f, "unsupported opcode: 0x{:02x}", op),
            _ => write!(f, "UnpickleError: {:?}", self),
        }
    }
}

impl std::error::Error for Error {}

macro_rules! read_fixed {
    ($data:expr, $pos:expr, $len:expr, $op:expr) => {{
        if $pos + $len > $data.len() {
            return Err(Error::TruncatedOpcode($op));
        }

        let start = $pos;
        $pos += $len;

        let arr: [u8; $len] = $data[start..start + $len].try_into().unwrap();

        arr
    }};
}

macro_rules! read_length_prefixed {
    ($data:expr, $pos:expr, $op:expr, 1) => {{
        let size = read_fixed!($data, $pos, 1, $op);
        let n = size[0] as usize;

        if $pos + n > $data.len() {
            return Err(Error::TruncatedData {
                op: $op,
                expected_len: n,
            });
        }

        let start = $pos;
        $pos += n;
        &$data[start..start + n]
    }};
    ($data:expr, $pos:expr, $op:expr, 4) => {{
        let prefix = read_fixed!($data, $pos, 4, $op);
        let n = u32::from_le_bytes(prefix) as usize;
        // TODO: python pickle implementation reads 4 bytes as signed size, but requires it to be signed.
        // Ensure the u32 value is not over i32::MAX

        if $pos + n > $data.len() {
            return Err(Error::TruncatedData {
                op: $op,
                expected_len: n,
            });
        }

        let start = $pos;
        $pos += n;
        &$data[start..start + n]
    }};
}

#[inline]
fn pop_value<'a>(stack: &mut Vec<PickleValue<'a>>, op: u8) -> Result<PickleValue<'a>, Error> {
    stack.pop().ok_or(Error::StackUnderflow { op })
}

#[inline]
fn pop_to_mark<'a>(
    stack: &mut Vec<PickleValue<'a>>,
    marks: &mut Vec<usize>,
    op: u8,
) -> Result<Vec<PickleValue<'a>>, Error> {
    let mark_pos = marks.pop().ok_or(Error::NoMarkFound { op })?;
    Ok(stack.split_off(mark_pos))
}

pub fn unpickle(data: &[u8]) -> Result<PickleData<'_>, Error> {
    let mut pos = 0;
    let mut stack: Vec<PickleValue<'_>> = Vec::new();
    let mut marks: Vec<usize> = Vec::new();
    let mut memo: Vec<PickleValue<'_>> = Vec::new();

    let mut proto: u8 = 0;
    let mut frame: [u8; 8] = [0; 8];

    loop {
        if pos >= data.len() {
            return Err(Error::Eof);
        }

        let op = data[pos];
        pos += 1;

        match op {
            STOP => {
                break;
            }
            PROTO => {
                let b = read_fixed!(data, pos, PROTO_LEN, op);
                proto = b[0];
            }
            FRAME => {
                frame = read_fixed!(data, pos, FRAME_LEN, op);
            }
            NONE => stack.push(PickleValue::None),
            TRUE => stack.push(PickleValue::Bool(true)),
            FALSE => stack.push(PickleValue::Bool(false)),
            BININT1 => {
                let b = read_fixed!(data, pos, BININT1_LEN, op);

                stack.push(PickleValue::Int(b[0] as i64));
            }
            BININT2 => {
                let b = read_fixed!(data, pos, BININT2_LEN, op);
                let v = u16::from_le_bytes(b) as i64;

                stack.push(PickleValue::Int(v));
            }
            BININT => {
                let b = read_fixed!(data, pos, BININT4_LEN, op);
                let v = i32::from_le_bytes(b) as i64;

                stack.push(PickleValue::Int(v));
            }
            BINFLOAT => {
                let b = read_fixed!(data, pos, BINFLOAT_LEN, op);
                let v = f64::from_be_bytes(b);

                stack.push(PickleValue::Float(v));
            }
            LONG1 => {
                let b = read_length_prefixed!(data, pos, op, 1);

                stack.push(PickleValue::BigInt(b));
            }
            SHORT_BINUNICODE => {
                let b = read_length_prefixed!(data, pos, op, 1);
                let s = std::str::from_utf8(b).map_err(|_| Error::InvalidUtf8)?;

                stack.push(PickleValue::String(s));
            }
            BINUNICODE => {
                let b = read_length_prefixed!(data, pos, op, 4);
                let s = std::str::from_utf8(b).map_err(|_| Error::InvalidUtf8)?;

                stack.push(PickleValue::String(s));
            }
            SHORT_BINBYTES => {
                let b = read_length_prefixed!(data, pos, op, 1);

                stack.push(PickleValue::Bytes(b));
            }
            BINBYTES => {
                let b = read_length_prefixed!(data, pos, op, 4);

                stack.push(PickleValue::Bytes(b));
            }
            EMPTY_LIST => stack.push(PickleValue::List(Vec::new())),
            EMPTY_TUPLE => stack.push(PickleValue::Tuple(Vec::new())),
            EMPTY_DICT => stack.push(PickleValue::Dict(Vec::new())),
            EMPTY_SET => stack.push(PickleValue::Set(Vec::new())),
            TUPLE1 => {
                let a = pop_value(&mut stack, op)?;

                stack.push(PickleValue::Tuple(vec![a]));
            }
            TUPLE2 => {
                let b = pop_value(&mut stack, op)?;
                let a = pop_value(&mut stack, op)?;

                stack.push(PickleValue::Tuple(vec![a, b]));
            }
            TUPLE3 => {
                let c = pop_value(&mut stack, op)?;
                let b = pop_value(&mut stack, op)?;
                let a = pop_value(&mut stack, op)?;

                stack.push(PickleValue::Tuple(vec![a, b, c]));
            }
            MARK => marks.push(stack.len()),
            MEMOIZE => {
                let top = stack.last().ok_or(Error::NoValueOnStack)?.clone();

                memo.push(top);
            }
            BINGET => {
                let idx = read_fixed!(data, pos, BINGET_LEN, op)[0] as usize;
                let v = memo.get(idx).ok_or(Error::IndexOutOfRange { op })?.clone();

                stack.push(v);
            }
            LONG_BINGET => {
                let bytes = read_fixed!(data, pos, LONG_BINGET_LEN, op);
                let idx = u32::from_le_bytes(bytes) as usize;

                let v = memo.get(idx).ok_or(Error::IndexOutOfRange { op })?.clone();

                stack.push(v);
            }
            SETITEM => {
                let value = pop_value(&mut stack, op)?;
                let key = pop_value(&mut stack, op)?;

                match stack.last_mut() {
                    Some(PickleValue::Dict(entries)) => {
                        entries.push((key, value));
                    }
                    _ => return Err(Error::SetitemWithoutDict { op }),
                }
            }
            SETITEMS => {
                let items = pop_to_mark(&mut stack, &mut marks, op)?;
                let mut iter = items.into_iter();
                let mut pairs = Vec::new();

                while let Some(key) = iter.next() {
                    let value = iter.next().unwrap_or(PickleValue::None);
                    pairs.push((key, value));
                }

                match stack.last_mut() {
                    Some(PickleValue::Dict(entries)) => {
                        entries.extend(pairs);
                    }
                    _ => return Err(Error::SetitemsWithoutDict { op }),
                }
            }
            APPENDS => {
                let items = pop_to_mark(&mut stack, &mut marks, op)?;

                match stack.last_mut() {
                    Some(PickleValue::List(list)) => {
                        list.extend(items);
                    }
                    _ => return Err(Error::AppendsWithoutList { op }),
                }
            }
            ADDITEMS => {
                let items = pop_to_mark(&mut stack, &mut marks, op)?;

                match stack.last_mut() {
                    Some(PickleValue::Set(set)) => {
                        set.extend(items);
                    }
                    _ => return Err(Error::AdditemsWithoutSet { op }),
                }
            }
            FROZENSET => {
                let items = pop_to_mark(&mut stack, &mut marks, op)?;

                stack.push(PickleValue::FrozenSet(items));
            }
            TUPLE => {
                let items = pop_to_mark(&mut stack, &mut marks, op)?;

                stack.push(PickleValue::Tuple(items));
            }
            GLOBAL => {
                let rest = &data[pos..];
                let first_nl = rest.iter().position(|&b| b == b'\n').ok_or(Error::Eof)?;
                let after_first = &rest[first_nl + 1..];
                let second_nl = after_first
                    .iter()
                    .position(|&b| b == b'\n')
                    .ok_or(Error::Eof)?;

                let module =
                    std::str::from_utf8(&rest[..first_nl]).map_err(|_| Error::InvalidUtf8)?;
                let attr = std::str::from_utf8(&after_first[..second_nl])
                    .map_err(|_| Error::InvalidUtf8)?;

                pos += first_nl + 1 + second_nl + 1;

                stack.push(PickleValue::Global { module, attr });
            }
            STACK_GLOBAL => {
                let attr_val = pop_value(&mut stack, op)?;
                let module_val = pop_value(&mut stack, op)?;

                match (module_val, attr_val) {
                    (PickleValue::String(m), PickleValue::String(a)) => {
                        stack.push(PickleValue::Global { module: m, attr: a });
                    }
                    _ => return Err(Error::InvalidGlobal { op }),
                }
            }
            REDUCE => {
                let args_val = pop_value(&mut stack, op)?;
                let callable = pop_value(&mut stack, op)?;

                let (module, attr) = match callable {
                    PickleValue::Global { module, attr } => (module, attr),
                    _ => return Err(Error::ReduceWithoutCallable { op }),
                };

                let args = match args_val {
                    PickleValue::Tuple(items) => items,
                    _ => vec![args_val],
                };

                stack.push(PickleValue::Object {
                    module,
                    attr,
                    args,
                    state: Box::new(PickleValue::None),
                });
            }
            NEWOBJ => {
                let args_val = pop_value(&mut stack, op)?;
                let callable = pop_value(&mut stack, op)?;

                let (module, attr) = match callable {
                    PickleValue::Global { module, attr } => (module, attr),
                    _ => return Err(Error::ReduceWithoutCallable { op }),
                };

                let args = match args_val {
                    PickleValue::Tuple(items) => items,
                    _ => vec![args_val],
                };

                stack.push(PickleValue::Object {
                    module,
                    attr,
                    args,
                    state: Box::new(PickleValue::None),
                });
            }
            NEWOBJ_EX => {
                let _kwargs = pop_value(&mut stack, op)?;
                let args_val = pop_value(&mut stack, op)?;
                let callable = pop_value(&mut stack, op)?;

                let (module, attr) = match callable {
                    PickleValue::Global { module, attr } => (module, attr),
                    _ => return Err(Error::ReduceWithoutCallable { op }),
                };

                let args = match args_val {
                    PickleValue::Tuple(items) => items,
                    _ => vec![args_val],
                };

                stack.push(PickleValue::Object {
                    module,
                    attr,
                    args,
                    state: Box::new(PickleValue::None),
                });
            }
            BUILD => {
                let state = pop_value(&mut stack, op)?;

                match stack.last_mut() {
                    Some(PickleValue::Object {
                        state: obj_state, ..
                    }) => {
                        *obj_state = Box::new(state);
                    }
                    _ => return Err(Error::BuildWithoutObject { op }),
                }
            }
            BINPUT => {
                let idx = read_fixed!(data, pos, BINPUT_LEN, op)[0] as usize;
                let top = stack.last().ok_or(Error::NoValueOnStack)?.clone();

                if idx >= memo.len() {
                    memo.resize(idx + 1, PickleValue::None);
                }

                memo[idx] = top;
            }
            LONG_BINPUT => {
                let bytes = read_fixed!(data, pos, LONG_BINPUT_LEN, op);
                let idx = u32::from_le_bytes(bytes) as usize;
                let top = stack.last().ok_or(Error::NoValueOnStack)?.clone();

                if idx >= memo.len() {
                    memo.resize(idx + 1, PickleValue::None);
                }

                memo[idx] = top;
            }
            APPEND => {
                let item = pop_value(&mut stack, op)?;

                match stack.last_mut() {
                    Some(PickleValue::List(list)) => {
                        list.push(item);
                    }
                    _ => return Err(Error::AppendsWithoutList { op }),
                }
            }
            DUP => {
                let top = stack.last().ok_or(Error::NoValueOnStack)?.clone();

                stack.push(top);
            }
            POP => {
                // TODO: check if this should error out
                stack.pop();
            }
            POP_MARK => {
                pop_to_mark(&mut stack, &mut marks, op)?;
            }
            BINPERSID => {
                let pid = pop_value(&mut stack, op)?;

                stack.push(pid);
            }

            _ => return Err(Error::UnsupportedOpcode(op)),
        }
    }

    let val = stack.pop().ok_or(Error::EmptyStack)?;

    Ok(PickleData {
        proto,
        frame,
        root: val,
    })
}
