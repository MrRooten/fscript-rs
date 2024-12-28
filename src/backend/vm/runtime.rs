use std::{cell::{Cell, RefCell}, collections::HashMap, sync::{atomic::AtomicU64, Mutex}};

use ahash::AHashMap;

use crate::{
    backend::{memory::size_alloc::FSRObjectAllocator, types::{
        base::{FSRGlobalObjId, FSRObject, FSRValue, ObjId}, class::FSRClass, fn_def::FSRFn, integer::FSRInteger, iterator::FSRInnerIterator, list::FSRList, module::FSRModule, string::FSRString
    }},
    std::{io::init_io, utils::init_utils},
};

use super::thread::FSRThreadRuntime;

#[derive(Hash, Debug, Eq, PartialEq)]
pub enum ConstType<'a> {
    String(&'a str),
    Integer(i64)
}

pub struct FSRVM<'a> {
    global: HashMap<String, ObjId>,
    global_modules  : HashMap<&'a str, ObjId>,
    const_integer_global: RefCell<HashMap<i64, ObjId>>,
    pub(crate) const_map: Mutex<AHashMap<ConstType<'a>, ObjId>>,
    pub allocator   : FSRObjectAllocator<'a>
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
        self.const_map.lock().unwrap().contains_key(&ConstType::String(id))
    }

    pub fn insert_str_const(&mut self, id: &'a str, obj: FSRObject<'a>) {
        let obj_id = FSRVM::leak_object(Box::new(obj));
        self.const_map.lock().unwrap().insert(ConstType::String(id), obj_id);
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
        self.const_map.lock().unwrap().contains_key(&ConstType::Integer(*id))
    }

    pub fn insert_int_const(&mut self, id: &i64, obj: FSRObject<'a>) {
        let obj_id = FSRVM::leak_object(Box::new(obj));
        self.const_map.lock().unwrap().insert(ConstType::Integer(*id), obj_id);
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

    pub fn new() -> Self {
        Self::init_static_object();
        let mut v = Self {
            global: HashMap::new(),
            global_modules: HashMap::new(),
            const_integer_global: RefCell::new(HashMap::new()),
            const_map: Mutex::new(AHashMap::new()),
            allocator: FSRObjectAllocator::new(),
        };
        v.init();
        v
    }

    pub fn get_integer(&self, integer: i64) -> ObjId {
        let mut const_obj = self.const_integer_global.borrow_mut();
        const_obj.entry(integer).or_insert_with(|| {
            let obj = FSRObject {
                obj_id: 0,
                value: FSRValue::Integer(integer),
                cls: FSRGlobalObjId::IntegerCls as ObjId,
                ref_count: AtomicU64::new(1),
                delete_flag: Cell::new(true),
                leak: Cell::new(false),
            };

            FSRVM::register_object(obj)
        });

        *const_obj.get(&integer).unwrap()
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
                OBJECTS.push(Self::new_stataic_object_with_id(0, FSRValue::None));
                OBJECTS.push(Self::new_stataic_object_with_id(1, FSRValue::Bool(true)));
                OBJECTS.push(Self::new_stataic_object_with_id(2, FSRValue::Bool(false)));
                OBJECTS.push(Self::new_stataic_object_with_id(3, FSRValue::Class(Box::new(FSRInteger::get_class()))));
                OBJECTS.push(Self::new_stataic_object_with_id(4, FSRValue::Class(Box::new(FSRFn::get_class()))));
                OBJECTS.push(Self::new_stataic_object_with_id(5, FSRValue::Class(Box::new(FSRInnerIterator::get_class()))));
                OBJECTS.push(Self::new_stataic_object_with_id(6, FSRValue::Class(Box::new(FSRList::get_class()))));
                OBJECTS.push(Self::new_stataic_object_with_id(7, FSRValue::Class(Box::new(FSRString::get_class()))));
                OBJECTS.push(Self::new_stataic_object_with_id(8, FSRValue::Class(Box::new(FSRClass::new("Class")))));
                OBJECTS.push(Self::new_stataic_object_with_id(9, FSRValue::Class(Box::new(FSRModule::get_class()))));
            }
        }
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
    }

    pub fn get_base_cls(&self, cls_id: ObjId) -> Option<&FSRClass<'a>> {
        unsafe {
            if let Some(s) = OBJECTS.get(cls_id) {
                if let FSRValue::Class(c) = &s.value {
                    return Some(c);
                } else {
                    return None
                }
            }
        }
        None
    }

    fn new_stataic_object_with_id(id: ObjId, value: FSRValue<'static>) -> FSRObject<'static> {
        FSRObject {
            obj_id: id,
            value,
            cls: 0,
            ref_count: AtomicU64::new(0),
            delete_flag: Cell::new(true),
            leak: Cell::new(false),
        }
    }

    pub fn register_global_object(&mut self, name: &str, obj_id: ObjId) {
        self.global.insert(name.to_string(), obj_id);
    }

    fn get_object_id(obj: &FSRObject) -> ObjId {
        obj as *const FSRObject as ObjId
    }

    pub fn leak_object(mut object: Box<FSRObject<'a>>) -> ObjId {
        #[cfg(feature="alloc_trace")]
        crate::backend::types::base::HEAP_TRACE.add_object();
        let id = Self::get_object_id(&object);
        object.obj_id = id;
        object.leak.set(true);
        //self.obj_map.insert(id, object);
        Box::leak(object);
        id
    }

    pub fn register_object(object: FSRObject<'a>) -> ObjId {
        let mut object = Box::new(object);
        let id = Self::get_object_id(&object);
        object.obj_id = id;

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
