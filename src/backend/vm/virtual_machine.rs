use std::{
    cell::RefCell,
    collections::{HashMap, LinkedList},
    os::unix::thread,
    sync::{
        atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering},
        Arc, Condvar, Mutex, MutexGuard,
    },
};

use ahash::AHashMap;

use crate::{
    backend::{
        memory::size_alloc::FSRObjectAllocator,
        types::{
            base::{FSRGlobalObjId, FSRObject, FSRValue, ObjId},
            bool::FSRBool,
            class::FSRClass,
            code::FSRCode,
            error::FSRException,
            float::FSRFloat,
            fn_def::FSRFn,
            integer::FSRInteger,
            iterator::FSRInnerIterator,
            list::FSRList,
            module::FSRModule,
            range::FSRRange,
            string::FSRString,
        },
    },
    std::{gc::init_gc, io::init_io, thread::init_thread, utils::init_utils},
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
    threads: Mutex<Vec<Mutex<FSRThreadRuntime<'a>>>>,
    thread_stop: Mutex<Vec<Arc<(Mutex<bool>, Condvar)>>>,
    thread_id_up : AtomicUsize,
    thread_len: AtomicUsize
}

// pub static mut NONE_OBJECT: Option<FSRObject> = None;
// pub static mut TRUE_OBJECT: Option<FSRObject> = None;
// pub static mut FALSE_OBJECT: Option<FSRObject> = None;
pub static mut OBJECTS: Vec<FSRObject> = vec![];
impl Default for FSRVM<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> FSRVM<'a> {
    pub fn new() -> Self {
        Self::init_static_object();
        let mut v = Self {
            global: HashMap::new(),
            global_modules: HashMap::new(),
            threads: Mutex::new(vec![]),
            thread_stop: Mutex::new(vec![]),
            thread_id_up: AtomicUsize::new(0),
            thread_len: AtomicUsize::new(0)
        };
        v.init();
        v
    }

    pub fn get_thread<F, R>(&self, thread_id: usize, f: F) -> Option<R>
    where
        F: FnOnce(&mut FSRThreadRuntime<'a>) -> R,
    {
        let threads = self.threads.lock().unwrap();
        // threads.iter().find(|x| x.).map(|thread| {
        //     let mut t = thread.lock().unwrap();
        //     f(&mut t)
        // })
        if thread_id < threads.len() {
            let thread = &threads[thread_id];
            let mut thread_guard = thread.lock().unwrap();
            Some(f(&mut thread_guard))
        } else {
            None
        }
    }

    pub fn add_thread(&self, thread: Mutex<FSRThreadRuntime<'a>>) -> usize {
        let len = self.threads.lock().unwrap().len();
        thread.lock().unwrap().thread_id = len;
        let id = len;
        self.threads.lock().unwrap().push(thread);
        self.thread_stop
            .lock()
            .unwrap()
            .push(Arc::new((Mutex::new(false), Condvar::new())));
        self.thread_id_up.fetch_add(1, Ordering::SeqCst);
        self.thread_len.store(self.threads.lock().unwrap().len(), Ordering::Relaxed);
        id
    }

    #[inline(always)]
    pub fn get_true_id(&self) -> ObjId {
        1
    }

    pub fn stop_all_threads(&self) {
        for i in 0..self.thread_len.load(Ordering::SeqCst) {
            let pair = self.thread_stop.lock().unwrap()[i].clone();
            self.threads.lock().unwrap().get(i).unwrap().lock().unwrap().stop(pair.clone());
            *pair.0.lock().unwrap() = true;
        }
        
    }


    pub fn continue_all_threads(&self) {
        for i in 0..self.thread_len.load(Ordering::SeqCst) {
            let pair = self.thread_stop.lock().unwrap().get(i).unwrap().clone();
            *pair.0.lock().unwrap() = false;
            pair.1.notify_all();
        }
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
                    FSRGlobalObjId::CodeCls as ObjId,
                    FSRValue::Class(Box::new(FSRCode::get_class())),
                ));
                OBJECTS.push(Self::new_stataic_object_with_id(
                    FSRGlobalObjId::BoolCls as ObjId,
                    FSRValue::Class(Box::new(FSRBool::get_class())),
                ));
                OBJECTS.push(Self::new_stataic_object_with_id(
                    FSRGlobalObjId::FloatCls as ObjId,
                    FSRValue::Class(Box::new(FSRFloat::get_class())),
                ));
                OBJECTS.push(Self::new_stataic_object_with_id(
                    FSRGlobalObjId::Exception as ObjId,
                    FSRValue::Class(Box::new(FSRException::get_class())),
                ));
                OBJECTS.push(Self::new_stataic_object_with_id(
                    FSRGlobalObjId::RangeCls as ObjId,
                    FSRValue::Class(Box::new(FSRRange::get_class())),
                ));
                OBJECTS.push(Self::new_stataic_object_with_id(
                    FSRGlobalObjId::ModuleCls as ObjId,
                    FSRValue::Class(Box::new(FSRModule::get_class())),
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
        self.global.insert("none".to_string(), self.get_none_id());
        self.global
            .insert("Exception".to_string(), FSRGlobalObjId::Exception as ObjId);
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
            garbage_id: 0,
            garbage_collector_id: 0,
            free: false,
            mark: false,
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
        self.global.get(name)
    }

    pub fn register_module(&mut self, name: &'a str, module: ObjId) {
        self.global_modules.insert(name, module);
    }

    pub fn get_module(&self, name: &str) -> Option<ObjId> {
        self.global_modules.get(name).copied()
    }
}
