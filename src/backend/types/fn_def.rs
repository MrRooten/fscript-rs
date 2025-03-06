use std::{borrow::Cow, cell::Cell, sync::atomic::{AtomicU32, AtomicU64}};

use crate::{
    backend::{compiler::bytecode::Bytecode, vm::{runtime::FSRVM, thread::FSRThreadRuntime}},
    utils::error::FSRError,
};

use super::{
    base::{FSRGlobalObjId, FSRObject, FSRRetValue, FSRValue, ObjId},
    class::FSRClass
};

type FSRRustFn = for<'a> fn(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: Option<ObjId>
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
    pub(crate) module: ObjId
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

    pub fn from_fsr_fn(module: &str, u: (usize, usize), _: Vec<String>, bytecode: &'a Bytecode, m_obj: ObjId) -> FSRObject<'a> {
        let fn_obj = FSRFnInner {
            name: Cow::Owned(module.to_string()),
            fn_ip: u,
            bytecode
        };

        let v = Self {
            fn_def: FSRnE::FSRFn(fn_obj),
            module: m_obj
        };
        FSRObject {
            value: FSRValue::Function(Box::new(v)),
            cls: FSRGlobalObjId::FnCls as ObjId,
            ref_count: AtomicU32::new(0),
            delete_flag: Cell::new(true),
            leak: Cell::new(false),
            garbage_id: Cell::new(0),
        }
    }

    pub fn from_rust_fn(f: FSRRustFn) -> FSRObject<'a> {
        let v = Self {
            fn_def: FSRnE::RustFn(f),
            module: 0
        };
        FSRObject {
            value: FSRValue::Function(Box::new(v)),
            cls: FSRGlobalObjId::FnCls as ObjId,
            ref_count: AtomicU32::new(0),
            delete_flag: Cell::new(true),
            leak: Cell::new(false),
            garbage_id: Cell::new(0),
        }
    }

    pub fn get_class() -> FSRClass<'static> {
        FSRClass::new("Fn")
    }

    #[inline(always)]
    pub fn invoke(
        &'a self,
        args: &[ObjId],
        thread: &mut FSRThreadRuntime<'a>,
        module: Option<ObjId>,
    ) -> Result<FSRRetValue<'a>, FSRError> {
        if let FSRnE::RustFn(f) = &self.fn_def {
            return f(args, thread, module);
        } else if let FSRnE::FSRFn(f) = &self.fn_def {
            let v = FSRThreadRuntime::call_fn(thread, f, args, Some(self.module))?;
            let v = match v {
                crate::backend::vm::thread::SValue::Global(g) => g,
                crate::backend::vm::thread::SValue::BoxObject(o) => {
                    FSRVM::leak_object(o)
                },
                _ => unimplemented!()
            };
            return Ok(FSRRetValue::GlobalId(v));
        }
        unimplemented!()
    }
}
