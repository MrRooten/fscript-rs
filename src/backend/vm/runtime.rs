use std::{collections::HashMap, sync::atomic::{AtomicU64, Ordering}};

use crate::backend::types::base::{FSRObject, FSRValue};

use super::thread::FSRThreadRuntime;

pub struct FSRVM<'a> {
    threads         : HashMap<u64, FSRThreadRuntime>,
    update_id       : AtomicU64,
    obj_map         : HashMap<u64, FSRObject<'a>>
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

    pub fn new_object(&mut self) -> &mut FSRObject<'a> {
        let id = self.update_id.fetch_add(1, Ordering::Relaxed);
        let obj = FSRObject {
            obj_id: id.clone(),
            value: FSRValue::None
        };
        self.obj_map.insert(obj.obj_id, obj);
        return self.obj_map.get_mut(&id).unwrap();
    }
}