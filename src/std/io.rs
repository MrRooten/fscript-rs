use std::{cell::Ref, collections::HashMap, sync::atomic::AtomicU64};

use crate::backend::{types::{base::{FSRObject, FSRValue}, fn_def::FSRFn}, vm::{runtime::FSRVM, thread::CallState}};

pub fn fsr_fn_print<'a>(args: Vec<u64>, stack: &mut CallState, vm: &FSRVM<'a>) -> Result<FSRObject<'a>, ()> {
    let value = FSRObject::id_to_obj(args[0]);
    let obj = value.to_string();
    if let FSRValue::String(s) = &obj.value {
        print!("{}", s);
    }
    let v = FSRObject {
        obj_id: 0,
        value: FSRValue::None,
        cls: "None",
        ref_count: AtomicU64::new(0)
    };
    return Ok(v);
}

pub fn fsr_fn_println<'a>(args: Vec<u64>, stack: &mut CallState, vm: &FSRVM<'a>) -> Result<FSRObject<'a>, ()> {
    let value = FSRObject::id_to_obj(args[0]);
    let obj = value.to_string();
    if let FSRValue::String(s) = &obj.value {
        println!("{}", s);
    }
    let v = FSRObject {
        obj_id: 0,
        value: FSRValue::None,
        cls: "None",
        ref_count: AtomicU64::new(0)
    };
    return Ok(v);
}

pub fn init_io<'a>() -> HashMap<&'static str, FSRObject<'a>> {
    let print_fn = FSRFn::from_rust_fn(fsr_fn_print);
    let println_fn = FSRFn::from_rust_fn(fsr_fn_println);
    let mut m = HashMap::new();
    m.insert("print", print_fn);
    m.insert("println", println_fn);
    return m;
}