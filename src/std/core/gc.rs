use std::collections::HashMap;

use crate::{
    backend::{
        memory::GarbageCollector,
        types::{
            base::{FSRObject, FSRRetValue, GlobalObj, ObjId},
            fn_def::FSRFn, list::FSRList,
        },
        vm::thread::FSRThreadRuntime,
    }, to_rs_list, utils::error::FSRError
};


pub fn fn_gc_info(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    // thread.garbage_collect.init_size();
    println!("{:#?}", thread.garbage_collect.tracker);
    println!(
        "stw_time: {:?} ms",
        thread.garbage_collect.get_stop_time() / 1000
    );


    Ok(FSRRetValue::GlobalId(FSRObject::none_id()))
}

pub fn fn_gc_collect(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    thread.garbage_collect.clear_marks();
    thread.set_ref_objects_mark(true, &[]);
    thread.collect_gc(true);
    Ok(FSRRetValue::GlobalId(FSRObject::none_id()))
}

pub fn fn_minjor_gc_collect(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    thread.garbage_collect.clear_marks();
    thread.set_ref_objects_mark(false, &[]);
    thread.collect_gc(false);
    Ok(FSRRetValue::GlobalId(FSRObject::none_id()))
}

pub fn fn_gc_shrink(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    thread.garbage_collect.shrink();
    Ok(FSRRetValue::GlobalId(FSRObject::none_id()))
}


pub fn init_gc() -> HashMap<&'static str, FSRObject<'static>> {
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
