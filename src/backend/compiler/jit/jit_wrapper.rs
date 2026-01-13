use std::{ops::Range, sync::atomic::Ordering};

use crate::{
    backend::{
        compiler::bytecode::{BinaryOffset, CompareOperator},
        types::{
            base::{FSRObject, FSRValue, GlobalObj, ObjId},
            iterator::next_obj,
            list::FSRList,
            range::FSRRange,
            string::FSRString,
        },
        vm::{
            thread::{CallFrame, FSRThreadRuntime},
            virtual_machine::gid,
        },
    },
    to_rs_list,
};

macro_rules! obj_cls {
    ($a:expr) => {
        FSRObject::id_to_obj($a).cls
    };
}

pub extern "C" fn get_constant(code: ObjId, index: u32) -> ObjId {
    // let module = FSRObject::id_to_obj(code).as_code().module;
    // let module = FSRObject::id_to_obj(module).as_module();
    // let constant = module.get_const(index as usize).unwrap();
    // constant
    unimplemented!()
}

pub extern "C" fn call_fn(
    fn_ptr: *const u8,
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId,
) -> ObjId {
    let call_fn_target = unsafe {
        std::mem::transmute::<_, extern "C" fn(&mut FSRThreadRuntime, ObjId) -> ObjId>(fn_ptr)
    };
    let args = to_rs_list!(args, len);
    for arg in args.iter().cloned() {
        thread
            .get_cur_mut_frame()
            .static_args
            .push(FSRObject::id_to_obj(arg).get_static_value_ptr());
    }
    call_fn_target(thread, code)
}

pub extern "C" fn save_to_exp(args: *const ObjId, len: usize, thread: &mut FSRThreadRuntime) {
    let args = to_rs_list!(args, len);
    let frame = thread.get_cur_mut_frame();
    frame.clear_exp();
    frame.extend_exp(args);
}

