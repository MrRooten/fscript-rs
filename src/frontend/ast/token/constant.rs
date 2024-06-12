use super::base::FSRPosition;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum FSRConstantType {
    String(Vec<u8>),
    Integer(i64),
    Float(f64),
}
#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy)]
pub enum FSROrinStr<'a> {
    Integer(&'a str),
    Float(&'a str),
    String(&'a str)
}

#[derive(Debug, Clone)]
pub struct FSRConstant<'a> {
    const_str: FSROrinStr<'a>,
    constant: FSRConstantType,
    pub(crate) len: usize,
    pub(crate) single_op: Option<&'static str>,
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

    pub fn from_float(f: f64, meta: FSRPosition, s: &'a str) -> Self {
        FSRConstant {
            constant: FSRConstantType::Float(f),
            len: 0,
            single_op: None,
            meta,
            const_str: FSROrinStr::Float(s)
        }
    }

    pub fn from_int(i: i64, meta: FSRPosition, s: &'a str) -> Self {
        FSRConstant {
            constant: FSRConstantType::Integer(i),
            len: 0,
            single_op: None,
            meta,
            const_str: FSROrinStr::Integer(s)
        }
    }

    pub fn get_len(&self) -> usize {
        self.len
    }
}
