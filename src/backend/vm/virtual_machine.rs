use std::{
    cell::UnsafeCell,
    collections::HashMap,
    sync::{atomic::AtomicBool, Arc, Mutex},
};

use ahash::AHashMap;

use crate::{
    backend::types::{
        any::FSRThreadHandle,
        base::{Area, FSRObject, FSRValue, GlobalObj, ObjId, FALSE_ID, NONE_ID, TRUE_ID},
        bool::FSRBool,
        bytes::FSRInnerBytes,
        class::FSRClass,
        code::FSRCode,
        error::FSRException,
        ext::{hashmap::FSRHashMap, hashset::FSRHashSet},
        float::FSRFloat,
        fn_def::FSRFn,
        integer::FSRInteger,
        iterator::FSRInnerIterator,
        list::FSRList,
        module::{FSRModule, NewModuleFn},
        none::FSRNone,
        range::FSRRange,
        string::FSRString,
    },
    std::{
        core::{gc::init_gc, io::init_io, thread::init_thread, utils::init_utils},
        fs::{file::FSRInnerFile, FSRFileSystem},
    },
};

use super::thread::FSRThreadRuntime;

#[derive(Hash, Debug, Eq, PartialEq)]
pub enum ConstType<'a> {
    String(&'a str),
    Integer(i64),
}

pub struct FSRVM<'a> {
    global: AHashMap<String, ObjId>,
    global_modules: AHashMap<&'a str, ObjId>,
    pub(crate) core_module: AHashMap<&'static str, NewModuleFn>,
    threads: Mutex<Vec<Option<UnsafeCell<FSRThreadRuntime<'a>>>>>,
}

pub static mut VM: Option<Arc<FSRVM<'static>>> = None;

// pub static mut NONE_OBJECT: Option<FSRObject> = None;
// pub static mut TRUE_OBJECT: Option<FSRObject> = None;
// pub static mut FALSE_OBJECT: Option<FSRObject> = None;
pub static mut OBJECTS: Vec<Option<FSRObject>> = vec![];
impl Default for FSRVM<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(feature = "more_inline", inline(always))]
pub extern "C" fn get_object_by_global_id(id: GlobalObj) -> ObjId {
    unsafe {
        let obj = OBJECTS.get(id as usize).unwrap();
        if let Some(obj) = obj {
            FSRObject::obj_to_id(obj)
        } else {
            for i in OBJECTS.iter().enumerate() {
                println!("{}: {:?}", i.0, i.1);
            }
            panic!("Global object not found: {:?}", &OBJECTS[id as usize]);
        }
    }
}

#[cfg_attr(feature = "more_inline", inline(always))]
pub extern "C" fn get_true() -> ObjId {
    get_object_by_global_id(GlobalObj::True)
}

#[cfg_attr(feature = "more_inline", inline(always))]
pub extern "C" fn get_false() -> ObjId {
    get_object_by_global_id(GlobalObj::False)
}

#[cfg_attr(feature = "more_inline", inline(always))]
pub extern "C" fn get_none() -> ObjId {
    get_object_by_global_id(GlobalObj::NoneObj)
}

pub static mut GLOBAL_CLASS: Option<FSRClass<'static>> = None;

