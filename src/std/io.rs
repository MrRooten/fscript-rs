use std::{borrow::Cow, collections::HashMap};

use crate::{
    backend::{
        memory::GarbageCollector, types::{
            base::{FSRGlobalObjId, FSRObject, FSRRetValue, FSRValue, ObjId},
            fn_def::FSRFn, string::{FSRInnerString, FSRString},
        }, vm::thread::FSRThreadRuntime
    },
    utils::error::FSRError,
};


pub fn fsr_fn_print(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime,
    module: ObjId
) -> Result<FSRRetValue, FSRError> {
    let value = FSRObject::id_to_obj(args[0]);
    let rest = args[1..].to_vec();
    let obj = value.to_string(thread, module);
    let res_obj = rest.iter().map(|x| {
        let obj = FSRObject::id_to_obj(*x);
        obj.to_string(thread, module)
    }).collect::<Vec<_>>();
    let mut ret = FSRInnerString::new("");

    if let FSRValue::String(s) = &obj {
        ret.push_inner_str(s);
        for r in res_obj {
            if let FSRValue::String(s) = &r {
                ret.push_inner_str(s);
            }
        }
        print!("{}", ret);
    }
    Ok(FSRRetValue::GlobalId(0))
}

pub fn fsr_fn_println(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime,
    module: ObjId
) -> Result<FSRRetValue, FSRError> {
    let value = FSRObject::id_to_obj(args[0]);
    let rest = args[1..].to_vec();
    let obj = value.to_string(thread, module);
    let res_obj = rest.iter().map(|x| {
        let obj = FSRObject::id_to_obj(*x);
        obj.to_string(thread, module)
    }).collect::<Vec<_>>();
    let mut ret = FSRInnerString::new("");

    if let FSRValue::String(s) = &obj {
        ret.push_inner_str(s);
        for r in res_obj {
            if let FSRValue::String(s) = &r {
                ret.push_inner_str(s);
            }
        }
        println!("{}", ret);
    }
    Ok(FSRRetValue::GlobalId(0))
}

pub fn fsr_fn_dump(
    args: &[ObjId],
    _thread: &mut FSRThreadRuntime,
    _module: ObjId
) -> Result<FSRRetValue, FSRError> {
    let value = FSRObject::id_to_obj(args[0]);
    println!("{:#?}", value);
    Ok(FSRRetValue::GlobalId(0))
}

pub fn fsr_fn_format(
    args: &[ObjId],
    _thread: &mut FSRThreadRuntime,
    _module: ObjId
) -> Result<FSRRetValue, FSRError> {
    
    let value = FSRObject::id_to_obj(args[0]);
    println!("{:#?}", value);
    Ok(FSRRetValue::GlobalId(0))
}

pub fn fsr_fn_str(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime,
    module: ObjId
) -> Result<FSRRetValue, FSRError> {
    
    let value = FSRObject::id_to_obj(args[0]);
    let s = value.to_string(thread, module);
    let obj_id = thread.garbage_collect.new_object(
        s,
        FSRGlobalObjId::StringCls as ObjId,
    );
    Ok(FSRRetValue::GlobalId(obj_id))
}

pub fn fsr_fn_throw_error(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime,
    _module: ObjId
) -> Result<FSRRetValue, FSRError> {
    if args.is_empty() || args[0] == 0 {
        thread.exception = FSRObject::none_id();
    } else {
        thread.exception = args[0];
    }
    Ok(FSRRetValue::GlobalId(0))
}

pub fn fsr_fn_get_error(
    _args: &[ObjId],
    thread: &mut FSRThreadRuntime,
    _module: ObjId
) -> Result<FSRRetValue, FSRError> {
    let ret = thread.get_cur_mut_frame().handling_exception;
    Ok(FSRRetValue::GlobalId(ret))
}

pub fn init_io() -> HashMap<&'static str, FSRObject<'static>> {
    let print_fn = FSRFn::from_rust_fn_static(fsr_fn_print, "print");
    let println_fn = FSRFn::from_rust_fn_static(fsr_fn_println, "println");
    let dump_fn = FSRFn::from_rust_fn_static(fsr_fn_dump, "dump");
    let throw_error = FSRFn::from_rust_fn_static(fsr_fn_throw_error, "throw_error");
    let get_error = FSRFn::from_rust_fn_static(fsr_fn_get_error, "pop_error");
    let str_fn = FSRFn::from_rust_fn_static(fsr_fn_str, "str");
    let mut m = HashMap::new();
    m.insert("print", print_fn);
    m.insert("println", println_fn);
    m.insert("dump", dump_fn);
    m.insert("throw_error", throw_error);
    m.insert("get_error", get_error);
    m.insert("str", str_fn);
    m
}

