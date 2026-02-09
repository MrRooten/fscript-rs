use std::collections::HashMap;

use crate::{
    backend::{
        types::{
            base::{FSRObject, FSRRetValue, FSRValue, ObjId},
            fn_def::FSRFn, module::FSRModule,
        },
        vm::thread::FSRThreadRuntime,
    }, register_fn, to_rs_list, utils::error::FSRError
};


pub fn fn_time_timestamp(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    // thread.garbage_collect.init_size();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let obj_id = thread.garbage_collect.get_integer(timestamp);
    Ok(FSRRetValue::GlobalId(obj_id))
}

pub fn fn_time_timestamp_ms(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    // thread.garbage_collect.init_size();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    let obj_id = thread.garbage_collect.get_integer(timestamp);
    Ok(FSRRetValue::GlobalId(obj_id))
}

pub struct Time {}

impl Time {
    pub fn new_module(thread: &mut FSRThreadRuntime) -> FSRValue<'static> {
        let mut module = FSRModule::new_module("time");
        // register_class!(module, thread, "File", FSRInnerFile::get_class());
        // register_class!(module, thread, "Dir", FSRDir::get_class());
        // register_fn!(module, thread, "is_file", fsr_fn_is_file);
        //register_fn!(module, thread, "is_dir", fsr_fn_is_dir);
        register_fn!(module, thread, "timestamp", fn_time_timestamp);
        register_fn!(module, thread, "timestamp_ms", fn_time_timestamp_ms);
        FSRValue::Module(Box::new(module))
    }
}