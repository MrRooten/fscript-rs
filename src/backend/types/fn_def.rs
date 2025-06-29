use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::{Debug, Formatter},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use ahash::AHashMap;

use crate::{
    backend::{
        compiler::bytecode::Bytecode,
        vm::{thread::FSRThreadRuntime, virtual_machine::get_object_by_global_id},
    },
    utils::error::FSRError,
};

use super::{
    base::{Area, AtomicObjId, GlobalObj, FSRObject, FSRRetValue, FSRValue, ObjId},
    class::FSRClass,
};

pub type FSRRustFn = for<'a> fn(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime<'a>,
    code: ObjId,
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
    pub fn get_name(&self) -> &Cow<str> {
        &self.name
    }

    pub fn get_ip(&self) -> (usize, usize) {
        self.fn_ip
    }

    // pub fn get_bytecode(&self) -> &Bytecode {
    //     self.bytecode
    // }
}

#[derive(Debug)]
pub enum FSRnE<'a> {
    RustFn((Cow<'a, str>, FSRRustFn)),
    FSRFn(FSRFnInner<'a>),
}

pub struct FSRFn<'a> {
    pub(crate) fn_def: FSRnE<'a>,
    pub(crate) code: ObjId,
    pub(crate) closure_fn: Vec<ObjId>, // fn define chain
    /// Store cells for closure variables
    /// The key is the variable name, and the value is the object id
    ///
    /// fn abc() {
    ///     a = 1
    ///     fn def() {
    ///          println(a)
    ///     }
    /// }
    ///
    pub(crate) store_cells: AHashMap<&'a str, AtomicObjId>,
}

impl Debug for FSRFn<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {:?}>", self.as_str())
    }
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
        self.store_cells
            .values()
            .map(|s| s.load(Ordering::Relaxed))
            .collect()
    }

    pub fn as_str(&self) -> String {
        if let FSRnE::RustFn(r) = &self.fn_def {
            return format!("<fn {:?}>", r);
        } else if let FSRnE::FSRFn(f) = &self.fn_def {
            return format!("<fn {:?}>", f.name);
        }

        unimplemented!()
    }

    pub fn get_name(&self) -> &Cow<str> {
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

    pub fn get_def(&self) -> &FSRnE {
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
        };
        FSRValue::Function(Box::new(v))
    }

    pub fn from_fsr_fn(
        fn_name: &str,
        u: (usize, usize),
        _: Vec<String>,
        //bytecode: &'a Bytecode,
        code_obj: ObjId,
        fn_id: ObjId, // Which father fn define this son fn
        jit_code: Option<*const u8>,
        is_async: bool,
    ) -> FSRValue<'a> {
        let fn_obj = FSRFnInner {
            name: Cow::Owned(fn_name.to_string()),
            fn_ip: u,
            jit_code: jit_code.map(|x| x as usize),
            is_async: false,
            //bytecode,
        };

        let c = if fn_id != 0 {
            let obj = FSRObject::id_to_obj(fn_id);
            let father_fn = obj.as_fn();
            let mut closure = father_fn.closure_fn.clone();
            closure.push(fn_id);
            closure
        } else {
            vec![]
        };

        let v = Self {
            fn_def: FSRnE::FSRFn(fn_obj),
            code: code_obj,
            closure_fn: c,
            store_cells: AHashMap::new(),
        };
        FSRValue::Function(Box::new(v))
    }

    pub fn from_rust_fn_static(f: FSRRustFn, name: &'a str) -> FSRObject<'a> {
        let v = Self {
            fn_def: FSRnE::RustFn((Cow::Borrowed(name), f)),
            code: 0,
            closure_fn: vec![],
            store_cells: AHashMap::new(),
        };
        FSRObject {
            value: FSRValue::Function(Box::new(v)),
            cls: FSRObject::id_to_obj(get_object_by_global_id(GlobalObj::FnCls)).as_class(),
            // garbage_id: 0,
            // garbage_collector_id: 0,
            free: false,
            mark: AtomicBool::new(false),
            area: Area::Global,
            write_barrier: AtomicBool::new(true),
            gc_count: 0,
        }
    }

    pub fn get_class() -> FSRClass<'static> {
        FSRClass::new_without_method("Fn")
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn invoke(
        &'a self,
        args: &[ObjId],
        thread: &mut FSRThreadRuntime<'a>,
        code: ObjId,
        fn_id: ObjId,
    ) -> Result<FSRRetValue, FSRError> {
        if let FSRnE::RustFn(f) = &self.fn_def {
            let len = args.len();
            let args = args.as_ptr();
            let v = f.1(args, len, thread, code);
            return v;
        } else if let FSRnE::FSRFn(f) = &self.fn_def {
            if f.jit_code.is_some() {
                let code = *f.jit_code.as_ref().unwrap();
                let code = code as *const u8;
                //  self.ctx.func.signature.params.push(AbiParam::new(ptr)); // Add a parameter for the thread runtime.
                // self.ctx.func.signature.params.push(AbiParam::new(ptr)); // Add a parameter for the code object.
                // self.ctx.func.signature.params.push(AbiParam::new(ptr)); // Add a parameter for list of arguments.
                // self.ctx
                //     .func
                //     .signature
                //     .params
                //     .push(AbiParam::new(types::I32)); // Add a parameter for the number of arguments.
                // self.ctx.func.signature.returns.push(AbiParam::new(ptr)); // Add a return type for the function.
                let frame = thread
                    .frame_free_list
                    .new_frame(FSRObject::id_to_obj(fn_id).as_fn().code, fn_id);
                thread.push_frame(frame);
                for arg in args.iter() {
                    thread.get_cur_mut_frame().args.push(*arg);
                }
                let call_fn = unsafe {
                    std::mem::transmute::<
                        _,
                        extern "C" fn(&mut FSRThreadRuntime<'a>, ObjId, *const ObjId, i32) -> ObjId,
                    >(code)
                };
                let res = call_fn(thread, thread.get_context().code, args.as_ptr(), args.len() as i32);
                let v = thread.pop_frame();
                thread.frame_free_list.free(v);
                return Ok(FSRRetValue::GlobalId(res));
            }

            let frame = thread.frame_free_list.new_frame(code, fn_id);
            thread.push_frame(frame);
            let v = FSRThreadRuntime::call_fn(thread, f, args, self.code)?;
            return Ok(FSRRetValue::GlobalId(v));
        }
        unimplemented!()
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn invoke_binary(
        &'a self,
        left: ObjId,
        right: ObjId,
        thread: &mut FSRThreadRuntime<'a>,
        code: ObjId,
        fn_id: ObjId,
    ) -> Result<FSRRetValue, FSRError> {
        if let FSRnE::RustFn(f) = &self.fn_def {
            let args = [left, right];
            let len = args.len();
            let args = args.as_ptr();
            let v = f.1(args, len, thread, code);
            return v;
        } else if let FSRnE::FSRFn(f) = &self.fn_def {
            let frame = thread.frame_free_list.new_frame(code, fn_id);
            thread.push_frame(frame);
            let v = FSRThreadRuntime::call_fn(thread, f, &[left, right], self.code)?;
            return Ok(FSRRetValue::GlobalId(v));
        }
        unimplemented!()
    }
}
