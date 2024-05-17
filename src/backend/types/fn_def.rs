use std::{borrow::Cow, sync::atomic::AtomicU64};

use crate::{
    backend::{compiler::bytecode::Bytecode, vm::{runtime::FSRVM, thread::FSRThreadRuntime}},
    utils::error::FSRError,
};

use super::{
    base::{FSRObject, FSRRetValue, FSRValue},
    class::FSRClass,
};

type FSRRustFn = for<'a> fn(
    args: Vec<u64>,
    thread: &mut FSRThreadRuntime<'a>,
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
            cls: "Fn",
            ref_count: AtomicU64::new(0),
        }
    }

    pub fn from_rust_fn(f: FSRRustFn) -> FSRObject<'a> {
        let v = Self {
            fn_def: FSRnE::RustFn(f),
        };
        FSRObject {
            obj_id: 0,
            value: FSRValue::Function(v),
            cls: "Fn",
            ref_count: AtomicU64::new(0),
        }
    }

    pub fn get_class(_: &mut FSRVM) -> FSRClass<'static> {
        unimplemented!()
    }

    pub fn invoke(
        &'a self,
        args: Vec<u64>,
        thread: &mut FSRThreadRuntime<'a>,
    ) -> Result<FSRRetValue<'a>, FSRError> {
        if let FSRnE::RustFn(f) = &self.fn_def {
            return f(args, thread);
        }

        if let FSRnE::FSRFn(f) = &self.fn_def {
            let v = FSRThreadRuntime::call_fn(thread, f, args)?;
            return Ok(FSRRetValue::GlobalId(v));
        }
        unimplemented!()
    }
}
