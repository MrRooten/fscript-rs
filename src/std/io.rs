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
    module: Option<ObjId>
) -> Result<FSRRetValue<'a>, FSRError> {
    let value = FSRObject::id_to_obj(args[0]);
    let obj = value.to_string(thread);
    if let FSRValue::String(s) = &obj.value {
        print!("{}", s);
    }
    return Ok(FSRRetValue::GlobalId(0));
}

pub fn fsr_fn_println<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: Option<ObjId>
) -> Result<FSRRetValue<'a>, FSRError> {
    let value = FSRObject::id_to_obj(args[0]);
    let obj = value.to_string(thread);
    if let FSRValue::String(s) = &obj.value {
        println!("{}", s);
    }
    return Ok(FSRRetValue::GlobalId(0));
}

pub fn fsr_fn_dump<'a>(
    args: &[ObjId],
    _thread: &mut FSRThreadRuntime<'a>,
    module: Option<ObjId>
) -> Result<FSRRetValue<'a>, FSRError> {
    let value = FSRObject::id_to_obj(args[0]);
    println!("{:#?}", value);
    return Ok(FSRRetValue::GlobalId(0));
}

pub fn fsr_fn_format<'a>(
    args: &[ObjId],
    _thread: &mut FSRThreadRuntime<'a>,
    module: Option<ObjId>
) -> Result<FSRRetValue<'a>, FSRError> {
    
    let value = FSRObject::id_to_obj(args[0]);
    println!("{:#?}", value);
    return Ok(FSRRetValue::GlobalId(0));
}

pub fn init_io<'a>() -> HashMap<&'static str, FSRObject<'a>> {
    let print_fn = FSRFn::from_rust_fn(fsr_fn_print, "print");
    let println_fn = FSRFn::from_rust_fn(fsr_fn_println, "println");
    let dump_fn = FSRFn::from_rust_fn(fsr_fn_dump, "dump");
    let mut m = HashMap::new();
    m.insert("print", print_fn);
    m.insert("println", println_fn);
    m.insert("dump", dump_fn);
    m
}

