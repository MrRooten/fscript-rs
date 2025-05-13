use std::collections::HashMap;

use crate::{
    backend::{
        memory::GarbageCollector,
        types::{
            base::{FSRGlobalObjId, FSRObject, FSRRetValue, ObjId},
            fn_def::FSRFn, list::FSRList,
        },
        vm::thread::FSRThreadRuntime,
    },
    utils::error::FSRError,
};


pub fn fn_gc_info<'a>(
    _args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    _module: ObjId,
) -> Result<FSRRetValue, FSRError> {
    // thread.garbage_collect.init_size();
    println!("{:#?}", thread.garbage_collect.tracker);
    println!(
        "stw_time: {:?} ms",
        thread.garbage_collect.get_stop_time() / 1000
    );


    Ok(FSRRetValue::GlobalId(0))
}

pub fn fn_gc_collect<'a>(
    _args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    _module: ObjId,
) -> Result<FSRRetValue, FSRError> {
    thread.garbage_collect.clear_marks();
    thread.set_ref_objects_mark(true);
    thread.collect_gc(true);
    Ok(FSRRetValue::GlobalId(0))
}

pub fn fn_minjor_gc_collect<'a>(
    _args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    _module: ObjId,
) -> Result<FSRRetValue, FSRError> {
    thread.garbage_collect.clear_marks();
    thread.set_ref_objects_mark(false);
    thread.collect_gc(false);
    Ok(FSRRetValue::GlobalId(0))
}

pub fn fn_gc_shrink<'a>(
    _args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    _module: ObjId,
) -> Result<FSRRetValue, FSRError> {
    thread.garbage_collect.shrink();
    Ok(FSRRetValue::GlobalId(0))
}


pub fn init_gc<'a>() -> HashMap<&'static str, FSRObject<'a>> {
    let gc_info = FSRFn::from_rust_fn_static(fn_gc_info, "gc_info");
    let gc_collect = FSRFn::from_rust_fn_static(fn_gc_collect, "gc_collect");
    let gc_minjor_collect = FSRFn::from_rust_fn_static(fn_minjor_gc_collect, "gc_minjor_collect");
    let gc_shrink = FSRFn::from_rust_fn_static(fn_gc_shrink, "gc_shrink");
    let mut m = HashMap::new();
    m.insert("gc_info", gc_info);
    m.insert("gc_collect", gc_collect);
    m.insert("gc_minjor_collect", gc_minjor_collect);
    m.insert("gc_shrink", gc_shrink);
    m
}
