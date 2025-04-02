use std::collections::HashMap;

use crate::{
    backend::{
        memory::GarbageCollector,
        types::{
            base::{FSRObject, FSRRetValue, FSRValue, ObjId},
            code::FSRCode,
            fn_def::FSRFn,
        },
        vm::thread::FSRThreadRuntime,
    },
    utils::error::FSRError,
};

use super::utils::{fsr_fn_assert, fsr_fn_export};

pub fn fn_gc_info<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId,
) -> Result<FSRRetValue<'a>, FSRError> {
    print!(
        "gc_info_track: {}, ",
        thread.garbage_collect.get_object_count()
    );

    print!("gc_speed: {:.2}/ms, ", thread.garbage_collect.get_speed());
    println!(
        "stw_time: {:?} ms",
        thread.garbage_collect.get_stop_time() / 1000
    );
    Ok(FSRRetValue::GlobalId(0))
}

pub fn fn_gc_collect<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId,
) -> Result<FSRRetValue<'a>, FSRError> {
    let mut other = thread.flow_tracker.for_iter_obj.clone();
    other.extend(thread.flow_tracker.ref_for_obj.clone());
    other.extend(thread.flow_tracker.iter_objects.clone());
    thread
        .garbage_collect
        .collect(&thread.call_frames, &thread.cur_frame, &other);
    Ok(FSRRetValue::GlobalId(0))
}

pub fn init_gc<'a>() -> HashMap<&'static str, FSRObject<'a>> {
    let gc_info = FSRFn::from_rust_fn_static(fn_gc_info, "gc_info");
    let gc_collect = FSRFn::from_rust_fn_static(fn_gc_collect, "gc_collect");
    let mut m = HashMap::new();
    m.insert("gc_info", gc_info);
    m.insert("gc_collect", gc_collect);
    m
}
