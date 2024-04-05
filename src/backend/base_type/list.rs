use crate::{
    backend::{
        base_type::base::FSRValue,
        vm::{runtime::FSRThreadRuntime, vm::FSRVirtualMachine},
    },
    utils::error::FSRRuntimeError,
};

use super::{
    base::{FSRClass, FSRObject, IFSRObject},
    function::FSRFn,
    string::FSRString,
};

#[derive(Debug)]
pub struct FSRList {
    value: Vec<u64>,
}

impl FSRList {
    pub fn from_list<'a>(
        list: Vec<u64>,
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<u64, FSRRuntimeError> {
        let list_obj = FSRObject::new(vm);
        let v = Self { value: list };
        list_obj.set_cls(vm.get_cls("List").unwrap());
        list_obj.set_value(FSRValue::List(v));
        return Ok(list_obj.get_id());
    }

    fn register_to_string_func<'a>(
        vm: &'a FSRVirtualMachine<'a>,
        rt: &'a mut FSRThreadRuntime<'a>,
    ) -> Result<u64, FSRRuntimeError<'a>> {
        let s = rt.find_symbol("self", vm, None).unwrap();
        let self_obj = vm.get_obj_by_id(&s).unwrap();
        let mut s = String::new();
        s.push('[');
        if let FSRValue::List(l) = self_obj.get_value() {
            for id in &l.value {
                let obj = vm.get_obj_by_id(&id).unwrap();
                let str_object = obj.invoke_method("to_string", vm, rt)?;
                let obj = vm.get_obj_by_id(&str_object).unwrap();
                if let FSRValue::String(_s) = obj.get_value() {
                    s.push_str(_s.get_string());
                    s.push(',');
                }
            }
            if l.value.len() > 1 {
                s.pop();
            }
        }
        s.push(']');

        let obj = FSRString::from(s, vm);
        return Ok(obj.get_id());
    }

    fn register_to_index_func<'a>(
        vm: &'a FSRVirtualMachine<'a>,
        rt: &'a mut FSRThreadRuntime<'a>,
    ) -> Result<u64, FSRRuntimeError<'a>> {
        let s = rt.find_symbol("self", vm, None).unwrap();
        let self_obj = vm.get_obj_by_id(&s).unwrap();
        let index = rt.find_symbol("index", vm, None).unwrap();
        let index_obj = vm.get_obj_by_id(&index).unwrap();
        let integer = index_obj.get_integer().unwrap();
        if let FSRValue::List(l) = self_obj.get_value() {
            return Ok(l.value[integer.get_value() as usize]);
        }
        
        unimplemented!()
    }
}

impl IFSRObject for FSRList {
    fn init(&mut self) {}

    fn get_class_name() -> &'static str {
        "List"
    }

    fn get_class(vm: &FSRVirtualMachine) -> FSRClass {
        let mut cls = FSRClass::new("List");
        let fn_obj = FSRFn::from_func(FSRList::register_to_string_func, vm, vec!["self", "other"]);
        cls.register_obj("to_string", fn_obj.get_id());

        let fn_obj = FSRFn::from_func(FSRList::register_to_index_func, vm, vec!["self", "index"]);
        cls.register_obj("index", fn_obj.get_id());
        return cls;
    }
}
