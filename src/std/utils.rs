use std::{collections::HashMap, ops::Range, time::Duration};

use crate::{
    backend::{
        memory::GarbageCollector,
        types::{
            base::{FSRGlobalObjId, FSRObject, FSRRetValue, FSRValue, ObjId},
            fn_def::FSRFn,
            range::FSRRange,
            string::FSRString,
        },
        vm::{thread::FSRThreadRuntime, virtual_machine::get_object_by_global_id},
    },
    utils::error::{FSRErrCode, FSRError},
};

pub fn fsr_fn_assert(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let value = FSRObject::id_to_obj(args[0]);
    if args.len() == 2 && value.is_false() {
        let message = args[1];
        if let FSRValue::String(cow) = &FSRObject::id_to_obj(message).value {
            panic!("{}", cow);
        } else {
            return Err(FSRError::new("not a string", FSRErrCode::NotValidArgs));
        }
    }
    if value.is_false() {
        panic!("assert error")
    }
    Ok(FSRRetValue::GlobalId(FSRObject::none_id()))
}

pub fn fsr_fn_export(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let name = match &FSRObject::id_to_obj(args[0]).value {
        FSRValue::String(cow) => cow,
        _ => {
            return Err(FSRError::new("not a string", FSRErrCode::NotValidArgs));
        }
    };

    let obj = args[1];

    let s = code;
    let module = FSRObject::id_to_mut_obj(
        FSRObject::id_to_obj(code)
            .as_code()
            .module,
    )
    .unwrap()
    .as_mut_module();
    module.register_object(name.as_str(), obj);

    Ok(FSRRetValue::GlobalId(FSRObject::none_id()))
}

pub fn fsr_fn_range(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    if args.len() != 2 {
        return Err(FSRError::new("too many args", FSRErrCode::NotValidArgs));
    }

    let start = FSRObject::id_to_obj(args[0]);
    let end = FSRObject::id_to_obj(args[1]);

    if let FSRValue::Integer(start) = start.value {
        if let FSRValue::Integer(end) = end.value {
            let range = FSRRange {
                range: Range { start, end },
            };

            return Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
                FSRValue::Range(Box::new(range)),
                get_object_by_global_id(FSRGlobalObjId::RangeCls) as ObjId,
            )));
        }
    }

    unimplemented!()
}

pub fn fsr_fn_type(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    if args.len() != 1 {
        return Err(FSRError::new("too many args", FSRErrCode::NotValidArgs));
    }

    let obj = FSRObject::id_to_obj(args[0]);

    match &obj.value {
        FSRValue::Integer(i) => Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
            FSRString::new_value("Integer"),
            get_object_by_global_id(FSRGlobalObjId::StringCls),
        ))),
        FSRValue::Float(_) => Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
            FSRString::new_value("Float"),
            get_object_by_global_id(FSRGlobalObjId::StringCls),
        ))),
        FSRValue::String(fsrinner_string) => {
            Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
                FSRString::new_value("String"),
                get_object_by_global_id(FSRGlobalObjId::StringCls),
            )))
        }
        FSRValue::Class(fsrclass) => Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
            FSRString::new_value("Class"),
            get_object_by_global_id(FSRGlobalObjId::StringCls),
        ))),
        FSRValue::ClassInst(fsrclass_inst) => {
            let name = fsrclass_inst.get_cls_name();
            Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
                FSRString::new_value(name),
                get_object_by_global_id(FSRGlobalObjId::StringCls),
            )))
        }
        FSRValue::Function(fsrfn) => Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
            FSRString::new_value("Fn"),
            get_object_by_global_id(FSRGlobalObjId::StringCls),
        ))),
        FSRValue::Bool(_) => Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
            FSRString::new_value("Bool"),
            get_object_by_global_id(FSRGlobalObjId::StringCls),
        ))),
        FSRValue::List(fsrlist) => Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
            FSRString::new_value("List"),
            get_object_by_global_id(FSRGlobalObjId::StringCls),
        ))),
        FSRValue::Iterator(fsrinner_iterator) => {
            Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
                FSRString::new_value("Iterator"),
                get_object_by_global_id(FSRGlobalObjId::StringCls),
            )))
        }
        FSRValue::Code(fsrcode) => Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
            FSRString::new_value("Code"),
            get_object_by_global_id(FSRGlobalObjId::StringCls),
        ))),
        FSRValue::Range(fsrrange) => Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
            FSRString::new_value("Range"),
            get_object_by_global_id(FSRGlobalObjId::StringCls),
        ))),
        FSRValue::Any(any_type) => Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
            FSRString::new_value("Any"),
            get_object_by_global_id(FSRGlobalObjId::StringCls),
        ))),
        FSRValue::Module(fsrmodule) => {
            Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
                FSRString::new_value("Module"),
                get_object_by_global_id(FSRGlobalObjId::StringCls),
            )))
        }
        FSRValue::None => Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
            FSRString::new_value("None"),
            get_object_by_global_id(FSRGlobalObjId::StringCls),
        ))),
    }
}

