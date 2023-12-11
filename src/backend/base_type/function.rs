use std::{collections::HashMap, fmt::Error};

use crate::backend::base_type::base::FSRValue;

use super::base::{FSRObject, FSRObjectManager};


type FSRFuncType = fn(args: &HashMap<&str, u64>, manager: &FSRObjectManager) -> Result<FSRObject<'static>, Error>;


pub struct FSRFunction {
    value       : FSRFuncType,
    identify    : u32
}

impl std::fmt::Debug for FSRFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FSRFunction").field("value", &self.identify).finish()
    }
}

impl<'a> FSRFunction {
    pub fn new(func: FSRFuncType) -> FSRFunction {
        unimplemented!()
    }

    pub fn from_func(func: FSRFuncType) -> FSRObject<'static> {
        let v = Self {
            value: func,
            identify: 0, 
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