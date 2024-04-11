use std::{cell::Ref, collections::{HashMap, LinkedList}, sync::atomic::AtomicU64};

use crate::backend::{compiler::bytecode::BytecodeArg, vm::{runtime::FSRVM, thread::CallState}};

use super::{base::{FSRObject, FSRValue}, class::FSRClass};


type FSRRustFn = for<'a> fn(args: Vec<Ref<FSRObject<'a>>>, stack: &mut CallState, vm: &FSRVM<'a>) -> Result<FSRObject<'a>, ()>;
#[derive(Debug, Clone)]
pub enum FSRnE {
    RustFn(FSRRustFn),
    FSRFn(u64)
}

#[derive(Debug, Clone)]
pub struct FSRFn {
    fn_def      : FSRnE,
    args        : Vec<String>
}

impl<'a> FSRFn {
    pub fn get_def(&self) -> &FSRnE {
        return &self.fn_def
    }

    pub fn get_args(&self) -> &Vec<String> {
        unimplemented!()
    }

    pub fn from_fsr_fn(u: u64, args: Vec<String>) -> FSRObject<'static> {
        let v = Self {
            fn_def: FSRnE::FSRFn(u),
            args: args,
        };
        FSRObject {
            obj_id: 0,
            value: FSRValue::Function(v),
            cls: "Fn",
            ref_count: AtomicU64::new(0)
        }
    }

    pub fn from_rust_fn(f: FSRRustFn) -> FSRObject<'static> {
        let v = Self {
            fn_def: FSRnE::RustFn(f),
            args: vec![],
        };
        FSRObject {
            obj_id: 0,
            value: FSRValue::Function(v),
            cls: "Fn",
            ref_count: AtomicU64::new(0)
        }
    }

    pub fn get_class(vm: &mut FSRVM) -> FSRClass<'static> {
        unimplemented!()
    }

    pub fn invoke(&self, args: Vec<Ref<FSRObject<'a>>>, stack: &mut CallState, vm: &FSRVM<'a>) -> Result<FSRObject<'a>,()> {
        if let FSRnE::RustFn(f) = &self.fn_def {
            return f(args, stack, vm);
        }

        unimplemented!()
    }
}