pub fn fsr_fn_id(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    if args.len() != 1 {
        return Err(FSRError::new("too many args", FSRErrCode::NotValidArgs));
    }

    let integer = thread.garbage_collect.new_object(
        FSRValue::Integer(args[0] as i64),
        get_object_by_global_id(FSRGlobalObjId::IntegerCls),
    );
    Ok(FSRRetValue::GlobalId(integer))
}

fn fsr_is_class(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    if args.len() != 2 {
        return Err(FSRError::new("too many args", FSRErrCode::NotValidArgs));
    }

    let obj = FSRObject::id_to_obj(args[0]);

    if obj.cls == args[1] {
        return Ok(FSRRetValue::GlobalId(FSRObject::true_id()));
    }

    Ok(FSRRetValue::GlobalId(FSRObject::false_id()))
}

pub fn fsr_timeit(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    if args.len() != 2 {
        return Err(FSRError::new("too many args", FSRErrCode::NotValidArgs));
    }

    let fn_def = FSRObject::id_to_obj(args[0]);
    if let FSRValue::Integer(count) = &FSRObject::id_to_obj(args[1]).value {
        let start = std::time::Instant::now();
        for _ in 0..*count {
            match fn_def.call(&[], thread, code, args[0])? {
                FSRRetValue::GlobalId(id) => {
                    if FSRObject::is_sp_object(id) {
                        continue;
                    }
                }
                // FSRRetValue::Reference(_) => {}
            };
        }
        let end = std::time::Instant::now();
        println!(
            "times: {}\nduration: {:?}\nspeed: {}/s",
            count,
            end - start,
            *count as f64 / (end - start).as_secs_f64()
        );
        return Ok(FSRRetValue::GlobalId(FSRObject::none_id()));
    }
    unimplemented!()
}

pub fn fsr_sleep(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    if let FSRValue::Integer(i) = &FSRObject::id_to_obj(args[0]).value {
        //thread.release();
        thread.safe_point_to_stop();
        std::thread::sleep(Duration::from_millis(*i as u64));
        thread.acquire();
    }

    Ok(FSRRetValue::GlobalId(FSRObject::none_id()))
}

pub fn init_utils() -> HashMap<&'static str, FSRObject<'static>> {
    let assert_fn = FSRFn::from_rust_fn_static(fsr_fn_assert, "assert");
    let export_fn = FSRFn::from_rust_fn_static(fsr_fn_export, "export");
    let sleep_fn = FSRFn::from_rust_fn_static(fsr_sleep, "sleep");
    let time_it = FSRFn::from_rust_fn_static(fsr_timeit, "timeit");
    let range = FSRFn::from_rust_fn_static(fsr_fn_range, "range");
    let is_class = FSRFn::from_rust_fn_static(fsr_is_class, "is_class");
    let type_fn = FSRFn::from_rust_fn_static(fsr_fn_type, "type");
    let id_fn = FSRFn::from_rust_fn_static(fsr_fn_id, "id");
    let mut m = HashMap::new();
    m.insert("assert", assert_fn);
    m.insert("export", export_fn);
    m.insert("sleep", sleep_fn);
    m.insert("timeit", time_it);
    m.insert("range", range);
    m.insert("is_class", is_class);
    m.insert("type", type_fn);
    m.insert("id", id_fn);
    m
}
