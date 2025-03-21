use std::{borrow::Cow, cell::Cell, fmt::{Debug, Formatter}, sync::atomic::{AtomicBool, AtomicU32, AtomicU64}};

use crate::{
    backend::{compiler::bytecode::Bytecode, vm::{thread::{FSRThreadRuntime, ThreadContext}, virtual_machine::FSRVM}},
    utils::error::FSRError,
};

use super::{
    base::{FSRGlobalObjId, FSRObject, FSRRetValue, FSRValue, ObjId},
    class::FSRClass
};

type FSRRustFn = for<'a> fn(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId
) -> Result<FSRRetValue<'a>, FSRError>;

#[derive(Debug, Clone)]
pub struct FSRFnInner<'a> {
    name: Cow<'a, str>,
    fn_ip   : (usize, usize),
    bytecode    : &'a Bytecode
}

impl FSRFnInner<'_> {
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
    RustFn((Cow<'a, str>, FSRRustFn)),
    FSRFn(FSRFnInner<'a>),
}


#[derive(Clone)]
pub struct FSRFn<'a> {
    fn_def: FSRnE<'a>,
    pub(crate) module: ObjId
}

impl Debug for FSRFn<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {:?}>", self.as_str())
    }
}


impl<'a> FSRFn<'a> {
    pub fn as_str(&self) -> String {
        if let FSRnE::RustFn(r) = &self.fn_def {
            return format!("<fn {:?}>", r)
        } else if let FSRnE::FSRFn(f) = &self.fn_def {
            return format!("<fn {:?}>", f.name)
        }

        unimplemented!()
        
    }

    pub fn get_name(&self) -> &Cow<str> {
        if let FSRnE::FSRFn(f) = &self.fn_def {
            return f.get_name()
        } else if let FSRnE::RustFn(f) = &self.fn_def {
            return &Cow::Borrowed("RustFn")
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

    pub fn from_fsr_fn(fn_name: &str, u: (usize, usize), _: Vec<String>, bytecode: &'a Bytecode, m_obj: ObjId) -> FSRValue<'a> {
        let fn_obj = FSRFnInner {
            name: Cow::Owned(fn_name.to_string()),
            fn_ip: u,
            bytecode
        };

        let v = Self {
            fn_def: FSRnE::FSRFn(fn_obj),
            module: m_obj
        };
        FSRValue::Function(Box::new(v))
    }

    pub fn from_rust_fn_static(f: FSRRustFn, name: &'a str) -> FSRObject<'a> {
        let v = Self {
            fn_def: FSRnE::RustFn((Cow::Borrowed(name), f)),
            module: 0
        };
        FSRObject {
            value: FSRValue::Function(Box::new(v)),
            cls: FSRGlobalObjId::FnCls as ObjId,
            ref_count: AtomicU32::new(1),
            delete_flag: AtomicBool::new(true),
            garbage_id: AtomicU32::new(0),
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
        module: ObjId,
    ) -> Result<FSRRetValue<'a>, FSRError> {
        if let FSRnE::RustFn(f) = &self.fn_def {
            return f.1(args, thread, module);
        } else if let FSRnE::FSRFn(f) = &self.fn_def {
            thread.call_frames.push(thread.frame_free_list.new_frame(self.get_name(), module));
            let v = FSRThreadRuntime::call_fn(thread, f, args, self.module)?;
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
