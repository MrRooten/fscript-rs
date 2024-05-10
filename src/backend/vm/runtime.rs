use std::{collections::HashMap, sync::atomic::AtomicU64};

use crate::{
    backend::types::{
        base::{FSRObject, FSRValue}, class::FSRClass, integer::FSRInteger, list::FSRList, string::FSRString
    },
    std::io::init_io,
};

use super::thread::FSRThreadRuntime;

pub struct FSRVM<'a> {
    #[allow(unused)]
    threads: HashMap<u64, FSRThreadRuntime<'a>>,
    obj_map: HashMap<u64, Box<FSRObject<'a>>>,
    global: HashMap<String, u64>,
    base_types: HashMap<&'a str, FSRClass<'a>>,
}

pub static mut NONE_OBJECT: Option<FSRObject> = None;
pub static mut TRUE_OBJECT: Option<FSRObject> = None;
pub static mut FALSE_OBJECT: Option<FSRObject> = None;

impl<'a> Default for FSRVM<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> FSRVM<'a> {
    pub fn new() -> Self {
        Self::init_static_object();
        let main_thread = FSRThreadRuntime::new();
        let mut maps = HashMap::new();
        maps.insert(0, main_thread);
        let mut v = Self {
            threads: maps,
            obj_map: HashMap::new(),
            base_types: HashMap::new(),
            global: HashMap::new(),
        };
        v.init();
        v
    }

    pub fn check_delete(&mut self, id: u64) {
        let obj = FSRObject::id_to_mut_obj(id);
        if *obj.ref_count.get_mut() == 0 {
            self.obj_map.remove(&id);
        }
    }

    pub fn get_true_id(&self) -> u64 {
        1
    }

    pub fn get_false_id(&self) -> u64 {
        2
    }

    pub fn get_none_id(&self) -> u64 {
        0
    }

    pub fn init_static_object() {
        unsafe {
            if NONE_OBJECT.is_none() {
                NONE_OBJECT = Some(Self::new_stataic_object_with_id(0, FSRValue::None));
            }

            if TRUE_OBJECT.is_none() {
                TRUE_OBJECT = Some(Self::new_stataic_object_with_id(1, FSRValue::Bool(true)));
            }

            if FALSE_OBJECT.is_none() {
                FALSE_OBJECT = Some(Self::new_stataic_object_with_id(2, FSRValue::Bool(false)));
            }
        }
    }

    pub fn init(&mut self) {
        // Set none variable as uniq object id 0
        self.global.insert("none".to_string(), 0);
        // Set true variable as uniq object id 1
        self.global.insert("true".to_string(), 1);
        // Set false variable as uniq object id 2
        self.global.insert("false".to_string(), 2);

        let integer = FSRInteger::get_class(self);
        self.base_types.insert("Integer", integer);

        let string = FSRString::get_class(self);
        self.base_types.insert("String", string);

        let list = FSRList::get_class(self);
        self.base_types.insert("List", list);

        let objs = init_io();
        for obj in objs {
            let id = self.register_object(obj.1);
            self.global.insert(obj.0.to_string(), id);
        }
    }

    pub fn get_cls(&self, name: &str) -> Option<&FSRClass<'a>> {
        return self.base_types.get(name);
    }

    fn new_stataic_object_with_id(id: u64, value: FSRValue<'static>) -> FSRObject<'static> {
        FSRObject {
            obj_id: id,
            value,
            cls: "",
            ref_count: AtomicU64::new(0),
        }
    }

    pub fn get_obj_by_id(&self, id: &u64) -> Option<&FSRObject<'a>> {
        match self.obj_map.get(id) {
            Some(s) => Some(s),
            None => None,
        }
    }

    pub fn register_global_object(&mut self, name: &str, obj_id: u64) {
        self.global.insert(name.to_string(), obj_id);
    }

    fn get_object_id(obj: &FSRObject) -> u64 {
        obj as *const FSRObject as u64
    }

    pub fn register_object(&mut self, object: FSRObject<'a>) -> u64 {
        let mut object = Box::new(object);
        let id = Self::get_object_id(&object);
        object.obj_id = id;

        self.obj_map.insert(id, object);

        id
    }

    pub fn get_mut_obj_by_id(&mut self, id: &u64) -> Option<&mut Box<FSRObject<'a>>> {
        return self.obj_map.get_mut(id);
    }

    pub fn get_global_obj_by_name(&self, name: &str) -> Option<&u64> {
        return self.global.get(name);
    }
}
