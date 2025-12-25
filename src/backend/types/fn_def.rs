use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::{Debug, Formatter},
    sync::{
        Arc, atomic::{AtomicBool, AtomicI16, Ordering}
    },
};

use ahash::AHashMap;

use crate::{
    backend::{
        compiler::bytecode::Bytecode,
        vm::{
            thread::{FSRThreadRuntime, IndexMap},
            virtual_machine::gid,
        },
    },
    utils::error::FSRError,
};

use super::{
    base::{Area, AtomicObjId, FSRObject, FSRRetValue, FSRValue, GlobalObj, ObjId},
    class::FSRClass,
};

pub type FSRRustFn = for<'a> fn(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime<'a>,
) -> Result<FSRRetValue, FSRError>;

#[derive(Debug, Clone)]
pub struct FSRFnInner<'a> {
    name: Cow<'a, str>,
    fn_ip: (usize, usize),
    pub(crate) jit_code: Option<usize>,
    pub(crate) is_async: bool,
    //bytecode: &'a Bytecode,
}

impl FSRFnInner<'_> {
    pub fn get_name(&self) -> &Cow<'_, str> {
        &self.name
    }

    pub fn get_ip(&self) -> (usize, usize) {
        self.fn_ip
    }
}

#[derive(Debug)]
pub enum FSRnE<'a> {
    RustFn((Cow<'a, str>, FSRRustFn)),
    FSRFn(FSRFnInner<'a>),
}

pub struct FSRJitInfo {
    run_count: AtomicI16,
}

impl FSRJitInfo {
    pub fn call_once(&self) {
        self.run_count.fetch_add(1, Ordering::Relaxed);
    }
}

pub struct FSRFn<'a> {
    pub(crate) fn_def: FSRnE<'a>,
    pub(crate) code: ObjId,
    pub(crate) closure_fn: Vec<ObjId>, // fn define chain
    /// Store cells for closure variables
    /// The key is the variable name, and the value is the object id
    pub(crate) store_cells: AHashMap<&'a str, AtomicObjId>,
    pub(crate) const_map: Arc<IndexMap>,
}

impl Debug for FSRFn<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {:?}>", self.as_str())
    }
}

pub struct FnDesc {
    pub(crate) u: (usize, usize),
    pub(crate) args: Vec<String>,
    //bytecode: &'a Bytecode,
    pub(crate) code_obj: ObjId,
    pub(crate) fn_id: ObjId, // Which father fn define this son fn
    pub(crate) jit_code: Option<*const u8>,
    pub(crate) is_async: bool,
    pub(crate) const_map: Arc<IndexMap>,
}

impl<'a> FSRFn<'a> {
    pub fn get_closure_var(&self, name: &str) -> Option<ObjId> {
        let obj = self.store_cells.get(name);
        if let Some(s) = obj {
            return Some(s.load(Ordering::Relaxed));
        }
        for i in self.closure_fn.iter().rev() {
            let obj = FSRObject::id_to_obj(*i);
            if let FSRValue::Function(f) = &obj.value {
                //println!("check closure fn: {:?}", f.store_cells);
                let v = match f.store_cells.get(name) {
                    Some(s) => s.load(Ordering::Relaxed),
                    None => continue,
                };
                return Some(v);
            }
        }
        None
    }

    pub fn get_references(&self) -> Vec<ObjId> {
        let mut v1: Vec<usize> = self
            .store_cells
            .values()
            .map(|s| s.load(Ordering::Relaxed))
            .collect();

        for val in self.const_map.iter() {
            v1.push(val.load(Ordering::Relaxed));
        }

        v1
    }

    pub fn as_str(&self) -> String {
        if let FSRnE::RustFn(r) = &self.fn_def {
            return format!("<fn {:?}>", r);
        } else if let FSRnE::FSRFn(f) = &self.fn_def {
            return format!("<fn {:?}>", f.name);
        }

        unimplemented!()
    }

