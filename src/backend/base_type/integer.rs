use std::{fmt::Error, collections::HashMap};

use crate::backend::base_type::base::FSRValue;

use super::{base::{FSRObject, FSRObjectManager, FSRClassRegister, FSRAttrs, FSRClass}, function::FSRFunction};

enum _FSRInteger {
    Unsigned32(u32),
    Unsigned64(u64),
    Signed32(i32),
    Signed64(i64),
    Float32(f32),
    Float64(f64),
    Usize(usize)
}

pub struct FSRIntegerAttrs {

}


pub struct FSRInteger {
    value       : _FSRInteger,
}


const INTEGER_CLASS: FSRClass<'_> = FSRClass::new("Integer");
impl FSRInteger {
    fn register_add_func(args: &HashMap<&str, u64>, manager: &FSRObjectManager) -> Result<FSRObject<'static>, Error> {
        let s = args.get("self").unwrap();
        let id = args.get("other").unwrap();
        let self_obj = manager.get_obj_by_id(s).unwrap();
        let obj = manager.get_obj_by_id(id).unwrap();
        let self_i = self_obj.get_integer().unwrap();
        let i = obj.get_integer().unwrap();
        let result = self_i.add(i);

        return Ok(FSRInteger::from_integer(result));
    }

    pub fn cls_register(manager: &'static mut FSRObjectManager) {
        let func_obj = FSRFunction::from_func(FSRInteger::register_add_func);
        manager.register_object(func_obj);
    }

    pub fn register(&mut self, obj: &mut FSRObject, manager: &mut FSRObjectManager) {
        
    }

    pub fn from_integer(integer: FSRInteger) -> FSRObject<'static> {
        let mut obj = FSRObject::new();
        obj.set_cls(&INTEGER_CLASS);
        let value = FSRValue::Integer(integer);
        obj.set_value(value);
        
        return obj;
    }

    pub fn from_u32(integer: u32) -> FSRObject<'static> {
        let v = FSRInteger {
            value: _FSRInteger::Unsigned32(integer),
        };

        let mut obj = FSRObject::new();
        obj.set_cls(&INTEGER_CLASS);
        let value = FSRValue::Integer(v);
        obj.set_value(value);
        
        return obj;
    }

    pub fn from_usize(integer: usize) -> FSRObject<'static> {
        let v = FSRInteger {
            value: _FSRInteger::Usize(integer),
        };

        let mut obj = FSRObject::new();
        obj.set_cls(&INTEGER_CLASS);
        let value = FSRValue::Integer(v);
        obj.set_value(value);
        
        return obj;
    }

    pub fn get_integer<T>(&self) -> T {
        unimplemented!()
    }

    pub fn add(&self, other: &FSRInteger) -> FSRInteger {
        if let _FSRInteger::Unsigned32(u) = other.value {
            if let _FSRInteger::Unsigned32(v) = self.value {
                return FSRInteger {
                    // 可能溢出的错误处理
                    value: _FSRInteger::Unsigned32(u + v)
                };
            }
        }

        if let _FSRInteger::Usize(u) = other.value {
            if let _FSRInteger::Usize(v) = self.value {
                return FSRInteger {
                    // 可能溢出的错误处理
                    value: _FSRInteger::Usize(u + v)
                };
            }
        }
        unimplemented!()
    }

    pub fn sub(&self, other: &FSRInteger) -> FSRInteger {
        unimplemented!()
    }

    pub fn mul(&self, other: &FSRInteger) -> FSRInteger {
        unimplemented!()   
    }

    pub fn div(&self, other: &FSRInteger) -> FSRInteger {
        unimplemented!()
    }
}



impl FSRClassRegister for FSRInteger {
    fn get_class_name() -> &'static str {
        "Integer"
    }

    fn get_attrs() -> FSRAttrs<'static> {
        let func_obj = FSRFunction::from_func(FSRInteger::register_add_func);
        
        let mut attrs = HashMap::new();
        attrs.insert("add", func_obj);
        return attrs;
    }

    fn register_attrs(manager: &mut FSRObjectManager) {
        
    }

    fn get_cls_name(&self) -> &'static str {
        return FSRInteger::get_class_name();
    }
}