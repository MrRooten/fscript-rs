use std::collections::HashMap;

use crate::{
    backend::{
        types::{
            base::{FSRObject, FSRRetValue, FSRValue, ObjId},
            fn_def::FSRFn, module::FSRModule,
        },
        vm::thread::FSRThreadRuntime,
    },
    utils::error::FSRError,
};

use super::utils::{fsr_fn_assert, fsr_fn_export};

pub fn fsr_fn_print<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId
) -> Result<FSRRetValue<'a>, FSRError> {
    let value = FSRObject::id_to_obj(args[0]);
    let obj = value.to_string(thread, module);
    if let FSRValue::String(s) = &obj.value {
        print!("{}", s);
    }
    Ok(FSRRetValue::GlobalId(0))
}

pub fn fsr_fn_println<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId
) -> Result<FSRRetValue<'a>, FSRError> {
    let value = FSRObject::id_to_obj(args[0]);
    let obj = value.to_string(thread, module);
    if let FSRValue::String(s) = &obj.value {
        println!("{}", s);
    }
    Ok(FSRRetValue::GlobalId(0))
}

pub fn fsr_fn_dump<'a>(
    args: &[ObjId],
    _thread: &mut FSRThreadRuntime<'a>,
    module: ObjId
) -> Result<FSRRetValue<'a>, FSRError> {
    let value = FSRObject::id_to_obj(args[0]);
    println!("{:#?}", value);
    Ok(FSRRetValue::GlobalId(0))
}

pub fn fsr_fn_format<'a>(
    args: &[ObjId],
    _thread: &mut FSRThreadRuntime<'a>,
    module: ObjId
) -> Result<FSRRetValue<'a>, FSRError> {
    
    let value = FSRObject::id_to_obj(args[0]);
    println!("{:#?}", value);
    Ok(FSRRetValue::GlobalId(0))
}

pub fn fsr_fn_throw_error<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId
) -> Result<FSRRetValue<'a>, FSRError> {
    if args.is_empty() || args[0] == 0 {
        thread.exception = FSRObject::none_id();
    } else {
        thread.exception = args[0];
    }
    Ok(FSRRetValue::GlobalId(0))
}

pub fn fsr_fn_get_error<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId
) -> Result<FSRRetValue<'a>, FSRError> {
    let ret = thread.get_cur_mut_frame().handling_exception;
    Ok(FSRRetValue::GlobalId(ret))
}

pub fn init_io<'a>() -> HashMap<&'static str, FSRObject<'a>> {
    let print_fn = FSRFn::from_rust_fn_static(fsr_fn_print, "print");
    let println_fn = FSRFn::from_rust_fn_static(fsr_fn_println, "println");
    let dump_fn = FSRFn::from_rust_fn_static(fsr_fn_dump, "dump");
    let throw_error = FSRFn::from_rust_fn_static(fsr_fn_throw_error, "throw_error");
    let get_error = FSRFn::from_rust_fn_static(fsr_fn_get_error, "pop_error");
    let mut m = HashMap::new();
    m.insert("print", print_fn);
    m.insert("println", println_fn);
    m.insert("dump", dump_fn);
    m.insert("throw_error", throw_error);
    m.insert("get_error", get_error);
    m
}

