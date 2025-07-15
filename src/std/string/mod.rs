use crate::{
    backend::{
        types::{
            base::{FSRObject, FSRRetValue, FSRValue, GlobalObj, ObjId},
            class::FSRClass,
            module::FSRModule,
            string::{fsr_fn_format_string, FSRString},
        },
        vm::thread::FSRThreadRuntime,
    },
    register_fn,
    utils::error::FSRError,
};

pub struct FSRStringModule {}

impl FSRStringModule {
    pub fn new_module(thread: &mut FSRThreadRuntime) -> FSRValue<'static> {
        let mut module = FSRModule::new_module("str");
        register_fn!(module, thread, "format", fsr_fn_format_string);
        FSRValue::Module(Box::new(module))
    }
}
