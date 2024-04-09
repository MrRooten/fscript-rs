use std::{cell::{Cell, RefCell}, collections::HashMap, sync::atomic::{AtomicU64, Ordering}};

use crate::{backend::types::{base::{FSRObject, FSRValue}, class::FSRClass}, std::io::init_io};

use super::thread::FSRThreadRuntime;

pub struct FSRVM<'a> {
    threads         : HashMap<u64, FSRThreadRuntime>,
    update_id       : AtomicU64,
    obj_map         : HashMap<u64, RefCell<FSRObject<'a>>>,
    global          : HashMap<&'a str, u64>,
    base_types      : HashMap<&'a str, FSRClass<'a>>
}

impl<'a> FSRVM<'a> {
    pub fn new() -> Self {
        let main_thread = FSRThreadRuntime::new();
        let mut maps = HashMap::new();
        maps.insert(0, main_thread);
        let mut v = Self {
            threads: maps,
            update_id: AtomicU64::new(1000),
            obj_map: HashMap::new(),
            base_types: HashMap::new(),
            global: HashMap::new(),
        };
        v.init();
        v
    }

    pub fn init(&mut self) {
        let objs = init_io();
        for obj in objs {
            let id = self.register_object(obj.1);
            self.global.insert(obj.0, id);
        }
    }

    pub fn get_cls(&self, name: &str) -> Option<&FSRClass<'a>> {
        return self.base_types.get(name)
    }

    pub fn new_object(&mut self) -> &RefCell<FSRObject<'a>> {
        let id = self.update_id.fetch_add(1, Ordering::Relaxed);
        let obj = FSRObject {
            obj_id: id.clone(),
            value: FSRValue::None,
            cls: "",
            attrs: HashMap::new(),
        };
        self.obj_map.insert(obj.obj_id, RefCell::new(obj));
        return self.obj_map.get(&id).unwrap();
    }

    pub fn get_obj_by_id(&self, id: &u64) -> Option<&RefCell<FSRObject<'a>>> {
        return self.obj_map.get(id)
    }

    pub fn register_object(&mut self, mut object: FSRObject<'a>) -> u64 {
        let id = self.update_id.fetch_add(1, Ordering::Relaxed);
        object.obj_id = id;
        self.obj_map.insert(id, RefCell::new(object));
        
        return id; 
    }

    pub fn get_mut_obj_by_id(&mut self, id: &u64) -> Option<&mut RefCell<FSRObject<'a>>> {
        return self.obj_map.get_mut(id);
    }

    pub fn get_global_obj_by_name(&self, name: &str) -> Option<&u64> {
        return self.global.get(name)
    }


}