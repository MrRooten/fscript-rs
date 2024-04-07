use crate::{backend::{base_type::base::FSRValue, vm::{runtime::FSRThreadRuntime, vm::FSRVirtualMachine}}, utils::error::FSRRuntimeError};

use super::{base::{FSRBaseType, FSRObject, IFSRObject}, function::FSRFn, string::FSRString, utils::i_to_m};


pub struct FSRIntegerAttrs {

}


#[derive(Debug)]
pub struct FSRInteger {
    value       : i64,
}



impl FSRInteger {
    pub fn get_value(&self) -> i64 {
        return self.value;
    }

    fn register_add_func<'a>(vm: &'a FSRVirtualMachine, rt: &mut FSRThreadRuntime) -> Result<u64, FSRRuntimeError<'a>> {
        unimplemented!()
    }

    fn register_self_add_func<'a>(vm: &'a FSRVirtualMachine, rt: &mut FSRThreadRuntime) -> Result<u64, FSRRuntimeError<'a>> {
        unimplemented!()
    }

    fn register_equal_func<'a>(vm: &'a FSRVirtualMachine, rt: &mut FSRThreadRuntime) -> Result<u64, FSRRuntimeError<'a>> {
        unimplemented!()
    }

    fn register_not_equal_func<'a>(vm: &'a FSRVirtualMachine, rt: &mut FSRThreadRuntime) -> Result<u64, FSRRuntimeError<'a>> {
        unimplemented!()
    }

    fn register_greater_func<'a>(vm: &'a FSRVirtualMachine, rt: &mut FSRThreadRuntime) -> Result<u64, FSRRuntimeError<'a>> {
        unimplemented!()
    }

    fn register_less_func<'a>(vm: &'a FSRVirtualMachine, rt: &mut FSRThreadRuntime) -> Result<u64, FSRRuntimeError<'a>> {
        unimplemented!()
    }

    fn register_greater_equal_func<'a>(vm: &'a FSRVirtualMachine, rt: &mut FSRThreadRuntime) -> Result<u64, FSRRuntimeError<'a>> {
        unimplemented!()
    }

    fn register_less_equal_func<'a>(vm: &'a FSRVirtualMachine, rt: &mut FSRThreadRuntime) -> Result<u64, FSRRuntimeError<'a>> {
        unimplemented!()
    }


    fn register_sub_func<'a>(vm: &'a FSRVirtualMachine, rt: &mut FSRThreadRuntime) -> Result<u64, FSRRuntimeError<'a>> {
        unimplemented!()
    }

    fn register_mul_func<'a>(vm: &'a FSRVirtualMachine, rt: &mut FSRThreadRuntime) -> Result<u64, FSRRuntimeError<'a>> {
        unimplemented!()
    }


    fn register_to_string_func<'a>(vm: &'a FSRVirtualMachine, rt: &mut FSRThreadRuntime) -> Result<u64, FSRRuntimeError<'a>> {
        unimplemented!()
    }

    pub fn from_integer<'a>(integer: FSRInteger, vm: &'a FSRVirtualMachine<'a>) -> &FSRObject<'a> {
        let obj = FSRObject::new(vm);
        obj.set_cls(vm.get_cls("Integer").unwrap());
        let value = FSRValue::Integer(integer);
        obj.set_value(value);
        
        return obj;
    }

    pub fn from_i64<'a>(integer: i64, vm: &'a FSRVirtualMachine<'a>) -> &FSRObject<'a> {
        let v: FSRInteger = FSRInteger {
            value: integer,
        };

        let obj = FSRObject::new(vm);
        obj.set_cls(vm.get_cls("Integer").unwrap());
        let value = FSRValue::Integer(v);
        obj.set_value(value);

        return obj;
    }

    pub fn get_integer<T>(&self) -> T {
        unimplemented!()
    }

    pub fn add<'a>(&self, other: &FSRInteger) -> FSRInteger {
        let ret = FSRInteger {
            value: self.value + other.value,
        };
        
        ret
    }

    pub fn sub(&self, other: &FSRInteger) -> FSRInteger {
        let ret = FSRInteger {
            value: self.value - other.value,
        };
        
        ret
    }

    pub fn mul(&self, other: &FSRInteger) -> FSRInteger {
        let ret = FSRInteger {
            value: self.value * other.value,
        };
        
        ret 
    }

    pub fn div(&self, _: &FSRInteger) -> FSRInteger {
        unimplemented!()
    }
}


impl IFSRObject for FSRInteger {
    
    fn get_class_name() -> &'static str {
        "Integer"
    }
    
    fn get_class(vm: &FSRVirtualMachine) -> FSRBaseType {
        let mut cls = FSRBaseType::new("Integer");
        let fn_obj = FSRFn::from_func(FSRInteger::register_add_func, vm, vec!["self", "other"]);
        cls.register_obj("__add__", fn_obj.get_id());
        let fn_obj = FSRFn::from_func(FSRInteger::register_sub_func, vm, vec!["self", "other"]);
        cls.register_obj("__sub__", fn_obj.get_id());
        let fn_obj = FSRFn::from_func(FSRInteger::register_mul_func, vm, vec!["self", "other"]);
        cls.register_obj("__mul__", fn_obj.get_id());
        let fn_obj = FSRFn::from_func(FSRInteger::register_equal_func, vm, vec!["self", "other"]);
        cls.register_obj("__eq__", fn_obj.get_id());
        let fn_obj = FSRFn::from_func(FSRInteger::register_not_equal_func, vm, vec!["self", "other"]);
        cls.register_obj("__not_eq__", fn_obj.get_id());
        let fn_obj = FSRFn::from_func(FSRInteger::register_to_string_func, vm, vec!["self"]);
        cls.register_obj("__str__", fn_obj.get_id());
        let fn_obj = FSRFn::from_func(FSRInteger::register_greater_func, vm, vec!["self", "other"]);
        cls.register_obj("__gt__", fn_obj.get_id());
        let fn_obj = FSRFn::from_func(FSRInteger::register_greater_equal_func, vm, vec!["self", "other"]);
        cls.register_obj("__gte__", fn_obj.get_id());
        let fn_obj = FSRFn::from_func(FSRInteger::register_less_func, vm, vec!["self", "other"]);
        cls.register_obj("__lt__", fn_obj.get_id());
        let fn_obj = FSRFn::from_func(FSRInteger::register_less_equal_func, vm, vec!["self", "other"]);
        cls.register_obj("__lte__", fn_obj.get_id());
        let fn_obj = FSRFn::from_func(FSRInteger::register_self_add_func, vm, vec!["self", "other"]);
        cls.register_obj("__self_add__", fn_obj.get_id());
        return cls;
    }
    
    fn init(&mut self) {
        todo!()
    }


}