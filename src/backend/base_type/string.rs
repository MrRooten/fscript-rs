use crate::backend::vm::runtime::FSRThreadRuntime;
use crate::backend::{base_type::function::FSRFn, vm::vm::FSRVirtualMachine};
use crate::backend::base_type::utils::i_to_m;
use crate::utils::error::FSRRuntimeError;

use super::{base::{FSRBaseType, FSRObject, FSRValue, IFSRObject}, integer::FSRInteger};

#[derive(Debug)]
pub struct FSRString {
    value       : String
}

impl FSRString {
    fn register_len_func<'a>(vm: &'a FSRVirtualMachine, rt: &mut FSRThreadRuntime) -> Result<u64, FSRRuntimeError<'a>> {
        unimplemented!()
    }

    fn register_to_string_func<'a>(vm: &'a FSRVirtualMachine, rt: &mut FSRThreadRuntime) -> Result<u64, FSRRuntimeError<'a>> {
        unimplemented!()
    }

    pub fn from<'a, T>(s: T, vm: &'a FSRVirtualMachine<'a>) -> &FSRObject<'a>
    where T: ToString {
        let v = FSRString {
            value: s.to_string()
        };

        let obj = FSRObject::new(vm);
        obj.set_cls(vm.get_cls("String").unwrap());
        obj.set_value(FSRValue::String(v));
        return obj;
    }

    pub fn from_str<'a>(s: String, vm: &'a FSRVirtualMachine<'a>) -> &FSRObject<'a> {
        let obj = FSRObject::new(vm);
        let v = FSRString {
            value: s
        };
        obj.set_cls(vm.get_cls("String").unwrap());
        obj.set_value(FSRValue::String(v));
        return obj;
    }

    pub fn get_string(&self) -> &String {
        return &self.value;
    }
}

impl IFSRObject for FSRString {
    fn get_class_name() -> &'static str {
        "String"
    }

    fn get_class(vm: &FSRVirtualMachine) -> FSRBaseType {
        let mut cls = FSRBaseType::new("String");
        let fn_obj = FSRFn::from_func(FSRString::register_len_func, vm, vec!["self"]);
        cls.register_obj("len", fn_obj.get_id());
        let fn_obj = FSRFn::from_func(FSRString::register_to_string_func, vm, vec!["self"]);
        cls.register_obj("__str__", fn_obj.get_id());
        return cls;
    }
    
    fn init(&mut self) {
        todo!()
    }
}

