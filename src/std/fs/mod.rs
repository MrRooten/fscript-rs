pub mod file;
pub mod dir;
use crate::{
    backend::{
        types::{
            base::{FSRObject, FSRValue},
            fn_def::FSRFn,
            module::FSRModule,
        },
        vm::{thread::FSRThreadRuntime, virtual_machine::get_object_by_global_id},
    },
    register_class, register_fn,
    std::fs::{dir::FSRDir, file::{fsr_fn_is_dir, fsr_fn_is_file, FSRInnerFile}},
};

pub struct FSRFileSystem {}

impl FSRFileSystem {
    pub fn new_module(thread: &mut FSRThreadRuntime) -> FSRValue<'static> {
        let mut module = FSRModule::new_module("fs");
        register_class!(module, thread, "File", FSRInnerFile::get_class());
        register_class!(module, thread, "Dir", FSRDir::get_class());
        register_fn!(module, thread, "is_file", fsr_fn_is_file);
        register_fn!(module, thread, "is_dir", fsr_fn_is_dir);

        FSRValue::Module(Box::new(module))
    }
}
