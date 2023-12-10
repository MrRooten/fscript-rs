use std::{collections::HashMap, fmt::Error};

use crate::backend::base_type::base::FSRValue;

use super::base::{FSRObject, FSRObjectManager};

type FSRFuncType = fn(args: &HashMap<&str, u64>, manager: &FSRObjectManager) -> Result<FSRObject<'static>, Error>;

pub struct FSRFunction {
    value   : FSRFuncType
}

impl<'a> FSRFunction {
    pub fn new(func: FSRFuncType) -> FSRFunction {
        unimplemented!()
    }

    pub fn from_func(func: FSRFuncType) -> FSRObject<'static> {
        let v = Self {
            value: func, 
        };
        let mut obj = FSRObject::new();
        obj.set_value(FSRValue::Function(v));
        return obj;
    }

    pub fn invoke(&self, args: &HashMap<&str, u64>, manager: &FSRObjectManager) -> Result<FSRObject<'static>, Error> {
        (unsafe { self.value })(args, manager)
    }
}

pub struct FSRMethod {

}