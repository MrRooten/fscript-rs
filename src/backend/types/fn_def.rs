use std::collections::LinkedList;

use crate::backend::compiler::bytecode::BytecodeArg;


type FSRRustFn = fn();
pub enum FSRnE {
    RustFn(FSRRustFn),
    FSRFn(u64)
}
pub struct FSRFn {
    fn_def      : FSRnE
}

impl FSRFn {
    pub fn from_fsr_fn(u: u64) -> Self {
        Self {
            fn_def: FSRnE::FSRFn(u)
        }
    }

    pub fn from_rust_fn(f: FSRRustFn) -> Self {
        Self {
            fn_def: FSRnE::RustFn(f)
        }
    }

    
}