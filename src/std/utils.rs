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
    return Ok(FSRRetValue::GlobalId(0));
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
    return Ok(FSRRetValue::GlobalId(0));
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

    return Ok(FSRRetValue::Value(Box::new(FSRInteger::new_inst(
        FSRObject::id_to_obj(args[0]).count_ref() as i64
    ))));
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
            return Ok(FSRRetValue::Value(Box::new(FSRString::new_inst(
                Cow::Borrowed("Integer")
            ))));
        }
        FSRValue::Float(f) => {
            return Ok(FSRRetValue::Value(Box::new(FSRString::new_inst(
                Cow::Borrowed("Float")
            ))));
        },
        FSRValue::String(cow) => {
            return Ok(FSRRetValue::Value(Box::new(FSRString::new_inst(
                Cow::Borrowed("String")
            ))));
        },
        FSRValue::Class(fsrclass) => {
            return Ok(FSRRetValue::Value(Box::new(FSRString::new_inst(
                Cow::Borrowed("Class")
            ))));
        },
        FSRValue::ClassInst(fsrclass_inst) => {
            return Ok(FSRRetValue::Value(Box::new(FSRString::new_inst(
                Cow::Borrowed(fsrclass_inst.get_cls_name())
            ))));
        },
        FSRValue::Function(fsrfn) => {
            return Ok(FSRRetValue::Value(Box::new(FSRString::new_inst(
                Cow::Borrowed("Function")
            ))));
        },
        FSRValue::Bool(b) => {
            return Ok(FSRRetValue::Value(Box::new(FSRString::new_inst(
                Cow::Borrowed("Bool")
            ))));
        },
        FSRValue::List(fsrlist) => {
            return Ok(FSRRetValue::Value(Box::new(FSRString::new_inst(
                Cow::Borrowed("List")
            ))));
        },
        FSRValue::Iterator(fsrinner_iterator) => {
            return Ok(FSRRetValue::Value(Box::new(FSRString::new_inst(
                Cow::Borrowed("Iterator")
            ))));
        },
        FSRValue::Module(fsrmodule) => {
            return Ok(FSRRetValue::Value(Box::new(FSRString::new_inst(
                Cow::Borrowed("Module")
            ))));
        },
        FSRValue::None => {
            return Ok(FSRRetValue::Value(Box::new(FSRString::new_inst(
                Cow::Borrowed("None")
            ))));
        },
    }
}

pub fn init_utils<'a>() -> HashMap<&'static str, FSRObject<'a>> {
    let assert_fn = FSRFn::from_rust_fn(fsr_fn_assert);
    let export_fn = FSRFn::from_rust_fn(fsr_fn_export);
    let ref_count = FSRFn::from_rust_fn(fsr_fn_ref_count);
    let type_fn = FSRFn::from_rust_fn(fsr_fn_type);
    let mut m = HashMap::new();
    m.insert("assert", assert_fn);
    m.insert("export", export_fn);
    m.insert("ref_count", ref_count);
    m.insert("type", type_fn);
    m
}