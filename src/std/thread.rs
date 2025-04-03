use std::{collections::HashMap, sync::Mutex};

use crate::{
    backend::{
        memory::GarbageCollector,
        types::{
            base::{FSRGlobalObjId, FSRObject, FSRRetValue, FSRValue, ObjId},
            code::FSRCode,
            fn_def::FSRFn,
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
) -> Result<FSRRetValue<'a>, FSRError> {
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
) -> Result<FSRRetValue<'a>, FSRError> {
    let fn_id = args[0];
    let args = &args[1..];
    let vm = thread.get_vm();
    
    let runtime = Mutex::new(FSRThreadRuntime::new(vm.clone()));
    let thread_id = vm.add_thread(runtime);
    let obj = thread.garbage_collect.new_object(
        FSRValue::Integer(thread_id as i64),
        FSRGlobalObjId::IntegerCls as ObjId,
    );
    let th = std::thread::spawn(move || {
        //vm
        // vm.get_thread(thread_id, |f| {
        //     let fn_def = FSRObject::id_to_obj(args[0]);
        //     fn_def.call(args, f, module, fn_id).unwrap();
        // })
        // .unwrap();
    });

    
    Ok(FSRRetValue::GlobalId(obj))
}

pub fn init_thread<'a>() -> HashMap<&'static str, FSRObject<'a>> {
    let print_fn = FSRFn::from_rust_fn_static(fsr_get_cur_thread_id, "__get_cur_thread_id");
    let mut m = HashMap::new();
    m.insert("__get_cur_thread_id", print_fn);
    m
}
