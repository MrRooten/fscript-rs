use crate::backend::{types::{base::{FSRObject, ObjId}, ext}, vm::thread::{CallFrame, FSRThreadRuntime}};

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

pub extern "C" fn call_fn(args: *const ObjId, len: usize, fn_id: ObjId, thread: &mut FSRThreadRuntime, code: ObjId) -> ObjId {
    let obj = FSRObject::id_to_obj(fn_id);
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let res = obj.call(args, thread, code, fn_id);
    res.unwrap().get_id()
}

pub extern "C" fn malloc(size: usize) -> *mut Vec<ObjId> {
    // Allocate a vector of ObjId with the specified size
    let vec = Vec::with_capacity(size);
    // Return a raw pointer to the vector
    Box::leak(Box::new(vec))
}

pub extern "C" fn free(ptr: *mut Vec<ObjId>) {
    // Convert the raw pointer back to a Box and drop it
    if !ptr.is_null() {
        unsafe {
            Box::from_raw(ptr);
        }
    }
}

pub extern "C" fn get_obj_by_name(name: *const u8, len: usize, thread: &mut FSRThreadRuntime) -> ObjId {
    let name_slice = unsafe { std::slice::from_raw_parts(name, len) };
    let name_str = std::str::from_utf8(name_slice).unwrap();
    let obj = FSRThreadRuntime::get_chain_by_name(thread, name_str).unwrap();
    obj
}