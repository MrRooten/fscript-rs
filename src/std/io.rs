use std::collections::HashMap;

use crate::{
    backend::{
        types::{
            base::{FSRObject, FSRRetValue, FSRValue},
            fn_def::FSRFn,
        },
        vm::thread::FSRThreadRuntime,
    },
    utils::error::FSRError,
};

pub fn fsr_fn_print<'a>(
    args: &[u64],
    thread: &mut FSRThreadRuntime<'a>
) -> Result<FSRRetValue<'a>, FSRError> {
    let value = FSRObject::id_to_obj(args[0]);
    let obj = value.to_string(thread);
    if let FSRValue::String(s) = &obj.value {
        print!("{}", s);
    }
    return Ok(FSRRetValue::GlobalId(0));
}

pub fn fsr_fn_println<'a>(
    args: &[u64],
    thread: &mut FSRThreadRuntime<'a>
) -> Result<FSRRetValue<'a>, FSRError> {
    let value = FSRObject::id_to_obj(args[0]);
    let obj = value.to_string(thread);
    if let FSRValue::String(s) = &obj.value {
        println!("{}", s);
    }
    return Ok(FSRRetValue::GlobalId(0));
}

pub fn fsr_fn_dump<'a>(
    args: &[u64],
    _thread: &mut FSRThreadRuntime<'a>
) -> Result<FSRRetValue<'a>, FSRError> {
    let value = FSRObject::id_to_obj(args[0]);
    println!("{:#?}", value);
    return Ok(FSRRetValue::GlobalId(0));
}

pub fn init_io<'a>() -> HashMap<&'static str, FSRObject<'a>> {
    let print_fn = FSRFn::from_rust_fn(fsr_fn_print);
    let println_fn = FSRFn::from_rust_fn(fsr_fn_println);
    let dump_fn = FSRFn::from_rust_fn(fsr_fn_dump);
    let mut m = HashMap::new();
    m.insert("print", print_fn);
    m.insert("println", println_fn);
    m.insert("dump", dump_fn);
    m
}
