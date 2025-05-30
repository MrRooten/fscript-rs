use std::collections::HashMap;

use crate::{
    backend::{
        memory::GarbageCollector,
        types::{
            any::FSRThreadHandle, base::{FSRGlobalObjId, FSRObject, FSRRetValue, FSRValue, ObjId}, fn_def::FSRFn
        },
        vm::{thread::FSRThreadRuntime, virtual_machine::get_object_by_global_id},
    },
    utils::error::FSRError,
};

pub fn fsr_get_cur_thread_id(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let id = thread.get_thread_id();
    let obj = thread.garbage_collect.new_object(
        FSRValue::Integer(id as i64),
        get_object_by_global_id(FSRGlobalObjId::IntegerCls),
    );
    Ok(FSRRetValue::GlobalId(obj))
}

pub fn fsr_new_thread(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let fn_id = args[0];
    let args = args[1..].to_vec();
    
    let vm = thread.get_vm();
    let th = std::thread::spawn(move || {
        
        let runtime = FSRThreadRuntime::new_runtime();
        let thread_id = vm.add_thread(runtime);
        let th = vm.get_thread(thread_id).unwrap();
        let fn_obj = FSRObject::id_to_obj(fn_id);
        let _ = fn_obj.call(&args, th, code, fn_id);
        // vm2.get_thread(thread_id, |x| {
        //     let fn_obj = FSRObject::id_to_obj(fn_id);
        //     fn_obj.call(&args, x, code, fn_id);
        // });
    });
    let handle = FSRThreadHandle::new(th);
    
    let thread_obj = thread.garbage_collect.new_object(handle.to_any_type(), get_object_by_global_id(FSRGlobalObjId::ThreadCls) as ObjId);
    
    Ok(FSRRetValue::GlobalId(thread_obj))
}

pub fn init_thread<'a>() -> HashMap<&'static str, FSRObject<'a>> {
    let get_cur_thread_id_fn = FSRFn::from_rust_fn_static(fsr_get_cur_thread_id, "__get_cur_thread_id");
    let new_thread_fn = FSRFn::from_rust_fn_static(fsr_new_thread, "__new_thread");
    let mut m = HashMap::new();
    m.insert("__get_cur_thread_id", get_cur_thread_id_fn);
    m.insert("__new_thread", new_thread_fn);
    m
}
