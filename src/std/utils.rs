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
        vm::thread::FSRThreadRuntime,
    },
    utils::error::{FSRErrCode, FSRError},
};

pub fn fsr_fn_assert<'a>(
    args: &[ObjId],
    _thread: &mut FSRThreadRuntime<'a>,
    _module: ObjId,
) -> Result<FSRRetValue<'a>, FSRError> {
    let value = FSRObject::id_to_obj(args[0]);
    if args.len() == 2 {
        if value.is_false() {
            let message = args[1];
            if let FSRValue::String(cow) = &FSRObject::id_to_obj(message).value {
                panic!("{}", cow);
            } else {
                return Err(FSRError::new("not a string", FSRErrCode::NotValidArgs));
            }
        }
    }
    if value.is_false() {
        panic!("assert error")
    }
    Ok(FSRRetValue::GlobalId(0))
}

pub fn fsr_fn_export<'a>(
    args: &[ObjId],
    _thread: &mut FSRThreadRuntime<'a>,
    module: ObjId,
) -> Result<FSRRetValue<'a>, FSRError> {
    let name = match &FSRObject::id_to_obj(args[0]).value {
        FSRValue::String(cow) => cow,
        _ => {
            return Err(FSRError::new("not a string", FSRErrCode::NotValidArgs));
        }
    };

    let obj = args[1];

    let s = module;
    let m = FSRObject::id_to_mut_obj(s)
        .expect("not a module object")
        .as_mut_code();
    m.register_object(name.as_str(), obj);

    Ok(FSRRetValue::GlobalId(0))
}

pub fn fsr_fn_range<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    _module: ObjId,
) -> Result<FSRRetValue<'a>, FSRError> {
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
                FSRGlobalObjId::RangeCls as ObjId,
            )));
        }
    }

    unimplemented!()
}

pub fn fsr_fn_type<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId,
) -> Result<FSRRetValue<'a>, FSRError> {
    if args.len() != 1 {
        return Err(FSRError::new("too many args", FSRErrCode::NotValidArgs));
    }

    let obj = FSRObject::id_to_obj(args[0]);

    match &obj.value {
        FSRValue::Integer(i) => Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
            FSRString::new_value("Integer"),
            FSRGlobalObjId::StringCls as ObjId,
        ))),
        FSRValue::Float(_) => Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
            FSRString::new_value("Float"),
            FSRGlobalObjId::StringCls as ObjId,
        ))),
        FSRValue::String(fsrinner_string) => {
            Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
                FSRString::new_value("String"),
                FSRGlobalObjId::StringCls as ObjId,
            )))
        }
        FSRValue::Class(fsrclass) => Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
            FSRString::new_value("Class"),
            FSRGlobalObjId::StringCls as ObjId,
        ))),
        FSRValue::ClassInst(fsrclass_inst) => {
            let name = fsrclass_inst.get_cls_name();
            Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
                FSRString::new_value(name),
                FSRGlobalObjId::StringCls as ObjId,
            )))
        }
        FSRValue::Function(fsrfn) => Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
            FSRString::new_value("Fn"),
            FSRGlobalObjId::StringCls as ObjId,
        ))),
        FSRValue::Bool(_) => Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
            FSRString::new_value("Bool"),
            FSRGlobalObjId::StringCls as ObjId,
        ))),
        FSRValue::List(fsrlist) => Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
            FSRString::new_value("List"),
            FSRGlobalObjId::StringCls as ObjId,
        ))),
        FSRValue::Iterator(fsrinner_iterator) => {
            Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
                FSRString::new_value("Iterator"),
                FSRGlobalObjId::StringCls as ObjId,
            )))
        }
        FSRValue::Code(fsrcode) => Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
            FSRString::new_value("Code"),
            FSRGlobalObjId::StringCls as ObjId,
        ))),
        FSRValue::Range(fsrrange) => Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
            FSRString::new_value("Range"),
            FSRGlobalObjId::StringCls as ObjId,
        ))),
        FSRValue::Any(any_type) => Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
            FSRString::new_value("Any"),
            FSRGlobalObjId::StringCls as ObjId,
        ))),
        FSRValue::Module(fsrmodule) => {
            Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
                FSRString::new_value("Module"),
                FSRGlobalObjId::StringCls as ObjId,
            )))
        }
        FSRValue::None => Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
            FSRString::new_value("None"),
            FSRGlobalObjId::StringCls as ObjId,
        ))),
    }
}

pub fn fsr_fn_id<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    _module: ObjId,
) -> Result<FSRRetValue<'a>, FSRError> {
    if args.len() != 1 {
        return Err(FSRError::new("too many args", FSRErrCode::NotValidArgs));
    }

    let integer = thread.garbage_collect.new_object(
        FSRValue::Integer(args[0] as i64),
        FSRGlobalObjId::IntegerCls as ObjId,
    );
    Ok(FSRRetValue::GlobalId(integer))
}

fn fsr_is_class<'a>(
    args: &[ObjId],
    _thread: &mut FSRThreadRuntime<'a>,
    _module: ObjId,
) -> Result<FSRRetValue<'a>, FSRError> {
    if args.len() != 2 {
        return Err(FSRError::new("too many args", FSRErrCode::NotValidArgs));
    }

    let obj = FSRObject::id_to_obj(args[0]);

    if obj.cls == args[1] {
        return Ok(FSRRetValue::GlobalId(FSRObject::true_id()));
    }

    Ok(FSRRetValue::GlobalId(FSRObject::false_id()))
}

pub fn fsr_timeit<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId,
) -> Result<FSRRetValue<'a>, FSRError> {
    if args.len() != 2 {
        return Err(FSRError::new("too many args", FSRErrCode::NotValidArgs));
    }

    let fn_def = FSRObject::id_to_obj(args[0]);
    if let FSRValue::Integer(count) = &FSRObject::id_to_obj(args[1]).value {
        let start = std::time::Instant::now();
        for _ in 0..*count {
            let _ = match fn_def.call(&[], thread, module, args[0])? {
                FSRRetValue::GlobalId(id) => {
                    if FSRObject::is_sp_object(id) {
                        continue;
                    }
                }
                FSRRetValue::Reference(_) => {}
            };
        }
        let end = std::time::Instant::now();
        println!(
            "times: {}\nduration: {:?}\nspeed: {}/s",
            count,
            end - start,
            *count as f64 / (end - start).as_secs_f64()
        );
        return Ok(FSRRetValue::GlobalId(0));
    }
    unimplemented!()
}

pub fn fsr_sleep<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    _module: ObjId,
) -> Result<FSRRetValue<'a>, FSRError> {
    if let FSRValue::Integer(i) = &FSRObject::id_to_obj(args[0]).value {
        //thread.release();
        thread.safe_point_to_stop();
        std::thread::sleep(Duration::from_millis(*i as u64));
        thread.acquire();
    }

    Ok(FSRRetValue::GlobalId(0))
}

pub fn init_utils<'a>() -> HashMap<&'static str, FSRObject<'a>> {
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
