use std::{collections::HashMap, fmt::Debug, ptr::addr_of, sync::atomic::AtomicUsize};

use ahash::AHashMap;

use crate::backend::vm::virtual_machine::get_object_by_global_id;

use super::{base::{AtomicObjId, FSRGlobalObjId, FSRObject, FSRValue, ObjId}, class::FSRClass};


pub struct FSRModule<'a> {
    name: String,
    fn_map: HashMap<String, FSRObject<'a>>,
    object_map: AHashMap<String, AtomicObjId>,
    const_table: Vec<Option<ObjId>>,
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
            object_map: AHashMap::new(),
            const_table: vec![],
        };
        let mut object = FSRObject::new();
        object.value = FSRValue::Module(Box::new(module));
        object.cls = get_object_by_global_id(FSRGlobalObjId::CodeCls) as ObjId;
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

    pub fn register_object(&mut self, name: &'a str, obj_id: ObjId) {
        self.object_map
            .insert(name.to_string(), AtomicObjId::new(obj_id));
    }

    pub fn get_object(&self, name: &str) -> Option<&AtomicObjId> {
        self.object_map.get(name)
    }

    pub fn insert_const(&mut self, const_index: usize, obj: ObjId) {
        if const_index >= self.const_table.len() {
            self.const_table.resize(const_index + 1, None);
        }
        self.const_table[const_index] = Some(obj);
    }

    pub fn get_const(&self, const_index: usize) -> Option<ObjId> {
        if const_index < self.const_table.len() {
            self.const_table[const_index]
        } else {
            None
        }
    }
}