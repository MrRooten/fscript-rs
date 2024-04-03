use crate::backend::vm::module::FSRRuntimeModule;
use crate::backend::{base_type::function::FSRFn, vm::vm::FSRVirtualMachine};
use crate::backend::base_type::utils::i_to_m;
use crate::utils::error::FSRRuntimeError;

use super::{base::{FSRClass, FSRObject, FSRValue, IFSRObject}, integer::FSRInteger};

#[derive(Debug)]
pub struct FSRString {
    value       : String
}

impl FSRString {
    fn register_len_func<'a>(vm: &'a FSRVirtualMachine, rt: &mut FSRRuntimeModule) -> Result<u64, FSRRuntimeError<'a>> {
        let s = rt.find_symbol("self", vm, None).unwrap();

        let self_obj = vm.get_obj_by_id(&s).unwrap();
        let _str = self_obj.get_string().unwrap();
        let len = FSRInteger::from_i64(_str.value.len() as i64, i_to_m(vm));
        return Ok(len.get_id());
    }

    fn register_to_string_func<'a>(vm: &'a FSRVirtualMachine, rt: &mut FSRRuntimeModule) -> Result<u64, FSRRuntimeError<'a>> {
        let s = rt.find_symbol("self", vm, None).unwrap();
        return Ok(s.clone());
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

    pub fn get_string(&self) -> &String {
        return &self.value;
    }
}

impl IFSRObject for FSRString {
    fn get_class_name() -> &'static str {
        "String"
    }

    fn get_class(vm: &FSRVirtualMachine) -> FSRClass {
        let mut cls = FSRClass::new("String");
        let fn_obj = FSRFn::from_func(FSRString::register_len_func, vm, vec!["self"]);
        cls.register_obj("len", fn_obj.get_id());
        let fn_obj = FSRFn::from_func(FSRString::register_to_string_func, vm, vec!["self"]);
        cls.register_obj("to_string", fn_obj.get_id());
        return cls;
    }
    
    fn init(&mut self) {
        todo!()
    }
}

