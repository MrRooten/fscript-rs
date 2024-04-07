use crate::{
    backend::{
        base_type::base::FSRValue,
        vm::{runtime::FSRThreadRuntime, vm::FSRVirtualMachine},
    },
    utils::error::FSRRuntimeError,
};

use super::{
    base::{FSRBaseType, FSRObject, IFSRObject},
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
        unimplemented!()
    }

    fn register_to_index_func<'a>(
        vm: &'a FSRVirtualMachine<'a>,
        rt: &'a mut FSRThreadRuntime<'a>,
    ) -> Result<u64, FSRRuntimeError<'a>> {
        unimplemented!()
    
    }
}

impl IFSRObject for FSRList {
    fn init(&mut self) {}

    fn get_class_name() -> &'static str {
        "List"
    }

    fn get_class(vm: &FSRVirtualMachine) -> FSRBaseType {
        let mut cls = FSRBaseType::new("List");
        let fn_obj = FSRFn::from_func(FSRList::register_to_string_func, vm, vec!["self", "other"]);
        cls.register_obj("__str__", fn_obj.get_id());

        let fn_obj = FSRFn::from_func(FSRList::register_to_index_func, vm, vec!["self", "index"]);
        cls.register_obj("index", fn_obj.get_id());
        return cls;
    }
}