impl<'a> FSRVM<'a> {
    pub fn core_module() -> AHashMap<&'static str, NewModuleFn> {
        let mut res: AHashMap<&'static str, fn(&mut FSRThreadRuntime<'_>) -> FSRValue<'static>> =
            AHashMap::new();
        res.insert("fs", FSRFileSystem::new_module);
        res
    }

    fn new() -> Self {
        Self::init_static_object();
        let core_module = FSRVM::core_module();
        let mut v = Self {
            global: AHashMap::new(),
            global_modules: AHashMap::new(),
            threads: Mutex::new(vec![]),
            core_module,
        };
        v.init();
        v
    }

    pub fn single() -> Arc<FSRVM<'static>> {
        unsafe {
            if VM.is_none() {
                VM = Some(Arc::new(FSRVM::new()))
            }

            VM.as_ref().unwrap().clone()
        }
    }

    pub fn get_thread(&self, thread_id: usize) -> Option<&mut FSRThreadRuntime<'a>> {
        let threads = self.threads.lock().unwrap();
        if thread_id >= threads.len() {
            return None;
        }
        let thread = unsafe { &mut *threads[thread_id].as_ref().unwrap().get() };
        Some(thread)
    }

    pub fn add_thread(&self, mut thread: FSRThreadRuntime<'a>) -> usize {
        let mut threads_guard = self.threads.lock().unwrap();
        let mut id = 0;
        for item in threads_guard.iter() {
            if item.is_none() {
                break;
            }
            id += 1;
        }

        thread.thread_id = id;
        if id >= threads_guard.len() {
            threads_guard.push(Some(UnsafeCell::new(thread)));
        } else {
            threads_guard[id] = Some(UnsafeCell::new(thread));
        }

        id
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn get_true_id(&self) -> ObjId {
        1
    }

    pub fn stop_all_threads(&self) {
        let len = self.threads.lock().unwrap().len();
        for i in 0..len {
            let thread = self.get_thread(i).unwrap();
            thread.rt_stop();
        }

        for i in 0..len {
            let thread = self.get_thread(i).unwrap();
            thread.rt_wait_stop();
        }
        println!("stop all threads done");
    }

    pub fn continue_all_threads(&self) {
        let len = self.threads.lock().unwrap().len();
        for i in 0..len {
            let thread: &mut FSRThreadRuntime<'a> = self.get_thread(i).unwrap();
            thread.rt_continue();
        }
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn get_false_id(&self) -> ObjId {
        2
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn get_none_id(&self) -> ObjId {
        0
    }

    pub fn init_static_object() {
        unsafe {
            if OBJECTS.is_empty() {
                GLOBAL_CLASS = Some(FSRClass::new("Global"));
                OBJECTS.resize_with(100, || None);

                OBJECTS.insert(
                    GlobalObj::NoneObj as usize,
                    Some(Self::new_stataic_object_with_id(FSRValue::None)),
                );

                OBJECTS.insert(
                    GlobalObj::True as usize,
                    Some(Self::new_stataic_object_with_id(FSRValue::Bool(true))),
                );

                OBJECTS.insert(
                    GlobalObj::False as usize,
                    Some(Self::new_stataic_object_with_id(FSRValue::Bool(false))),
                );

                OBJECTS.insert(
                    GlobalObj::FnCls as usize,
                    Some(Self::new_stataic_object_with_id(FSRValue::Class(Box::new(
                        FSRFn::get_class(),
                    )))),
                );

                OBJECTS.insert(
                    GlobalObj::ClassCls as usize,
                    Some(Self::new_stataic_object_with_id(FSRValue::Class(Box::new(
                        FSRClass::new("Class"),
                    )))),
                );

                OBJECTS.insert(
                    GlobalObj::IntegerCls as usize,
                    Some(Self::new_stataic_object_with_id(FSRValue::Class(Box::new(
                        FSRInteger::get_class(),
                    )))),
                );

                OBJECTS.insert(
                    GlobalObj::InnerIterator as usize,
                    Some(Self::new_stataic_object_with_id(FSRValue::Class(Box::new(
                        FSRInnerIterator::get_class(),
                    )))),
                );

                OBJECTS.insert(
                    GlobalObj::ListCls as usize,
                    Some(Self::new_stataic_object_with_id(FSRValue::Class(Box::new(
                        FSRList::get_class(),
                    )))),
                );

                OBJECTS.insert(
                    GlobalObj::StringCls as usize,
                    Some(Self::new_stataic_object_with_id(FSRValue::Class(Box::new(
                        FSRString::get_class(),
                    )))),
                );

                OBJECTS.insert(
                    GlobalObj::CodeCls as usize,
                    Some(Self::new_stataic_object_with_id(FSRValue::Class(Box::new(
                        FSRCode::get_class(),
                    )))),
                );

                OBJECTS.insert(
                    GlobalObj::BoolCls as usize,
                    Some(Self::new_stataic_object_with_id(FSRValue::Class(Box::new(
                        FSRBool::get_class(),
                    )))),
                );

                OBJECTS.insert(
                    GlobalObj::FloatCls as usize,
                    Some(Self::new_stataic_object_with_id(FSRValue::Class(Box::new(
                        FSRFloat::get_class(),
                    )))),
                );

                OBJECTS.insert(
                    GlobalObj::Exception as usize,
                    Some(Self::new_stataic_object_with_id(FSRValue::Class(Box::new(
                        FSRException::get_class(),
                    )))),
                );

                OBJECTS.insert(
                    GlobalObj::RangeCls as usize,
                    Some(Self::new_stataic_object_with_id(FSRValue::Class(Box::new(
                        FSRRange::get_class(),
                    )))),
                );

                OBJECTS.insert(
                    GlobalObj::ModuleCls as usize,
                    Some(Self::new_stataic_object_with_id(FSRValue::Class(Box::new(
                        FSRModule::get_class(),
                    )))),
                );

                OBJECTS.insert(
                    GlobalObj::ThreadCls as usize,
                    Some(Self::new_stataic_object_with_id(FSRValue::Class(Box::new(
                        FSRThreadHandle::thread_cls(),
                    )))),
                );

                OBJECTS.insert(
                    GlobalObj::HashMapCls as usize,
                    Some(Self::new_stataic_object_with_id(FSRValue::Class(Box::new(
                        FSRHashMap::get_class(),
                    )))),
                );

                OBJECTS.insert(
                    GlobalObj::NoneCls as usize,
                    Some(Self::new_stataic_object_with_id(FSRValue::Class(Box::new(
                        FSRNone::get_class(),
                    )))),
                );

                OBJECTS.insert(
                    GlobalObj::BytesCls as usize,
                    Some(Self::new_stataic_object_with_id(FSRValue::Class(Box::new(
                        FSRInnerBytes::get_class(),
                    )))),
                );

                OBJECTS.insert(GlobalObj::HashSetCls as usize, 
                    Some(Self::new_stataic_object_with_id(FSRValue::Class(Box::new(
                        FSRHashSet::get_class(),
                    )))),
                );

                for object in OBJECTS.iter_mut() {
                    if let Some(object) = object {
                        if let FSRValue::Class(c) = &mut object.value {
                            object.cls =
                                FSRObject::id_to_obj(get_object_by_global_id(GlobalObj::ClassCls))
                                    .as_class();
                            c.init_method();
                        }

                        if let FSRValue::Bool(_) = object.value {
                            object.cls =
                                FSRObject::id_to_obj(get_object_by_global_id(GlobalObj::BoolCls))
                                    .as_class();
                        }

                        if let FSRValue::None = object.value {
                            object.cls =
                                FSRObject::id_to_obj(get_object_by_global_id(GlobalObj::NoneCls))
                                    .as_class();
                        }
                    }
                }

                // OBJECTS[0].cls = get_object_by_global_id(FSRGlobalObjId::NoneCls);
                // OBJECTS[1].cls = get_object_by_global_id(FSRGlobalObjId::BoolCls);
                // OBJECTS[2].cls = get_object_by_global_id(FSRGlobalObjId::BoolCls);

                NONE_ID = FSRObject::obj_to_id(OBJECTS[0].as_ref().unwrap());
                TRUE_ID = FSRObject::obj_to_id(OBJECTS[1].as_ref().unwrap());
                FALSE_ID = FSRObject::obj_to_id(OBJECTS[2].as_ref().unwrap());
            }
        }
    }

    /*
    init object like true false
    */

    fn init_global_object(&mut self) {
        self.global.insert("true".to_string(), FSRObject::true_id());
        self.global
            .insert("false".to_owned(), FSRObject::false_id());
        self.global.insert("none".to_string(), FSRObject::none_id());
        self.global.insert(
            "Exception".to_string(),
            get_object_by_global_id(GlobalObj::Exception) as ObjId,
        );
        self.global.insert(
            "HashMap".to_string(),
            get_object_by_global_id(GlobalObj::HashMapCls) as ObjId,
        );
        self.global.insert(
            "HashSet".to_string(),
            get_object_by_global_id(GlobalObj::HashSetCls) as ObjId,
        );
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

        let objs = init_gc();
        for obj in objs {
            let id = FSRVM::register_object(obj.1);
            self.global.insert(obj.0.to_string(), id);
        }

        let objs = init_thread();
        for obj in objs {
            let id = FSRVM::register_object(obj.1);
            self.global.insert(obj.0.to_string(), id);
        }

        self.init_global_object();
    }

    fn new_stataic_object_with_id(value: FSRValue<'static>) -> FSRObject<'static> {
        FSRObject {
            value,
            cls: unsafe { GLOBAL_CLASS.as_ref().unwrap() },
            // garbage_id: 0,
            // garbage_collector_id: 0,
            free: false,
            mark: AtomicBool::new(false),
            area: Area::Global,
            write_barrier: AtomicBool::new(true),
            gc_count: 0,
        }
    }

    pub fn register_global_object(&mut self, name: &str, obj_id: ObjId) {
        self.global.insert(name.to_string(), obj_id);
    }

    fn get_object_id(obj: &FSRObject) -> ObjId {
        obj as *const FSRObject as ObjId
    }

    pub fn leak_object(object: Box<FSRObject<'a>>) -> ObjId {
        let id = Self::get_object_id(&object);

        //self.obj_map.insert(id, object);
        Box::leak(object);
        id
    }

    pub fn register_object(object: FSRObject<'a>) -> ObjId {
        let object = Box::new(object);
        let id = FSRObject::obj_to_id(&object);
        //self.obj_map.insert(id, object);
        Box::leak(object);
        id
    }

    pub fn get_global_obj_by_name(&self, name: &str) -> Option<&ObjId> {
        self.global.get(name)
    }

    pub fn register_module(&mut self, name: &'a str, module: ObjId) {
        self.global_modules.insert(name, module);
    }

    pub fn get_module(&self, name: &str) -> Option<ObjId> {
        self.global_modules.get(name).copied()
    }
}
