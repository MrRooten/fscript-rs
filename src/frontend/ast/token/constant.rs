use super::{base::{FSRPosition, FSRType}, expr::SingleOp};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum FSRConstantType {
    String(Vec<u8>),
    Integer(i64),
    Float(f64),
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum FSROrinStr {
    Integer(String, Option<SingleOp>),
    Float(String, Option<SingleOp>),
    String(String),
}

impl FSROrinStr {
    pub fn to_2(&self) -> FSROrinStr2 {
        match self {
            FSROrinStr::Integer(i, op) => {
                FSROrinStr2::Integer(i.to_string(), op.clone())
            }
            FSROrinStr::Float(f, op) => {
                FSROrinStr2::Float(f.to_string(), op.clone())
            }
            FSROrinStr::String(s) => FSROrinStr2::String(s.to_string()),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum FSROrinStr2 {
    Integer(String, Option<SingleOp>),
    Float(String, Option<SingleOp>),
    String(String),
}

#[derive(Debug, Clone)]
pub struct FSRConstant {
    const_str: FSROrinStr,
    constant: FSRConstantType,
    pub(crate) len: usize,
    pub(crate) single_op: Option<SingleOp>,
    meta: FSRPosition,
}

impl FSRConstant {
    pub fn get_const_str(&self) -> &FSROrinStr {
        &self.const_str
    }

    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }

    pub fn get_constant(&self) -> &FSRConstantType {
        &self.constant
    }

    pub fn from_str(s: &[u8], meta: FSRPosition) -> Self {
        FSRConstant {
            constant: FSRConstantType::String(s.to_vec()),
            len: 0,
            single_op: None,
            meta,
            const_str: FSROrinStr::String(unsafe { std::str::from_utf8_unchecked(s) }.to_string()),
        }
    }

    pub fn from_float(f: f64, meta: FSRPosition, s: &str, op: Option<SingleOp>) -> Self {
        FSRConstant {
            constant: FSRConstantType::Float(f),
            len: 0,
            single_op: op,
            meta,
            const_str: FSROrinStr::Float(s.to_string(), op),
        }
    }

    pub fn from_int(i: i64, meta: FSRPosition, s: &str, op: Option<SingleOp>) -> Self {
        FSRConstant {
            constant: FSRConstantType::Integer(i),
            len: 0,
            single_op: op,
            meta,
            const_str: FSROrinStr::Integer(s.to_string(), op.clone()),
        }
    }

    pub fn get_len(&self) -> usize {
        self.len
    }

    pub fn deduction(&self) -> FSRType {
        match &self.constant {
            FSRConstantType::String(_) => FSRType::new("String"),
            FSRConstantType::Integer(_) => FSRType::new("Integer"),
            FSRConstantType::Float(_) => FSRType::new("Float"),
        }
    }
}
