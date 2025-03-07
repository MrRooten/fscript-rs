use std::{
    alloc::GlobalAlloc,
    cell::{Cell, RefCell},
    collections::HashMap,
    sync::{atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering}, Mutex},
};

use ahash::AHashMap;

use crate::{
    backend::{
        memory::{gc::GarbageCollector, size_alloc::FSRObjectAllocator},
        types::{
            base::{FSRGlobalObjId, FSRObject, FSRValue, ObjId}, bool::FSRBool, class::FSRClass, float::FSRFloat, fn_def::FSRFn, integer::FSRInteger, iterator::FSRInnerIterator, list::FSRList, module::FSRModule, string::FSRString
        },
    },
    std::{io::init_io, utils::init_utils},
};

use super::thread::FSRThreadRuntime;

#[derive(Hash, Debug, Eq, PartialEq)]
pub enum ConstType<'a> {
    String(&'a str),
    Integer(i64),
}

pub struct FSRVM<'a> {
    global: HashMap<String, ObjId>,
    global_modules: HashMap<&'a str, ObjId>,
    const_integer_global: RefCell<HashMap<i64, ObjId>>,
    pub(crate) const_map: Mutex<AHashMap<ConstType<'a>, ObjId>>,
    pub allocator: FSRObjectAllocator<'a>,
    pub garbage_collector: GarbageCollector,
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
    #[inline(always)]
    pub fn has_str_const(&self, id: &str) -> bool {
        self.const_map
            .lock()
            .unwrap()
            .contains_key(&ConstType::String(id))
    }

    pub fn insert_str_const(&mut self, id: &'a str, obj: FSRObject<'a>) -> ObjId {
        let obj_id = FSRVM::leak_object(Box::new(obj));
        self.const_map
            .lock()
            .unwrap()
            .insert(ConstType::String(id), obj_id);
        obj_id
    }

    #[inline(always)]
    pub fn get_str_const(&self, id: &'a str) -> Option<ObjId> {
        if let Some(s) = self.const_map.lock().unwrap().get(&ConstType::String(id)) {
            if s == &0 {
                return None;
            }

            return Some(*s);
        }

        None
    }

    #[inline(always)]
    pub fn has_int_const(&self, id: &i64) -> bool {
        self.const_map
            .lock()
            .unwrap()
            .contains_key(&ConstType::Integer(*id))
    }

    pub fn has_float_const(&self, id: &f64) -> bool {
        self.const_map
            .lock()
            .unwrap()
            .contains_key(&ConstType::Integer(*id as i64))
    }

    pub fn insert_int_const(&mut self, id: &i64, obj: FSRObject<'a>) {
        let obj_id = FSRVM::leak_object(Box::new(obj));
        self.const_map
            .lock()
            .unwrap()
            .insert(ConstType::Integer(*id), obj_id);
    }

    pub fn insert_float_const(&mut self, id: &f64, obj: FSRObject<'a>) {
        let obj_id = FSRVM::leak_object(Box::new(obj));
        self.const_map
            .lock()
            .unwrap()
            .insert(ConstType::Integer(*id as i64), obj_id);
    }

    #[inline(always)]
    pub fn get_int_const(&self, id: &i64) -> Option<ObjId> {
        if let Some(s) = self.const_map.lock().unwrap().get(&ConstType::Integer(*id)) {
            if s == &0 {
                return None;
            }

            return Some(*s);
        }

        None
    }


    pub fn get_float_const(&self, id: &f64) -> Option<ObjId> {
        if let Some(s) = self.const_map.lock().unwrap().get(&ConstType::Integer(*id as i64)) {
            if s == &0 {
                return None;
            }

            return Some(*s);
        }

        None
    }

    pub fn new() -> Self {
        Self::init_static_object();
        let mut v = Self {
            global: HashMap::new(),
            global_modules: HashMap::new(),
            const_integer_global: RefCell::new(HashMap::new()),
            const_map: Mutex::new(AHashMap::new()),
            allocator: FSRObjectAllocator::new(),
            garbage_collector: GarbageCollector::new(),
        };
        v.init();
        v
    }

    #[inline(always)]
    pub fn get_true_id(&self) -> ObjId {
        1
    }

    #[inline(always)]
    pub fn get_false_id(&self) -> ObjId {
        2
    }

    #[inline(always)]
    pub fn get_none_id(&self) -> ObjId {
        0
    }

    pub fn init_static_object() {
        unsafe {
            if OBJECTS.is_empty() {
                OBJECTS.push(Self::new_stataic_object_with_id(
                    FSRGlobalObjId::None as ObjId,
                    FSRValue::None,
                ));
                OBJECTS.push(Self::new_stataic_object_with_id(
                    FSRGlobalObjId::True as ObjId,
                    FSRValue::Bool(true),
                ));
                OBJECTS.push(Self::new_stataic_object_with_id(
                    FSRGlobalObjId::False as ObjId,
                    FSRValue::Bool(false),
                ));
                OBJECTS.push(Self::new_stataic_object_with_id(
                    FSRGlobalObjId::IntegerCls as ObjId,
                    FSRValue::Class(Box::new(FSRInteger::get_class())),
                ));
                OBJECTS.push(Self::new_stataic_object_with_id(
                    FSRGlobalObjId::FnCls as ObjId,
                    FSRValue::Class(Box::new(FSRFn::get_class())),
                ));
                OBJECTS.push(Self::new_stataic_object_with_id(
                    FSRGlobalObjId::InnerIterator as ObjId,
                    FSRValue::Class(Box::new(FSRInnerIterator::get_class())),
                ));
                OBJECTS.push(Self::new_stataic_object_with_id(
                    FSRGlobalObjId::ListCls as ObjId,
                    FSRValue::Class(Box::new(FSRList::get_class())),
                ));
                OBJECTS.push(Self::new_stataic_object_with_id(
                    FSRGlobalObjId::StringCls as ObjId,
                    FSRValue::Class(Box::new(FSRString::get_class())),
                ));
                OBJECTS.push(Self::new_stataic_object_with_id(
                    FSRGlobalObjId::ClassCls as ObjId,
                    FSRValue::Class(Box::new(FSRClass::new("Class"))),
                ));
                OBJECTS.push(Self::new_stataic_object_with_id(
                    FSRGlobalObjId::ModuleCls as ObjId,
                    FSRValue::Class(Box::new(FSRModule::get_class())),
                ));
                OBJECTS.push(Self::new_stataic_object_with_id(
                    FSRGlobalObjId::BoolCls as ObjId,
                    FSRValue::Class(Box::new(FSRBool::get_class())),
                ));
                OBJECTS.push(Self::new_stataic_object_with_id(
                    FSRGlobalObjId::FloatCls as ObjId,
                    FSRValue::Class(Box::new(FSRFloat::get_class())),
                ));
            }
        }
    }

    /*
    init object like true false
    */

    fn init_global_object(&mut self) {
        self.global.insert("true".to_string(), self.get_true_id());
        self.global.insert("false".to_owned(), self.get_false_id());
    }

    pub fn init(&mut self) {
        let objs = init_io();
        for obj in objs {
            let id = FSRVM::register_object(obj.1);
            self.global.insert(obj.0.to_string(), id);
        }

        let objs = init_utils();

        for obj in objs {
            let id = FSRVM::register_object(obj.1);
            self.global.insert(obj.0.to_string(), id);
        }

        self.init_global_object();
    }

    pub fn get_base_cls(cls_id: ObjId) -> Option<&'static FSRClass<'a>> {
        unsafe {
            if let Some(s) = OBJECTS.get(cls_id) {
                if let FSRValue::Class(c) = &s.value {
                    return Some(c);
                } else {
                    return None;
                }
            }
        }
        None
    }

    fn new_stataic_object_with_id(id: ObjId, value: FSRValue<'static>) -> FSRObject<'static> {
        FSRObject {
            value,
            cls: 0,
            ref_count: AtomicU32::new(0),
            delete_flag: AtomicBool::new(true),
            leak: AtomicBool::new(false),
            garbage_id: AtomicU32::new(0),
        }
    }

    pub fn register_global_object(&mut self, name: &str, obj_id: ObjId) {
        self.global.insert(name.to_string(), obj_id);
    }

    fn get_object_id(obj: &FSRObject) -> ObjId {
        obj as *const FSRObject as ObjId
    }

    pub fn leak_object(mut object: Box<FSRObject<'a>>) -> ObjId {
        #[cfg(feature = "alloc_trace")]
        crate::backend::types::base::HEAP_TRACE.add_object();
        let id = Self::get_object_id(&object);

        object.leak.store(true, Ordering::Relaxed);
        //self.obj_map.insert(id, object);
        Box::leak(object);
        id
    }

    pub fn register_object(object: FSRObject<'a>) -> ObjId {
        let mut object = Box::new(object);
        let id = FSRObject::obj_to_id(&object);
        //self.obj_map.insert(id, object);
        Box::leak(object);
        id
    }

    pub fn get_global_obj_by_name(&self, name: &str) -> Option<&ObjId> {
        return self.global.get(name);
    }

    pub fn register_module(&mut self, name: &'a str, module: ObjId) {
        self.global_modules.insert(name, module);
    }

    pub fn get_module(&self, name: &str) -> Option<ObjId> {
        self.global_modules.get(name).copied()
    }
}
