pub mod file;
use crate::{backend::{types::{base::{FSRObject, FSRValue}, fn_def::FSRFn, module::FSRModule}, vm::{thread::FSRThreadRuntime, virtual_machine::get_object_by_global_id}}, std::fs::file::{fsr_fn_is_dir, fsr_fn_is_file, FSRInnerFile}};

pub struct FSRFileSystem {

}



impl FSRFileSystem {
    pub fn new_module(thread: &mut FSRThreadRuntime) -> FSRValue<'static> {
        let mut module = FSRModule::new_module("fs");
        let value = FSRValue::Class(Box::new(FSRInnerFile::get_class()));
        let object_id = thread.garbage_collect.new_object(value, get_object_by_global_id(crate::backend::types::base::GlobalObj::ClassCls));
        if let FSRValue::Class(c) = &mut FSRObject::id_to_mut_obj(object_id).unwrap().value {
            c.set_object_id(object_id);
        }
        module.register_object("File", object_id);
        let is_file = FSRFn::from_rust_fn_static_value(fsr_fn_is_file, "is_file");
        module.register_object("is_file", thread.garbage_collect.new_object(is_file, crate::backend::types::base::GlobalObj::FnCls.get_id()));
        let is_dir = FSRFn::from_rust_fn_static_value(fsr_fn_is_dir, "is_dir");
        module.register_object("is_dir", thread.garbage_collect.new_object(is_dir, crate::backend::types::base::GlobalObj::FnCls.get_id()));
        FSRValue::Module(Box::new(module))
    }
}