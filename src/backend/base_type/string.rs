use std::{fmt::Error, collections::HashMap};

use crate::backend::base_type::function::FSRFunction;

use super::{base::{FSRClassRegister, FSRObject, FSRObjectManager, FSRClass, FSRValue}, integer::FSRInteger};

pub struct FSRString {
    value       : String
}

const STRING_CLASS: FSRClass<'_> = FSRClass::new("String");

impl FSRString {
    fn register_len_func(args: &HashMap<&str, u64>, manager: &FSRObjectManager) -> Result<FSRObject<'static>, Error> {
        let s = args.get("self").unwrap();
        let self_obj = manager.get_obj_by_id(s).unwrap();
        let i = self_obj.get_string().unwrap();
        let len = i.value.len();
        Ok(FSRInteger::from_usize(len))
    }

    fn register_find_func(args: &HashMap<&str, u64>, manager: &FSRObjectManager) -> Result<FSRObject<'static>, Error> {
        
        unimplemented!()
    }

    pub fn from<T>(s: T) -> FSRObject<'static>
    where T: ToString {
        let v = FSRString {
            value: s.to_string()
        };

        let mut obj = FSRObject::new();
        obj.set_cls(&STRING_CLASS);
        obj.set_value(FSRValue::String(v));
        return obj;
    }
}

impl FSRClassRegister for FSRString {
    fn get_class_name() -> &'static str {
        "String"
    }

    fn register_attrs(manager: &mut super::base::FSRObjectManager) {
        todo!()
    }

    fn get_attrs() -> super::base::FSRAttrs<'static> {
        let len_method = FSRFunction::from_func(FSRString::register_len_func);
        
        let mut attrs = HashMap::new();
        attrs.insert("len", len_method);
        attrs.insert("find", FSRFunction::from_func(FSRString::register_find_func));
        return attrs;
    }

    fn get_cls_name(&self) -> &'static str {
        FSRString::get_class_name()
    }
}