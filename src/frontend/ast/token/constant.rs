use super::base::FSRPosition;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum FSRConstantType {
    String(Vec<u8>),
    Integer(i64),
    Float(f64),
}
#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy)]
pub enum FSROrinStr<'a> {
    Integer(&'a str, Option<&'a str>),
    Float(&'a str, Option<&'a str>),
    String(&'a str)
}

impl FSROrinStr<'_> {
    pub fn to_2(&self) -> FSROrinStr2 {
        match self {
            FSROrinStr::Integer(i, op) => FSROrinStr2::Integer(i.to_string(), op.map(|x| x.to_string())),
            FSROrinStr::Float(f, op) => FSROrinStr2::Float(f.to_string(), op.map(|x| x.to_string())),
            FSROrinStr::String(s) => FSROrinStr2::String(s.to_string()),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum FSROrinStr2 {
    Integer(String, Option<String>),
    Float(String, Option<String>),
    String(String)
}

#[derive(Debug, Clone)]
pub struct FSRConstant<'a> {
    const_str: FSROrinStr<'a>,
    constant: FSRConstantType,
    pub(crate) len: usize,
    pub(crate) single_op: Option<&'a str>,
    meta: FSRPosition,
}

impl<'a> FSRConstant<'a> {
    pub fn get_const_str(&self) -> &FSROrinStr<'a> {
        &self.const_str
    }

    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }

    pub fn get_constant(&self) -> &FSRConstantType {
        &self.constant
    }

    pub fn from_str(s: &'a [u8], meta: FSRPosition) -> Self {
        FSRConstant {
            constant: FSRConstantType::String(s.to_vec()),
            len: 0,
            single_op: None,
            meta,
            const_str: FSROrinStr::String(unsafe { std::str::from_utf8_unchecked(s) })
        }
    }

    pub fn from_float(f: f64, meta: FSRPosition, s: &'a str, op: Option<&'a str>) -> Self {
        FSRConstant {
            constant: FSRConstantType::Float(f),
            len: 0,
            single_op: op,
            meta,
            const_str: FSROrinStr::Float(s, op)
        }
    }

    pub fn from_int(i: i64, meta: FSRPosition, s: &'a str, op: Option<&'a str>) -> Self {
        FSRConstant {
            constant: FSRConstantType::Integer(i),
            len: 0,
            single_op: op,
            meta,
            const_str: FSROrinStr::Integer(s, op)
        }
    }

    pub fn get_len(&self) -> usize {
        self.len
    }
}