pub extern "C" fn clear_exp(thread: &mut FSRThreadRuntime) {
    let frame = thread.get_cur_mut_frame();
    frame.clear_exp();
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

    FSRThreadRuntime::get_chain_by_name(thread, name_str).unwrap()
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
) -> ObjId {
    if FSRThreadRuntime::compare(left, right, op, thread).unwrap() {
        FSRObject::true_id()
    } else {
        FSRObject::false_id()
    }
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
    code: ObjId,
    thread: &mut FSRThreadRuntime,
) -> ObjId {
    let args = [left, right];
    let len = args.len();
    if let Some(rust_fn) = obj_cls!(left).get_rust_fn(op) {
        return rust_fn(args.as_ptr(), len, thread).unwrap().get_id();
    }

    if let Some(op_fn) = obj_cls!(left).get_offset_attr(op) {
        let op_fn = op_fn.load(std::sync::atomic::Ordering::Relaxed);
        let fn_obj = FSRObject::id_to_obj(op_fn);
        let ret = fn_obj.call(&args, thread).unwrap().get_id();
        return ret;
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

pub extern "C" fn get_cur_frame<'a>(thread: &'a mut FSRThreadRuntime<'a>) -> *mut CallFrame {
    let frame = thread.get_cur_mut_frame();
    frame as *mut CallFrame
}

/// Get the number of arguments passed to the current function.
/// # Arguments
/// * `thread` - The current thread runtime.
/// * `index` - The index of the argument to retrieve, where 0 is the last argument.
/// # Returns
/// The object ID of the argument at the specified index, or `FSRObject::none_id()` if no arguments exist.
pub extern "C" fn get_n_args(thread: &mut FSRThreadRuntime, index: i32) -> ObjId {
    let frame = thread.get_cur_mut_frame();
    let len = frame.static_args.len();
    if len == 0 {
        return 0;
    }
    frame.static_args.get(index as usize).cloned().unwrap_or(0)
}

pub extern "C" fn getter(
    container: ObjId,
    index_obj: ObjId,
    thread: &mut FSRThreadRuntime,
) -> ObjId {
    let container_obj = FSRObject::id_to_obj(container);
    let index = FSRObject::id_to_obj(index_obj);

    if let Some(rust_fn) = obj_cls!(container).get_rust_fn(BinaryOffset::GetItem) {
        let list = [container, index_obj];
        return rust_fn(list.as_ptr(), 2, thread).unwrap().get_id();
    }

    unimplemented!()
}

/// Gets an attribute from an object using a name provided as a raw pointer.
///
/// # Safety
/// This function is unsafe because it dereferences a raw pointer to create a slice.
/// The caller must ensure that:
/// - The `name` pointer is valid and points to a properly aligned memory region
/// - The memory region contains at least `len` valid bytes
/// - The memory region is not mutated during the lifetime of the slice
pub unsafe extern "C" fn binary_dot_getter(
    father: ObjId,
    name: *const u8,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> ObjId {
    let name_slice = unsafe { std::slice::from_raw_parts(name, len) };
    let name_str = std::str::from_utf8(name_slice).unwrap();
    let father_obj = FSRObject::id_to_obj(father);

    father_obj
        .get_attr(name_str)
        .unwrap()
        .load(Ordering::Relaxed)
}

pub extern "C" fn load_integer(value: i64, thread: &mut FSRThreadRuntime) -> ObjId {
    let obj = thread
        .garbage_collect
        .new_object(FSRValue::Integer(value), gid(GlobalObj::IntegerCls));
    obj
}

pub extern "C" fn load_string(
    value: *const u8,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> ObjId {
    let value_slice = unsafe { std::slice::from_raw_parts(value, len) };
    let value_str = std::str::from_utf8(value_slice).unwrap();
    let value = FSRString::new_value(value_str);

    thread
        .garbage_collect
        .new_object(value, gid(GlobalObj::StringCls))
}

pub extern "C" fn load_float(value: f64, thread: &mut FSRThreadRuntime) -> ObjId {
    let obj = thread
        .garbage_collect
        .new_object(FSRValue::Float(value), gid(GlobalObj::FloatCls));
    obj
}

pub extern "C" fn c_next_obj(obj: ObjId, thread: &mut FSRThreadRuntime) -> ObjId {
    // let obj = FSRObject::id_to_obj(obj);
    let args = [obj];

    next_obj(args.as_ptr(), 1, thread).unwrap().get_id()
}

pub extern "C" fn get_iter_obj(obj: ObjId, thread: &mut FSRThreadRuntime) -> ObjId {
    let iter_obj = FSRObject::id_to_obj(obj);
    let read_iter_id = match iter_obj.get_attr("__iter__") {
        Some(s) => {
            let iter_fn = s.load(Ordering::Relaxed);
            let iter_fn_obj = FSRObject::id_to_obj(iter_fn);
            let ret = iter_fn_obj.call(&[obj], thread).unwrap();
            ret.get_id()
        }
        None => obj,
    };

    read_iter_id
}

pub extern "C" fn binary_range(left: ObjId, right: ObjId, thread: &mut FSRThreadRuntime) -> ObjId {
    let start = FSRObject::id_to_obj(left);
    let end = FSRObject::id_to_obj(right);

    if let FSRValue::Integer(start) = start.value {
        if let FSRValue::Integer(end) = end.value {
            let range = FSRRange {
                range: Range { start, end },
            };

            let id = thread.garbage_collect.new_object(
                FSRValue::Range(Box::new(range)),
                gid(GlobalObj::RangeCls) as ObjId,
            );

            return id;
        }
    }

    panic!("binary_range only support integer range");
}

pub extern "C" fn get_current_fn_id(thread: &mut FSRThreadRuntime) -> ObjId {
    let frame = thread.get_cur_mut_frame();
    frame.fn_id
}

pub extern "C" fn get_obj_method(father: ObjId, name: *const u8, len: usize) -> ObjId {
    let name_slice = unsafe { std::slice::from_raw_parts(name, len) };
    let name_str = std::str::from_utf8(name_slice).unwrap();
    let father_obj = FSRObject::id_to_obj(father);

    if let Some(attr) = father_obj.cls.get_attr(name_str) {
        return attr.load(Ordering::Relaxed);
    }

    FSRObject::none_id()
}

pub extern "C" fn load_list(
    list_obj: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> ObjId {
    let list = unsafe { std::slice::from_raw_parts(list_obj, len) };
    let list = FSRList::new_value(list.to_vec());

    thread
        .garbage_collect
        .new_object(list, gid(GlobalObj::ListCls))
}

pub extern "C" fn c_println(obj: i64) {
    println!("test_args value: {}", obj);
}

pub extern "C" fn memcpy(
    dest: usize,
    src: usize,
    size: usize,
) {
    unsafe {
        std::ptr::copy_nonoverlapping(
            src as *const u8,
            dest as *mut u8,
            size as usize,
        );
    }
}