    pub fn get_name(&self) -> &Cow<'_, str> {
        if let FSRnE::FSRFn(f) = &self.fn_def {
            return f.get_name();
        } else if let FSRnE::RustFn(f) = &self.fn_def {
            return &Cow::Borrowed("RustFn");
        }
        unimplemented!()
    }

    pub fn is_fsr_function(&self) -> bool {
        matches!(&self.fn_def, FSRnE::FSRFn(_))
    }

    pub fn get_def(&self) -> &FSRnE<'_> {
        &self.fn_def
    }

    pub fn get_args(&self) -> &Vec<String> {
        unimplemented!()
    }

    pub fn new_empty() -> FSRValue<'a> {
        let fn_obj = FSRFnInner {
            name: Cow::Owned("__main__".to_string()),
            fn_ip: (0, 0),
            jit_code: None,
            is_async: false,
        };

        let v = Self {
            fn_def: FSRnE::FSRFn(fn_obj),
            code: 0,
            closure_fn: vec![],
            store_cells: AHashMap::new(),
            const_map: Arc::new(IndexMap::new()),
        };
        FSRValue::Function(Box::new(v))
    }

    pub fn from_fsr_fn(fn_name: &str, fn_desc: FnDesc) -> FSRValue<'a> {
        let fn_obj = FSRFnInner {
            name: Cow::Owned(fn_name.to_string()),
            fn_ip: fn_desc.u,
            jit_code: fn_desc.jit_code.map(|x| x as usize),
            is_async: fn_desc.is_async,
        };

        let c = if fn_desc.fn_id != 0 {
            let obj = FSRObject::id_to_obj(fn_desc.fn_id);
            let father_fn = obj.as_fn();
            let mut closure = father_fn.closure_fn.clone();
            closure.push(fn_desc.fn_id);
            closure
        } else {
            vec![]
        };

        let v = Self {
            fn_def: FSRnE::FSRFn(fn_obj),
            code: fn_desc.code_obj,
            closure_fn: c,
            store_cells: AHashMap::new(),
            const_map: fn_desc.const_map,
        };
        FSRValue::Function(Box::new(v))
    }

    pub fn from_rust_fn_static(f: FSRRustFn, name: &'a str) -> FSRObject<'a> {
        let v = Self {
            fn_def: FSRnE::RustFn((Cow::Borrowed(name), f)),
            code: 0,
            closure_fn: vec![],
            store_cells: AHashMap::new(),
            const_map: Arc::new(IndexMap::new()),
        };
        FSRObject {
            value: FSRValue::Function(Box::new(v)),
            cls: FSRObject::id_to_obj(gid(GlobalObj::FnCls)).as_class(),
            free: false,
            mark: AtomicBool::new(false),
            area: Area::Global,
            write_barrier: AtomicBool::new(true),
            gc_count: 0,
        }
    }

    pub fn from_rust_fn_static_value(f: FSRRustFn, name: &'a str) -> FSRValue<'a> {
        let v = Self {
            fn_def: FSRnE::RustFn((Cow::Borrowed(name), f)),
            code: 0,
            closure_fn: vec![],
            store_cells: AHashMap::new(),
            const_map: Arc::new(IndexMap::new()),
        };

        FSRValue::Function(Box::new(v))
    }

    pub fn get_class() -> FSRClass {
        FSRClass::new_without_method("Fn")
    }

    pub fn call_jit(
        f: &FSRFnInner,
        thread: &mut FSRThreadRuntime<'a>,
        fn_id: ObjId,
        args: &[ObjId],
        code: ObjId,
    ) -> ObjId {
        let jit_code = *f.jit_code.as_ref().unwrap();
        let jit_code = jit_code as *const u8;
        let frame = thread
            .frame_free_list
            .new_frame(FSRObject::id_to_obj(fn_id).as_fn().code, fn_id);
        thread.push_frame(frame, FSRObject::id_to_obj(fn_id).as_fn().const_map.clone());
        for arg in args.iter() {
            thread.get_cur_mut_frame().args.push(*arg);
        }
        let call_fn = unsafe {
            std::mem::transmute::<
                _,
                extern "C" fn(&mut FSRThreadRuntime<'a>, ObjId, *const ObjId, i32) -> ObjId,
            >(jit_code)
        };
        let res = call_fn(thread, code, args.as_ptr(), args.len() as i32);
        let v = thread.pop_frame();
        thread.frame_free_list.free(v);
        res
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn invoke(
        &self,
        args: &[ObjId],
        thread: &mut FSRThreadRuntime<'a>,
        fn_id: ObjId,
    ) -> Result<FSRRetValue, FSRError> {
        match &self.fn_def {
            FSRnE::RustFn(f) => {
                let len = args.len();
                let args = args.as_ptr();
                let v = f.1(args, len, thread);
                return v;
            }
            FSRnE::FSRFn(f) => {
                let frame = thread.frame_free_list.new_frame(self.code, fn_id);
                thread.push_frame(frame, FSRObject::id_to_obj(fn_id).as_fn().const_map.clone());
                thread
                    .get_cur_mut_frame()
                    .args
                    .extend(args.iter().rev().cloned());
                let v = FSRThreadRuntime::call_fn(thread, f, self.code)?;
                return Ok(FSRRetValue::GlobalId(v));
            }
        }
    }
}
