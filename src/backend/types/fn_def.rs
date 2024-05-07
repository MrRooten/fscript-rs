use std::{rc::Rc, sync::atomic::AtomicU64};

use crate::{
    backend::vm::{runtime::FSRVM, thread::CallState},
    utils::error::FSRError,
};

use super::{
    base::{FSRObject, FSRRetValue, FSRValue},
    class::FSRClass,
};

type FSRRustFn = for<'a> fn(
    args: Vec<u64>,
    stack: &mut CallState,
    vm: &FSRVM<'a>,
) -> Result<FSRRetValue<'a>, FSRError>;
#[derive(Debug, Clone)]
pub enum FSRnE {
    RustFn(FSRRustFn),
    FSRFn((Rc<String>, (u64, u64))),
}

#[derive(Debug, Clone)]
pub struct FSRFn {
    fn_def: FSRnE,
}

impl<'a> FSRFn {
    pub fn get_def(&self) -> &FSRnE {
        &self.fn_def
    }

    pub fn get_args(&self) -> &Vec<String> {
        unimplemented!()
    }

    pub fn from_fsr_fn(module: &str, u: (u64, u64), _: Vec<String>) -> FSRObject<'static> {
        let v = Self {
            fn_def: FSRnE::FSRFn((Rc::new(module.to_string()), u)),
        };
        FSRObject {
            obj_id: 0,
            value: FSRValue::Function(v),
            cls: "Fn",
            ref_count: AtomicU64::new(0),
        }
    }

    pub fn from_rust_fn(f: FSRRustFn) -> FSRObject<'static> {
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
        &self,
        args: Vec<u64>,
        stack: &mut CallState,
        vm: &FSRVM<'a>,
    ) -> Result<FSRRetValue<'a>, FSRError> {
        if let FSRnE::RustFn(f) = &self.fn_def {
            return f(args, stack, vm);
        }

        unimplemented!()
    }
}
