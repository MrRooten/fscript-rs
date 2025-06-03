use crate::backend::{
    compiler::bytecode::{BinaryOffset, CompareOperator},
    memory::GarbageCollector,
    types::{
        base::{FSRObject, ObjId},
        ext,
    },
    vm::thread::{CallFrame, FSRThreadRuntime, GcState},
};

macro_rules! obj_cls {
    ($a:expr) => {
        FSRObject::id_to_obj(FSRObject::id_to_obj($a).cls).as_class()
    };
}

pub extern "C" fn get_constant(code: ObjId, index: u64) -> ObjId {
    let module = FSRObject::id_to_obj(code).as_code().module;
    let module = FSRObject::id_to_obj(module).as_module();
    let constant = module.get_const(index as usize).unwrap();
    constant
}

pub extern "C" fn is_true(obj: ObjId) -> bool {
    obj == FSRObject::true_id()
}

pub extern "C" fn is_false(obj: ObjId) -> bool {
    obj == FSRObject::false_id()
}

pub extern "C" fn is_none(obj: ObjId) -> bool {
    obj == FSRObject::none_id()
}

pub extern "C" fn get_none() -> ObjId {
    FSRObject::none_id()
}

pub extern "C" fn call_fn(
    args: *const ObjId,
    len: usize,
    fn_id: ObjId,
    thread: &mut FSRThreadRuntime,
    code: ObjId,
) -> ObjId {
    let obj = FSRObject::id_to_obj(fn_id);
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let res = obj.call(args, thread, code, fn_id);
    res.unwrap().get_id()
}

pub extern "C" fn malloc(size: usize) -> *mut ObjId {
    let layout = std::alloc::Layout::array::<ObjId>(size).unwrap();
    unsafe {
        let ptr = std::alloc::alloc(layout) as *mut ObjId;
        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        ptr
    }
}

pub extern "C" fn free(ptr: *mut ObjId, size: usize) {
    // Convert the raw pointer back to a Box and drop it
    if !ptr.is_null() {
        let layout = std::alloc::Layout::array::<ObjId>(size).unwrap();
        unsafe {
            std::alloc::dealloc(ptr as *mut u8, layout);
        }
    }
}

pub extern "C" fn get_obj_by_name(
    name: *const u8,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> ObjId {
    let name_slice = unsafe { std::slice::from_raw_parts(name, len) };
    let name_str = std::str::from_utf8(name_slice).unwrap();
    let obj = FSRThreadRuntime::get_chain_by_name(thread, name_str).unwrap();
    obj
}

pub extern "C" fn check_gc(thread: &mut FSRThreadRuntime) -> bool {
    thread.garbage_collect.will_collect()
}

pub extern "C" fn gc_collect(thread: &mut FSRThreadRuntime, list_obj: *const ObjId, len: usize) {
    let list = unsafe { std::slice::from_raw_parts(list_obj, len) };

    let st = std::time::Instant::now();
    //if thread.gc_context.gc_state == GcState::Stop {
    thread.clear_marks();
    //}
    thread.set_ref_objects_mark(false, list);
    //if thread.gc_context.worklist.is_empty() {
    thread.collect_gc(false);
    //thread.gc_context.gc_state = GcState::Stop;
    //}

    thread.garbage_collect.tracker.collect_time += st.elapsed().as_micros() as u64;
}

pub extern "C" fn compare_test(
    thread: &mut FSRThreadRuntime,
    left: ObjId,
    right: ObjId,
    op: CompareOperator,
) -> bool {
    FSRThreadRuntime::compare(left, right, op, thread).unwrap()
}

/// Perform a binary operation on two objects.
/// # Arguments
/// * `left` - The left operand object ID.
/// * `right` - The right operand object ID.
/// * `op` - The binary operation to perform, represented by a `BinaryOffset`.
/// * `thread` - The current thread runtime.
/// # Returns
/// The result of the binary operation as an `ObjId`.
pub extern "C" fn binary_op(
    left: ObjId,
    right: ObjId,
    op: BinaryOffset,
    thread: &mut FSRThreadRuntime,
) -> ObjId {
    let args = [left, right];
    let len = args.len();
    if let Some(rust_fn) = obj_cls!(left).get_rust_fn(op) {
        return rust_fn(args.as_ptr(), len, thread, 0).unwrap().get_id();
    }

    unimplemented!("binary op {:?} not support in rust fn", op);
}

/// Get the attribute of an object by name.
/// # Arguments
/// * `obj` - The object ID from which to get the attribute.
/// * `name` - A pointer to the attribute name as a byte slice.
/// * `len` - The length of the attribute name byte slice.
/// * `thread` - The current thread runtime.
/// # Returns
/// The object ID of the attribute if it exists, or `FSRObject::none_id()` if it does not.
pub extern "C" fn get_attr_obj(
    obj: ObjId,
    name: *const u8,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> ObjId {
    let name_slice = unsafe { std::slice::from_raw_parts(name, len) };
    let name_str = std::str::from_utf8(name_slice).unwrap();
    let obj = FSRObject::id_to_obj(obj);
    let attr = obj.get_attr(name_str);
    attr.map(|x| x.load(std::sync::atomic::Ordering::Relaxed))
        .unwrap_or(FSRObject::none_id())
}

pub extern "C" fn get_cur_frame<'a>(thread: &'a mut FSRThreadRuntime<'a>) -> *mut CallFrame<'a> {
    let frame = thread.get_cur_mut_frame();
    frame as *mut CallFrame<'a>
}


/// Get the number of arguments passed to the current function.
/// # Arguments
/// * `thread` - The current thread runtime.
/// * `index` - The index of the argument to retrieve, where 0 is the last argument.
/// # Returns
/// The object ID of the argument at the specified index, or `FSRObject::none_id()` if no arguments exist.
pub extern "C" fn get_n_args(thread: &mut FSRThreadRuntime, index: i32) -> ObjId {
    let frame = thread.get_cur_mut_frame();
    let len = frame.args.len();
    if len == 0 {
        return FSRObject::none_id();
    }
    frame.args.get(len - 1 - index as usize).cloned().unwrap_or(FSRObject::none_id())
}