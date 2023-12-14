
pub enum FSRConstantType<'a> {
    String(&'a str),
    Integer(i64),
    Float(f64)
}

pub struct FSRConstant<'a> {
    constant    : FSRConstantType<'a>
}


impl<'a> FSRConstant<'a> {
    pub fn from_str(s: &'a str) -> Self {
        return FSRConstant{
            constant: FSRConstantType::String(s)
        };
    }

    pub fn from_float(f: f64) -> Self {
        return FSRConstant{
            constant: FSRConstantType::Float(f)
        };
    }

    pub fn from_int(i: i64) -> Self {
        return FSRConstant{
            constant: FSRConstantType::Integer(i)
        };
    }
}