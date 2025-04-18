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
) -> Result<FSRRetValue<'a>, FSRError> {
    print!(
        "gc_info_track: {}, ",
        thread.garbage_collect.get_object_count()
    );

    print!("gc_speed: {:.2}/ms, ", thread.garbage_collect.get_speed());
    print!("gc_collect_count: {}, ", thread.garbage_collect.get_collect_count());
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
) -> Result<FSRRetValue<'a>, FSRError> {
    thread.garbage_collect.clear_marks();
    thread.set_ref_objects_mark(true);
    thread.collect_gc(true);
    Ok(FSRRetValue::GlobalId(0))
}

pub fn fn_minjor_gc_collect<'a>(
    _args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    _module: ObjId,
) -> Result<FSRRetValue<'a>, FSRError> {
    thread.garbage_collect.clear_marks();
    thread.set_ref_objects_mark(false);
    thread.collect_gc(false);
    Ok(FSRRetValue::GlobalId(0))
}

pub fn fn_gc_shrink<'a>(
    _args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    _module: ObjId,
) -> Result<FSRRetValue<'a>, FSRError> {
    thread.garbage_collect.shrink();
    Ok(FSRRetValue::GlobalId(0))
}

pub fn fn_gc_referers<'a>(
    __args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    __module: ObjId,
) -> Result<FSRRetValue<'a>, FSRError> {
    thread.clear_marks();
    let referers = thread.get_referers();
    let object = FSRList::new_value(referers);
    let ret = thread.garbage_collect.new_object(object, FSRGlobalObjId::ListCls as ObjId);
    //println!("referers: {:?}", referers);
    Ok(FSRRetValue::GlobalId(ret))
}

pub fn init_gc<'a>() -> HashMap<&'static str, FSRObject<'a>> {
    let gc_info = FSRFn::from_rust_fn_static(fn_gc_info, "gc_info");
    let gc_collect = FSRFn::from_rust_fn_static(fn_gc_collect, "gc_collect");
    let gc_minjor_collect = FSRFn::from_rust_fn_static(fn_minjor_gc_collect, "gc_minjor_collect");
    let gc_referers = FSRFn::from_rust_fn_static(fn_gc_referers, "gc_referers");
    let gc_shrink = FSRFn::from_rust_fn_static(fn_gc_shrink, "gc_shrink");
    let mut m = HashMap::new();
    m.insert("gc_info", gc_info);
    m.insert("gc_collect", gc_collect);
    m.insert("gc_referers", gc_referers);
    m.insert("gc_minjor_collect", gc_minjor_collect);
    m.insert("gc_shrink", gc_shrink);
    m
}
