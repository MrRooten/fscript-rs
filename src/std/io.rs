use std::collections::HashMap;

use crate::backend::{types::{base::{FSRObject, FSRValue}, fn_def::FSRFn}, vm::{runtime::FSRVM, thread::CallState}};

pub fn fsr_fn_print<'a>(args: Vec<u64>, stack: &'a mut CallState) -> Result<FSRObject<'a>, ()> {
    let value = 100;
    let value = stack.get_var(&value).unwrap();
    // let value = vm.get_obj_by_id(value).unwrap();
    // let value = value.borrow();
    // let obj = value.to_string();
    // if let FSRValue::String(s) = &obj.value {
    //     print!("{}", s);
    // }
    unimplemented!()
}

pub fn fsr_fn_println<'a>(args: Vec<u64>, stack: &'a mut CallState, vm: &'a mut FSRVM<'a>) -> Result<u64, ()> {
    let value = args[0];
    let value = vm.get_obj_by_id(&value).unwrap();
    let value = value.borrow();
    let obj = value.to_string();
    if let FSRValue::String(s) = &obj.value {
        println!("{}", s);
    }
    return Ok(0);
}

pub fn init_io<'a>() -> HashMap<&'static str, FSRObject<'a>> {
    //let print_fn = FSRFn::from_rust_fn(fsr_fn_print);
    let println_fn = FSRFn::from_rust_fn(fsr_fn_println);
    let mut m = HashMap::new();
    //m.insert("print", print_fn);
    m.insert("println", println_fn);
    return m;
}