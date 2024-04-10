use std::{cell::Ref, collections::{HashMap, LinkedList}};

use crate::backend::{compiler::bytecode::BytecodeArg, vm::{runtime::FSRVM, thread::CallState}};

use super::{base::{FSRObject, FSRValue}, class::FSRClass};


type FSRRustFn = for<'a> fn(args: Vec<Ref<FSRObject<'a>>>, stack: &'a mut CallState, vm: &FSRVM<'a>) -> Result<FSRObject<'a>, ()>;
#[derive(Debug, Clone)]
pub enum FSRnE {
    RustFn(FSRRustFn),
    FSRFn(u64)
}

#[derive(Debug, Clone)]
pub struct FSRFn {
    fn_def      : FSRnE
}

impl FSRFn {
    pub fn from_fsr_fn(u: u64) -> FSRObject<'static> {
        let v = Self {
            fn_def: FSRnE::FSRFn(u)
        };
        FSRObject {
            obj_id: 0,
            value: FSRValue::Function(v),
            cls: "Fn",
        }
    }

    pub fn from_rust_fn(f: FSRRustFn) -> FSRObject<'static> {
        let v = Self {
            fn_def: FSRnE::RustFn(f)
        };
        FSRObject {
            obj_id: 0,
            value: FSRValue::Function(v),
            cls: "Fn",
        }
    }

    pub fn get_class(vm: &mut FSRVM) -> FSRClass<'static> {
        unimplemented!()
    }

    pub fn invoke<'a>(&self, args: Vec<Ref<FSRObject<'a>>>, stack: &'a mut CallState, vm: &FSRVM<'a>) -> Result<FSRObject<'a>,()> {
        if let FSRnE::RustFn(f) = &self.fn_def {
            return f(args, stack, vm);
        }

        unimplemented!()
    }
}