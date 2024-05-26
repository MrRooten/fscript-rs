use std::{cell::RefCell, collections::HashMap, sync::atomic::AtomicU64};

use crate::{
    backend::types::{
        base::{FSRObject, FSRValue}, class::FSRClass, integer::FSRInteger, iterator::FSRInnerIterator, list::FSRList, module::FSRModule, string::FSRString
    },
    std::io::init_io,
};

use super::thread::FSRThreadRuntime;

pub struct FSRVM<'a> {
    #[allow(unused)]
    threads: HashMap<u64, FSRThreadRuntime<'a>>,
    global: HashMap<String, u64>,
    base_types: HashMap<&'a str, FSRClass<'a>>,
    global_modules  : HashMap<&'a str, FSRModule<'a>>,
    const_integer_global: RefCell<HashMap<i64, u64>>
}

// pub static mut NONE_OBJECT: Option<FSRObject> = None;
// pub static mut TRUE_OBJECT: Option<FSRObject> = None;
// pub static mut FALSE_OBJECT: Option<FSRObject> = None;
pub static mut OBJECTS: Vec<FSRObject> = vec![];
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
            base_types: HashMap::new(),
            global: HashMap::new(),
            global_modules: HashMap::new(),
            const_integer_global: RefCell::new(HashMap::new()),
        };
        v.init();
        v
    }

    pub fn get_integer(&self, integer: i64) -> u64 {
        let mut const_obj = self.const_integer_global.borrow_mut();
        const_obj.entry(integer).or_insert_with(|| {
            let obj = FSRObject {
                obj_id: 0,
                value: FSRValue::Integer(integer),
                cls: "Integer",
                ref_count: AtomicU64::new(1),
            };

            self.register_object(obj)
        });

        *const_obj.get(&integer).unwrap()
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
            if OBJECTS.is_empty() {
                OBJECTS.push(Self::new_stataic_object_with_id(0, FSRValue::None));
                OBJECTS.push(Self::new_stataic_object_with_id(1, FSRValue::Bool(true)));
                OBJECTS.push(Self::new_stataic_object_with_id(2, FSRValue::Bool(false)));
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

        let inner_iter = FSRInnerIterator::get_class(self);
        self.base_types.insert("InnerIterator", inner_iter);

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

    pub fn register_global_object(&mut self, name: &str, obj_id: u64) {
        self.global.insert(name.to_string(), obj_id);
    }

    fn get_object_id(obj: &FSRObject) -> u64 {
        obj as *const FSRObject as u64
    }

    pub fn register_object(&self, object: FSRObject<'a>) -> u64 {
        let mut object = Box::new(object);
        let id = Self::get_object_id(&object);
        object.obj_id = id;

        //self.obj_map.insert(id, object);
        Box::leak(object);
        id
    }

    pub fn get_global_obj_by_name(&self, name: &str) -> Option<&u64> {
        return self.global.get(name);
    }

    pub fn register_module(&mut self, name: &'a str, module: FSRModule<'a>) {
        self.global_modules.insert(name, module);
    }

    pub fn get_module(&self, name: &str) -> Option<&FSRModule<'a>> {
        self.global_modules.get(name)
    }
}
