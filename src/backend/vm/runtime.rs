use std::{cell::RefCell, collections::HashMap, sync::atomic::AtomicU64};

use crate::{
    backend::types::{
        base::{FSRGlobalObjId, FSRObject, FSRValue}, class::FSRClass, fn_def::FSRFn, integer::FSRInteger, iterator::FSRInnerIterator, list::FSRList, module::FSRModule, string::FSRString
    },
    std::io::init_io,
};

use super::thread::FSRThreadRuntime;

pub struct FSRVM<'a> {
    #[allow(unused)]
    threads: HashMap<u64, FSRThreadRuntime<'a>>,
    global: HashMap<String, u64>,
    global_modules  : HashMap<&'a str, FSRModule<'a>>,
    const_integer_global: RefCell<HashMap<i64, u64>>,
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
                cls: FSRGlobalObjId::IntegerCls as u64,
                ref_count: AtomicU64::new(1),
                delete_flag: RefCell::new(true),
            };

            FSRVM::register_object(obj)
        });

        *const_obj.get(&integer).unwrap()
    }

    #[inline(always)]
    pub fn get_true_id(&self) -> u64 {
        1
    }

    #[inline(always)]
    pub fn get_false_id(&self) -> u64 {
        2
    }

    #[inline(always)]
    pub fn get_none_id(&self) -> u64 {
        0
    }

    pub fn init_static_object() {
        unsafe {
            if OBJECTS.is_empty() {
                OBJECTS.push(Self::new_stataic_object_with_id(0, FSRValue::None));
                OBJECTS.push(Self::new_stataic_object_with_id(1, FSRValue::Bool(true)));
                OBJECTS.push(Self::new_stataic_object_with_id(2, FSRValue::Bool(false)));
                OBJECTS.push(Self::new_stataic_object_with_id(3, FSRValue::Class(FSRInteger::get_class())));
                OBJECTS.push(Self::new_stataic_object_with_id(4, FSRValue::Class(FSRFn::get_class())));
                OBJECTS.push(Self::new_stataic_object_with_id(5, FSRValue::Class(FSRInnerIterator::get_class())));
                OBJECTS.push(Self::new_stataic_object_with_id(6, FSRValue::Class(FSRList::get_class())));
                OBJECTS.push(Self::new_stataic_object_with_id(7, FSRValue::Class(FSRString::get_class())));
                OBJECTS.push(Self::new_stataic_object_with_id(8, FSRValue::Class(FSRClass::new("Class"))));
            }
        }
    }

    pub fn init(&mut self) {
        let objs = init_io();
        for obj in objs {
            let id = FSRVM::register_object(obj.1);
            self.global.insert(obj.0.to_string(), id);
        }
    }

    pub fn get_base_cls(&self, cls_id: u64) -> Option<&FSRClass<'a>> {
        unsafe {
            if let Some(s) = OBJECTS.get(cls_id as usize) {
                if let FSRValue::Class(c) = &s.value {
                    return Some(c);
                } else {
                    return None
                }
            }
        }
        None
    }

    fn new_stataic_object_with_id(id: u64, value: FSRValue<'static>) -> FSRObject<'static> {
        FSRObject {
            obj_id: id,
            value,
            cls: 0,
            ref_count: AtomicU64::new(0),
            delete_flag: RefCell::new(true),
        }
    }

    pub fn register_global_object(&mut self, name: &str, obj_id: u64) {
        self.global.insert(name.to_string(), obj_id);
    }

    fn get_object_id(obj: &FSRObject) -> u64 {
        obj as *const FSRObject as u64
    }

    pub fn leak_object(mut object: Box<FSRObject<'a>>) -> u64 {
        #[cfg(feature="alloc_trace")]
        crate::backend::types::base::HEAP_TRACE.add_object();
        let id = Self::get_object_id(&object);
        object.obj_id = id;

        //self.obj_map.insert(id, object);
        Box::leak(object);
        id
    }

    pub fn register_object(object: FSRObject<'a>) -> u64 {
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
