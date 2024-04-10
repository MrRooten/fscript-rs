use std::{cell::Ref, collections::HashMap};

use crate::backend::{types::{base::{FSRObject, FSRValue}, fn_def::FSRFn}, vm::{runtime::FSRVM, thread::CallState}};

pub fn fsr_fn_print<'a>(args: Vec<Ref<FSRObject<'a>>>, stack: &mut CallState, vm: &FSRVM<'a>) -> Result<FSRObject<'a>, ()> {
    let value = &args[0];
    let obj = value.to_string();
    if let FSRValue::String(s) = &obj.value {
        print!("{}", s);
    }
    let v = FSRObject {
        obj_id: 0,
        value: FSRValue::None,
        cls: "None",
    };
    return Ok(v);
}

pub fn fsr_fn_println<'a>(args: Vec<Ref<FSRObject<'a>>>, stack: &mut CallState, vm: &FSRVM<'a>) -> Result<FSRObject<'a>, ()> {
    let value = &args[0];
    let obj = value.to_string();
    if let FSRValue::String(s) = &obj.value {
        println!("{}", s);
    }
    let v = FSRObject {
        obj_id: 0,
        value: FSRValue::None,
        cls: "None",
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
    unimplemented!()
}