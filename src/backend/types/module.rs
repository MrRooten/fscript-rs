use std::collections::HashMap;

use super::{base::{FSRGlobalObjId, FSRObject, FSRValue, ObjId}, class::FSRClass};

#[derive(Debug)]
pub struct FSRModule<'a> {
    name: String,
    fn_map: HashMap<String, FSRObject<'a>>,
}

impl<'a> FSRModule<'a> {
    pub fn as_string(&self) -> String {
        format!("Module: {}", self.name)
    }

    pub fn new_module(name: &str, fn_map: HashMap<String, FSRObject<'a>>) -> FSRObject<'a> {
        let module = FSRModule {
            name: name.to_string(),
            fn_map,
        };
        let mut object = FSRObject::new();
        object.value = FSRValue::Module(Box::new(module));
        object.cls = FSRGlobalObjId::CodeCls as ObjId;
        object
    }

    pub fn get_class() -> FSRClass<'static> {
        FSRClass::new("FSRModule")
    }

    pub fn get_fn(&self, name: &str) -> Option<&FSRObject<'a>> {
        self.fn_map.get(name)
    }

    pub fn iter_fn(&self) -> impl Iterator<Item = (&String, &FSRObject<'a>)> {
        self.fn_map.iter()
    }
}