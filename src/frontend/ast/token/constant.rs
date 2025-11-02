use super::{base::{FSRPosition, FSRType}, expr::SingleOp};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum FSRConstantType {
    String(Vec<u8>),
    Integer(String),
    Float(String),
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
                FSROrinStr2::Integer(i.to_string(), *op)
            }
            FSROrinStr::Float(f, op) => {
                FSROrinStr2::Float(f.to_string(), *op)
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

pub struct FSRFormatStruct {
    pub format_str: String,
    pub arg_strings: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum FSRConstType {
    Normal,
    FormatString,
    RegexString,
}

#[derive(Debug, Clone)]
pub struct FSRConstant {
    const_str: FSROrinStr,
    constant: FSRConstantType,
    const_type: FSRConstType,
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

    pub fn convert_str_type(type_str: &str) -> FSRConstType {
        match type_str {
            "f" => FSRConstType::FormatString,
            "r" => FSRConstType::RegexString,
            _ => FSRConstType::Normal,
        }
    }

    pub fn from_str(s: &[u8], meta: FSRPosition, str_type: FSRConstType) -> Self {
        FSRConstant {
            constant: FSRConstantType::String(s.to_vec()),
            len: 0,
            const_type: str_type,
            single_op: None,
            meta,
            const_str: FSROrinStr::String(unsafe { std::str::from_utf8_unchecked(s) }.to_string()),
        }
    }

    pub fn from_float(meta: FSRPosition, s: &str, op: Option<SingleOp>) -> Self {
        FSRConstant {
            constant: FSRConstantType::Float(s.to_string()),
            len: 0,
            single_op: op,
            const_type: FSRConstType::Normal,
            meta,
            const_str: FSROrinStr::Float(s.to_string(), op),
        }
    }

    pub fn from_int(meta: FSRPosition, s: &str, op: Option<SingleOp>) -> Self {
        FSRConstant {
            constant: FSRConstantType::Integer(s.to_string()),
            len: 0,
            single_op: op,
            const_type: FSRConstType::Normal,
            meta,
            const_str: FSROrinStr::Integer(s.to_string(), op),
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
