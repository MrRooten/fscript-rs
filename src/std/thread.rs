use std::{collections::HashMap, sync::{Arc, Mutex}};

use crate::{
    backend::{
        memory::GarbageCollector,
        types::{
            any::FSRThreadHandle, base::{FSRGlobalObjId, FSRObject, FSRRetValue, FSRValue, ObjId}, code::FSRCode, fn_def::FSRFn
        },
        vm::thread::FSRThreadRuntime,
    },
    utils::error::FSRError,
};

use super::utils::{fsr_fn_assert, fsr_fn_export};

pub fn fsr_get_cur_thread_id<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId,
) -> Result<FSRRetValue, FSRError> {
    let id = thread.get_thread_id();
    let obj = thread.garbage_collect.new_object(
        FSRValue::Integer(id as i64),
        FSRGlobalObjId::IntegerCls as ObjId,
    );
    Ok(FSRRetValue::GlobalId(obj))
}

pub fn fsr_new_thread<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId,
) -> Result<FSRRetValue, FSRError> {
    let fn_id = args[0];
    let args = args[1..].to_vec();
    
    let vm = thread.get_vm();
    let vm2 = vm.clone();
    let th = std::thread::spawn(move || {
        
        let runtime = Mutex::new(FSRThreadRuntime::new());
        let thread_id = vm.add_thread(Arc::new(runtime));
        let th = vm.get_thread(thread_id).unwrap();
        let fn_obj = FSRObject::id_to_obj(fn_id);
        fn_obj.call(&args, &mut th.lock().unwrap(), module, fn_id);
        // vm2.get_thread(thread_id, |x| {
        //     let fn_obj = FSRObject::id_to_obj(fn_id);
        //     fn_obj.call(&args, x, module, fn_id);
        // });
    });
    let handle = FSRThreadHandle::new(th);
    
    let thread_obj = thread.garbage_collect.new_object(handle.to_any_type(), FSRGlobalObjId::ThreadCls as ObjId);
    
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
