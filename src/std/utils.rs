use std::{borrow::Cow, collections::HashMap};

use crate::{
    backend::{
        types::{
            base::{FSRObject, FSRRetValue, FSRValue, ObjId}, fn_def::FSRFn, integer::FSRInteger, module::FSRModule, string::FSRString}
        ,
        vm::thread::FSRThreadRuntime,
    },
    utils::error::{FSRErrCode, FSRError},
};

pub fn fsr_fn_assert<'a>(
    args: &[ObjId],
    _thread: &mut FSRThreadRuntime<'a>,
    module: Option<ObjId>
) -> Result<FSRRetValue<'a>, FSRError> {
    let value = FSRObject::id_to_obj(args[0]);
    if value.is_false() {
        panic!("assert error")
    }
    Ok(FSRRetValue::GlobalId(0))
}


pub fn fsr_fn_export<'a>(
    args: &[ObjId],
    _thread: &mut FSRThreadRuntime<'a>,
    module: Option<ObjId>
) -> Result<FSRRetValue<'a>, FSRError> {
    let name = match &FSRObject::id_to_obj(args[0]).value {
        FSRValue::String(cow) => cow,
        _ => {
            return Err(FSRError::new("not a string", FSRErrCode::NotValidArgs));
        }
    };

    let obj = args[1];
    let r_obj = FSRObject::id_to_obj(obj);
    r_obj.ref_add();
    if let Some(s) = module {
        let m = FSRObject::id_to_obj(s).as_module();
        m.register_object(name, obj);
    }
    Ok(FSRRetValue::GlobalId(0))
}

pub fn fsr_fn_ref_count<'a>(
    args: &[ObjId],
    _thread: &mut FSRThreadRuntime<'a>,
    module: Option<ObjId>
) -> Result<FSRRetValue<'a>, FSRError> {
    if args.len() != 1 {
        return Err(FSRError::new("too many args", FSRErrCode::NotValidArgs));
    }

    if FSRObject::is_sp_object(args[0]) {
        return Ok(FSRRetValue::Value(Box::new(FSRInteger::new_inst(
            0
        ))));
    }

    Ok(FSRRetValue::Value(Box::new(FSRInteger::new_inst(
        FSRObject::id_to_obj(args[0]).count_ref() as i64
    ))))
}

pub fn fsr_fn_type<'a>(
    args: &[ObjId],
    _thread: &mut FSRThreadRuntime<'a>,
    module: Option<ObjId>
) -> Result<FSRRetValue<'a>, FSRError> {
    if args.len() != 1 {
        return Err(FSRError::new("too many args", FSRErrCode::NotValidArgs));
    }

    let obj = FSRObject::id_to_obj(args[0]);

    match &obj.value {
        FSRValue::Integer(i) => {
            Ok(FSRRetValue::Value(Box::new(FSRString::new_inst(
                Cow::Borrowed("Integer")
            ))))
        }
        FSRValue::Float(f) => {
            Ok(FSRRetValue::Value(Box::new(FSRString::new_inst(
                Cow::Borrowed("Float")
            ))))
        },
        FSRValue::String(cow) => {
            Ok(FSRRetValue::Value(Box::new(FSRString::new_inst(
                Cow::Borrowed("String")
            ))))
        },
        FSRValue::Class(fsrclass) => {
            Ok(FSRRetValue::Value(Box::new(FSRString::new_inst(
                Cow::Borrowed("Class")
            ))))
        },
        FSRValue::ClassInst(fsrclass_inst) => {
            Ok(FSRRetValue::Value(Box::new(FSRString::new_inst(
                Cow::Borrowed(fsrclass_inst.get_cls_name())
            ))))
        },
        FSRValue::Function(fsrfn) => {
            Ok(FSRRetValue::Value(Box::new(FSRString::new_inst(
                Cow::Borrowed("Function")
            ))))
        },
        FSRValue::Bool(b) => {
            Ok(FSRRetValue::Value(Box::new(FSRString::new_inst(
                Cow::Borrowed("Bool")
            ))))
        },
        FSRValue::List(fsrlist) => {
            Ok(FSRRetValue::Value(Box::new(FSRString::new_inst(
                Cow::Borrowed("List")
            ))))
        },
        FSRValue::Iterator(fsrinner_iterator) => {
            Ok(FSRRetValue::Value(Box::new(FSRString::new_inst(
                Cow::Borrowed("Iterator")
            ))))
        },
        FSRValue::Module(fsrmodule) => {
            Ok(FSRRetValue::Value(Box::new(FSRString::new_inst(
                Cow::Borrowed("Module")
            ))))
        },
        FSRValue::None => {
            Ok(FSRRetValue::Value(Box::new(FSRString::new_inst(
                Cow::Borrowed("None")
            ))))
        },
    }
}

pub fn fsr_timeit<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: Option<ObjId>
) -> Result<FSRRetValue<'a>, FSRError> {
    if args.len() != 2 {
        return Err(FSRError::new("too many args", FSRErrCode::NotValidArgs));
    }

    let fn_def = FSRObject::id_to_obj(args[0]);
    if let FSRValue::Integer(count) = &FSRObject::id_to_obj(args[1]).value {
        let start = std::time::Instant::now();
        for _ in 0..*count {
            fn_def.call(&[], thread, module)?;
        }
        let end = std::time::Instant::now();
        println!("times: {}\nduration: {:?}\nspeed: {}/s", count, end - start, *count as f64 / (end - start).as_secs_f64());
        return Ok(FSRRetValue::GlobalId(0));
    }
    unimplemented!()
}


pub fn init_utils<'a>() -> HashMap<&'static str, FSRObject<'a>> {
    let assert_fn = FSRFn::from_rust_fn_static(fsr_fn_assert, "assert");
    let export_fn = FSRFn::from_rust_fn_static(fsr_fn_export, "export");
    let ref_count = FSRFn::from_rust_fn_static(fsr_fn_ref_count, "ref_count");
    let type_fn = FSRFn::from_rust_fn_static(fsr_fn_type, "type");
    let time_it = FSRFn::from_rust_fn_static(fsr_timeit, "timeit");
    let mut m = HashMap::new();
    m.insert("assert", assert_fn);
    m.insert("export", export_fn);
    m.insert("ref_count", ref_count);
    m.insert("type", type_fn);
    m.insert("timeit", time_it);
    m
}