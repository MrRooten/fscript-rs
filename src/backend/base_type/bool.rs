#![allow(unused)]

use crate::backend::base_type::base::{FSRClass, FSRObject, FSRValue, IFSRObject};
use crate::backend::vm::vm::FSRVirtualMachine;

#[derive(Debug)]
pub struct FSRBool {
    boolean     : bool
}

impl FSRBool {
    pub fn new<'a>(b: bool, vm: &'a FSRVirtualMachine<'a>, id: u64) -> &'a FSRObject<'a> {
        let b = FSRBool {
            boolean: b
        };
        let obj = FSRObject::new_with_id(vm, id);
        obj.set_cls(vm.get_cls("bool").unwrap());
        obj.set_value(FSRValue::Bool(b));
        return obj;
    }
}

impl IFSRObject for FSRBool {
    

    fn get_class_name() -> &'static str {
        "bool"
    }

    fn get_class(vm: &FSRVirtualMachine) -> FSRClass {
        let cls = FSRClass::new("bool");
        return cls;
    }
    
    fn init(&mut self) {
        
    }
}