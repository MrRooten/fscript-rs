use std::{collections::HashMap, fmt::Debug, ptr::addr_of};

use super::{base::{FSRGlobalObjId, FSRObject, FSRValue, ObjId}, class::FSRClass};


pub struct FSRModule<'a> {
    name: String,
    fn_map: HashMap<String, FSRObject<'a>>,
}

impl Debug for FSRModule<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fn_map_debug = HashMap::new();
        for v in self.fn_map.iter() {
            let addr = addr_of!(self.fn_map) as usize;
            fn_map_debug.insert(v.0.as_str(), addr);
        }
        f.debug_struct("FSRModule")
            .field("name", &self.name)
            .field("fn_map", &fn_map_debug)
            .finish()
    }
}

impl<'a> FSRModule<'a> {
    pub fn as_string(&self) -> String {
        format!("Module: {}", self.name)
    }

    pub fn new_module(name: &str) -> FSRObject<'a> {
        let module = FSRModule {
            name: name.to_string(),
            fn_map: HashMap::new(),
        };
        let mut object = FSRObject::new();
        object.value = FSRValue::Module(Box::new(module));
        object.cls = FSRGlobalObjId::CodeCls as ObjId;
        object
    }

    pub fn init_fn_map(&mut self, fn_map: HashMap<String, FSRObject<'a>>) {
        self.fn_map = fn_map;
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