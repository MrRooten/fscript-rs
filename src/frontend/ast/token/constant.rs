use super::base::FSRMeta;


#[derive(Debug, Clone)]
pub enum FSRConstantType {
    String(Vec<u8>),
    Integer(i64),
    Float(f64)
}

#[derive(Debug, Clone)]
pub struct FSRConstant {
    constant    : FSRConstantType,
    pub(crate) len         : usize,
    pub(crate) single_op   : Option<&'static str>,
    meta        : FSRMeta
}


impl FSRConstant {
    pub fn get_meta(&self) -> &FSRMeta {
        return &self.meta;
    }

    pub fn get_constant(&self) -> &FSRConstantType {
        return &self.constant;
    }

    pub fn from_str(s: &[u8], meta: FSRMeta) -> Self {
        return FSRConstant{
            constant: FSRConstantType::String(s.to_vec()),
            len: 0,
            single_op: None,
            meta
        };
    }

    pub fn from_float(f: f64, meta: FSRMeta) -> Self {
        return FSRConstant{
            constant: FSRConstantType::Float(f),
            len: 0,
            single_op: None,
            meta
        };
    }

    pub fn from_int(i: i64, meta: FSRMeta) -> Self {
        return FSRConstant{
            constant: FSRConstantType::Integer(i),
            len: 0,
            single_op: None,
            meta
        };
    }

    pub fn get_len(&self) -> usize {
        self.len
    }
}