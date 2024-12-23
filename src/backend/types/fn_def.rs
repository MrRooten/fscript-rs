use std::{borrow::Cow, cell::RefCell, sync::atomic::AtomicU64};

use crate::{
    backend::{compiler::bytecode::Bytecode, vm::{runtime::FSRVM, thread::FSRThreadRuntime}},
    utils::error::FSRError,
};

use super::{
    base::{FSRGlobalObjId, FSRObject, FSRRetValue, FSRValue, ObjId},
    class::FSRClass, module::FSRModule,
};

type FSRRustFn = for<'a> fn(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: Option<&'a FSRModule<'a>>
) -> Result<FSRRetValue<'a>, FSRError>;

#[derive(Debug, Clone)]
pub struct FSRFnInner<'a> {
    name    : Cow<'a, str>,
    fn_ip   : (usize, usize),
    bytecode    : &'a Bytecode
}

impl<'a> FSRFnInner<'a> {
    pub fn get_name(&self) -> &Cow<str> {
        &self.name
    }

    pub fn get_ip(&self) -> (usize, usize) {
        self.fn_ip
    }

    pub fn get_bytecode(&self) -> &Bytecode {
        self.bytecode
    }
}

#[derive(Debug, Clone)]
pub enum FSRnE<'a> {
    RustFn(FSRRustFn),
    FSRFn(FSRFnInner<'a>),
}

#[derive(Debug, Clone)]
pub struct FSRFn<'a> {
    fn_def: FSRnE<'a>,
}

impl<'a> FSRFn<'a> {
    pub fn as_str(&self) -> String {
        if let FSRnE::RustFn(r) = self.fn_def {
            return format!("<fn {:?}>", r)
        } else if let FSRnE::FSRFn(f) = &self.fn_def {
            return format!("<fn {:?}>", f.name)
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

    pub fn from_fsr_fn(module: &str, u: (usize, usize), _: Vec<String>, bytecode: &'a Bytecode) -> FSRObject<'a> {
        let fn_obj = FSRFnInner {
            name: Cow::Owned(module.to_string()),
            fn_ip: u,
            bytecode
        };

        let v = Self {
            fn_def: FSRnE::FSRFn(fn_obj),
        };
        FSRObject {
            obj_id: 0,
            value: FSRValue::Function(v),
            cls: FSRGlobalObjId::FnCls as ObjId,
            ref_count: AtomicU64::new(0),
            delete_flag: RefCell::new(true),
        }
    }

    pub fn from_rust_fn(f: FSRRustFn) -> FSRObject<'a> {
        let v = Self {
            fn_def: FSRnE::RustFn(f),
        };
        FSRObject {
            obj_id: 0,
            value: FSRValue::Function(v),
            cls: FSRGlobalObjId::FnCls as ObjId,
            ref_count: AtomicU64::new(0),
            delete_flag: RefCell::new(true),
        }
    }

    pub fn get_class() -> FSRClass<'static> {
        FSRClass::new("Fn")
    }

    pub fn invoke(
        &'a self,
        args: &[ObjId],
        thread: &mut FSRThreadRuntime<'a>,
        module: Option<&'a FSRModule<'a>>,
    ) -> Result<FSRRetValue, FSRError> {
        if let FSRnE::RustFn(f) = &self.fn_def {
            return f(args, thread, module);
        } else if let FSRnE::FSRFn(f) = &self.fn_def {
            let v = FSRThreadRuntime::call_fn(thread, f, args, module)?;
            let v = match v {
                crate::backend::vm::thread::SValue::Global(g) => g,
                crate::backend::vm::thread::SValue::Object(o) => {
                    FSRVM::leak_object(o)
                },
                _ => unimplemented!()
            };
            return Ok(FSRRetValue::GlobalId(v));
        }
        unimplemented!()
    }
}
