use crate::{backend::{base_type::base::FSRValue, vm::{module::FSRRuntimeModule, vm::FSRVirtualMachine}}, utils::error::FSRRuntimeError};

use super::{base::{FSRClass, FSRObject, IFSRObject}, function::FSRFn, string::FSRString};


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

    fn register_add_func<'a>(vm: &'a FSRVirtualMachine, rt: &mut FSRRuntimeModule) -> Result<u64, FSRRuntimeError<'a>> {
        let s = rt.find_symbol("self", vm, None).unwrap();
        let id = rt.find_symbol("other", vm, None).unwrap();
        let self_obj = vm.get_obj_by_id(&s).unwrap();
        let obj = vm.get_obj_by_id(&id).unwrap();
        let self_i = self_obj.get_integer().unwrap();
        let i = obj.get_integer().unwrap();
        let result = self_i.add(i);
        let obj = FSRInteger::from_integer(result, vm);
        return Ok(obj.get_id());
    }

    fn register_equal_func<'a>(vm: &'a FSRVirtualMachine, rt: &mut FSRRuntimeModule) -> Result<u64, FSRRuntimeError<'a>> {
        let s = rt.find_symbol("self", vm, None).unwrap();
        let id = rt.find_symbol("other", vm, None).unwrap();
        if s == id {
            return Ok(vm.get_true_id());
        }
        let self_obj = vm.get_obj_by_id(&s).unwrap();
        let obj = vm.get_obj_by_id(&id).unwrap();
        let self_i = self_obj.get_integer().unwrap();
        let i = obj.get_integer().unwrap();
        if self_i.value == i.value {
            return Ok(vm.get_true_id());
        } else {
            return Ok(vm.get_false_id());
        }
    }

    fn register_greater_func<'a>(vm: &'a FSRVirtualMachine, rt: &mut FSRRuntimeModule) -> Result<u64, FSRRuntimeError<'a>> {
        let s = rt.find_symbol("self", vm, None).unwrap();
        let id = rt.find_symbol("other", vm, None).unwrap();

        let self_obj = vm.get_obj_by_id(&s).unwrap();
        let obj = vm.get_obj_by_id(&id).unwrap();
        let self_i = self_obj.get_integer().unwrap();
        let i = obj.get_integer().unwrap();
        if self_i.value > i.value {
            return Ok(vm.get_true_id());
        } else {
            return Ok(vm.get_false_id());
        }
    }

    fn register_less_func<'a>(vm: &'a FSRVirtualMachine, rt: &mut FSRRuntimeModule) -> Result<u64, FSRRuntimeError<'a>> {
        let s = rt.find_symbol("self", vm, None).unwrap();
        let id = rt.find_symbol("other", vm, None).unwrap();

        let self_obj = vm.get_obj_by_id(&s).unwrap();
        let obj = vm.get_obj_by_id(&id).unwrap();
        let self_i = self_obj.get_integer().unwrap();
        let i = obj.get_integer().unwrap();
        if self_i.value < i.value {
            return Ok(vm.get_true_id());
        } else {
            return Ok(vm.get_false_id());
        }
    }

    fn register_greater_equal_func<'a>(vm: &'a FSRVirtualMachine, rt: &mut FSRRuntimeModule) -> Result<u64, FSRRuntimeError<'a>> {
        let s = rt.find_symbol("self", vm, None).unwrap();
        let id = rt.find_symbol("other", vm, None).unwrap();

        let self_obj = vm.get_obj_by_id(&s).unwrap();
        let obj = vm.get_obj_by_id(&id).unwrap();
        let self_i = self_obj.get_integer().unwrap();
        let i = obj.get_integer().unwrap();
        if self_i.value >= i.value {
            return Ok(vm.get_true_id());
        } else {
            return Ok(vm.get_false_id());
        }
    }

    fn register_less_equal_func<'a>(vm: &'a FSRVirtualMachine, rt: &mut FSRRuntimeModule) -> Result<u64, FSRRuntimeError<'a>> {
        let s = rt.find_symbol("self", vm, None).unwrap();
        let id = rt.find_symbol("other", vm, None).unwrap();

        let self_obj = vm.get_obj_by_id(&s).unwrap();
        let obj = vm.get_obj_by_id(&id).unwrap();
        let self_i = self_obj.get_integer().unwrap();
        let i = obj.get_integer().unwrap();
        if self_i.value <= i.value {
            return Ok(vm.get_true_id());
        } else {
            return Ok(vm.get_false_id());
        }
    }


    fn register_sub_func<'a>(vm: &'a FSRVirtualMachine, rt: &mut FSRRuntimeModule) -> Result<u64, FSRRuntimeError<'a>> {
        let s = rt.find_symbol("self", vm, None).unwrap();
        let id = rt.find_symbol("other", vm, None).unwrap();
        let self_obj = vm.get_obj_by_id(&s).unwrap();
        let obj = vm.get_obj_by_id(&id).unwrap();
        let self_i = self_obj.get_integer().unwrap();
        let i = obj.get_integer().unwrap();
        let result = self_i.sub(i);
        let obj = FSRInteger::from_integer(result, vm);
        return Ok(obj.get_id());
    }

    fn register_mul_func<'a>(vm: &'a FSRVirtualMachine, rt: &mut FSRRuntimeModule) -> Result<u64, FSRRuntimeError<'a>> {
        let s = rt.find_symbol("self", vm, None).unwrap();
        let id = rt.find_symbol("other", vm, None).unwrap();
        let self_obj = vm.get_obj_by_id(&s).unwrap();
        let obj = vm.get_obj_by_id(&id).unwrap();
        let self_i = self_obj.get_integer().unwrap();
        let i = obj.get_integer().unwrap();
        let result = self_i.mul(i);
        let obj = FSRInteger::from_integer(result, vm);
        return Ok(obj.get_id());
    }

    fn register_to_string_func<'a>(vm: &'a FSRVirtualMachine, rt: &mut FSRRuntimeModule) -> Result<u64, FSRRuntimeError<'a>> {
        let s = rt.find_symbol("self", vm, None).unwrap();
        let self_obj = vm.get_obj_by_id(&s).unwrap();

        let v = self_obj.get_integer().unwrap();
        let integer = v.get_value();
        let obj = FSRString::from(integer, vm);
        return Ok(obj.get_id());
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
    
    fn get_class(vm: &FSRVirtualMachine) -> FSRClass {
        let mut cls = FSRClass::new("Integer");
        let fn_obj = FSRFn::from_func(FSRInteger::register_add_func, vm, vec!["self", "other"]);
        cls.register_obj("add", fn_obj.get_id());
        let fn_obj = FSRFn::from_func(FSRInteger::register_sub_func, vm, vec!["self", "other"]);
        cls.register_obj("sub", fn_obj.get_id());
        let fn_obj = FSRFn::from_func(FSRInteger::register_mul_func, vm, vec!["self", "other"]);
        cls.register_obj("mul", fn_obj.get_id());
        let fn_obj = FSRFn::from_func(FSRInteger::register_equal_func, vm, vec!["self", "other"]);
        cls.register_obj("eq", fn_obj.get_id());
        let fn_obj = FSRFn::from_func(FSRInteger::register_to_string_func, vm, vec!["self"]);
        cls.register_obj("to_string", fn_obj.get_id());
        let fn_obj = FSRFn::from_func(FSRInteger::register_greater_func, vm, vec!["self", "other"]);
        cls.register_obj("greater", fn_obj.get_id());
        let fn_obj = FSRFn::from_func(FSRInteger::register_greater_equal_func, vm, vec!["self", "other"]);
        cls.register_obj("greater_equal", fn_obj.get_id());
        let fn_obj = FSRFn::from_func(FSRInteger::register_less_func, vm, vec!["self", "other"]);
        cls.register_obj("less", fn_obj.get_id());
        let fn_obj = FSRFn::from_func(FSRInteger::register_less_equal_func, vm, vec!["self", "other"]);
        cls.register_obj("less_equal", fn_obj.get_id());
        return cls;
    }
    
    fn init(&mut self) {
        todo!()
    }


}