use super::base::FSRPosition;

#[derive(Debug, Clone)]
pub enum FSRConstantType {
    String(Vec<u8>),
    Integer(i64),
    Float(f64),
}

#[derive(Debug, Clone)]
pub struct FSRConstant {
    constant: FSRConstantType,
    pub(crate) len: usize,
    pub(crate) single_op: Option<&'static str>,
    meta: FSRPosition,
}

impl FSRConstant {
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
        }
    }

    pub fn from_float(f: f64, meta: FSRPosition) -> Self {
        FSRConstant {
            constant: FSRConstantType::Float(f),
            len: 0,
            single_op: None,
            meta,
        }
    }

    pub fn from_int(i: i64, meta: FSRPosition) -> Self {
        FSRConstant {
            constant: FSRConstantType::Integer(i),
            len: 0,
            single_op: None,
            meta,
        }
    }

    pub fn get_len(&self) -> usize {
        self.len
    }
}
