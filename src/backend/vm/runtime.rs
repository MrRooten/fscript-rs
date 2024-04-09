use std::{cell::{Cell, RefCell}, collections::HashMap, sync::atomic::{AtomicU64, Ordering}};

use crate::backend::types::base::{FSRObject, FSRValue};

use super::thread::FSRThreadRuntime;

pub struct FSRVM<'a> {
    threads         : HashMap<u64, FSRThreadRuntime>,
    update_id       : AtomicU64,
    obj_map         : HashMap<u64, RefCell<FSRObject<'a>>>
}

impl<'a> FSRVM<'a> {
    pub fn new() -> Self {
        let main_thread = FSRThreadRuntime::new();
        let mut maps = HashMap::new();
        maps.insert(0, main_thread);
        let v = Self {
            threads: maps,
            update_id: AtomicU64::new(1000),
            obj_map: HashMap::new(),
        };
        v
    }

    pub fn new_object(&mut self) -> &RefCell<FSRObject<'a>> {
        let id = self.update_id.fetch_add(1, Ordering::Relaxed);
        let obj = FSRObject {
            obj_id: id.clone(),
            value: FSRValue::None
        };
        self.obj_map.insert(obj.obj_id, RefCell::new(obj));
        return self.obj_map.get(&id).unwrap();
    }

    pub fn get_obj_by_id(&self, id: &u64) -> Option<&RefCell<FSRObject<'a>>> {
        return self.obj_map.get(id)
    }

    pub fn register_object(&mut self, object: FSRObject<'a>) -> u64 {
        let id = self.update_id.fetch_add(1, Ordering::Relaxed);
        self.obj_map.insert(id, RefCell::new(object));
        return id; 
    }
}