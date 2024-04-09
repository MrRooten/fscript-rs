use std::collections::{HashMap, LinkedList};

use crate::backend::{compiler::bytecode::BytecodeArg, vm::{runtime::FSRVM, thread::CallState}};

use super::{base::{FSRObject, FSRValue}, class::FSRClass};


type FSRRustFn = for<'a> fn(args: Vec<u64>, stack: &'a mut CallState, vm: &'a mut FSRVM<'a>) -> Result<u64, ()>;
#[derive(Debug)]
pub enum FSRnE {
    RustFn(FSRRustFn),
    FSRFn(u64)
}

#[derive(Debug)]
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
            attrs: HashMap::new(),
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
            attrs: HashMap::new(),
        }
    }

    pub fn get_class(vm: &mut FSRVM) -> FSRClass<'static> {
        unimplemented!()
    }

    pub fn invoke<'a>(&self, args: Vec<u64>, stack: &'a mut CallState, vm: &'a mut FSRVM<'a>) -> Result<u64,()> {
        if let FSRnE::RustFn(f) = &self.fn_def {
            return f(args, stack, vm);
        }

        unimplemented!()
    }
}