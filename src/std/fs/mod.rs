pub mod file;
use crate::{backend::{types::{base::{FSRObject, FSRValue}, module::FSRModule}, vm::{thread::FSRThreadRuntime, virtual_machine::get_object_by_global_id}}, std::fs::file::FSRInnerFile};

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
        FSRValue::Module(Box::new(module))
    }
}