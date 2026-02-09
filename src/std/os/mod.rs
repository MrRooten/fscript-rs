
use crate::{
    backend::{
        types::{
            base::{FSRObject, FSRRetValue, FSRValue, GlobalObj, ObjId}, class::FSRClass, ext::hashmap::FSRHashMap, fn_def::FSRFn, integer::FSRInteger, list::FSRList, module::FSRModule, string::FSRString
        },
        vm::thread::FSRThreadRuntime,
    }, register_fn, to_rs_list, utils::error::FSRError
};

pub struct FSROs {}

pub fn fsr_fn_cpu_arch(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let arch = std::env::consts::ARCH;
    let value = FSRString::new_value(arch);
    let res = thread
        .garbage_collect
        .new_object(value, GlobalObj::StringCls.get_id());
    Ok(FSRRetValue::GlobalId(res))
}

pub fn fsr_fn_platform(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let os = std::env::consts::OS;
    let value = FSRString::new_value(os);
    let res = thread
        .garbage_collect
        .new_object(value, GlobalObj::StringCls.get_id());
    Ok(FSRRetValue::GlobalId(res))
}

pub fn fsr_fn_os_version(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let value = FSRString::new_value("unknown");
    let res = thread
        .garbage_collect
        .new_object(value, GlobalObj::StringCls.get_id());
    Ok(FSRRetValue::GlobalId(res))
}

pub fn fsr_fn_get_pid(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let pid = std::process::id();

    let res = thread
        .garbage_collect
        .new_object(FSRValue::Integer(pid as i64), GlobalObj::StringCls.get_id());
    Ok(FSRRetValue::GlobalId(res))
}


pub fn fsr_fn_get_environ(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    if len == 1 {
        let key = args[0];
        let key_str = if let FSRValue::String(s) = &FSRObject::id_to_obj(key).value {
            s.to_string()
        } else {
            panic!("get_environ expects a string argument, got {:?}", FSRObject::id_to_obj(key).value);
        };
        let environ = std::env::vars()
            .filter(|(k, _)| k.eq_ignore_ascii_case(&key_str)).map(|x| x.1).collect::<Vec<_>>();
        if environ.is_empty() {
            return Ok(FSRRetValue::GlobalId(FSRObject::none_id()))
        }
        let value = FSRString::new_value(&environ[0]);
        let res = thread
            .garbage_collect
            .new_object(value, GlobalObj::StringCls.get_id());
        Ok(FSRRetValue::GlobalId(res))
    } else {
        let mut fsr_hashmap = FSRHashMap::new_hashmap();
        let environ = std::env::vars().map(|x| (x.0, x.1)).collect::<Vec<_>>();
        for (key, value) in environ {
            let key_obj = FSRString::new_value(&key);
            let value_obj = FSRString::new_value(&value);
            let key_id = thread.garbage_collect.new_object(key_obj, GlobalObj::StringCls.get_id());
            let value_id = thread.garbage_collect.new_object(value_obj, GlobalObj::StringCls.get_id());
            fsr_hashmap.insert(key_id, value_id, thread);
        }

        let res = thread
            .garbage_collect
            .new_object(fsr_hashmap.to_any_type(), GlobalObj::HashMapCls.get_id());
        Ok(FSRRetValue::GlobalId(res))
    }
}

pub fn fsr_fn_command(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    if len > 0 {
        let command = args[0];
        let args = &args[1..];
        if let FSRValue::String(s) = &FSRObject::id_to_obj(command).value {
            let args = args.iter().map(|&arg| {
                if let FSRValue::String(s) = &FSRObject::id_to_obj(arg).value {
                    s.to_string()
                } else {
                    panic!("command expects string arguments, got {:?}", FSRObject::id_to_obj(arg).value);
                }
            }).collect::<Vec<_>>();
            let output = std::process::Command::new(s.to_string()).args(args)
                .output()
                .map_err(|e| FSRError::new(format!("Failed to execute command: {}", e), crate::utils::error::FSRErrCode::RuntimeError))?;
            let output_str = String::from_utf8_lossy(&output.stdout);
            let value = FSRString::new_value(output_str);
            let res = thread
                .garbage_collect
                .new_object(value, GlobalObj::StringCls.get_id());
            Ok(FSRRetValue::GlobalId(res))
        } else {
            panic!("command expects a string argument, got {:?}", FSRObject::id_to_obj(command).value);
        }
    } else {
        panic!("command expects 1 argument, got {}", len);
    }
}

pub fn fsr_fn_get_args(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    // Get the command line arguments passed to the script
    let args = to_rs_list!(args, len);
    let args_vec = std::env::args().skip(1).collect::<Vec<String>>();
    let mut fsr_args = Vec::new();
    for arg in args_vec {
        let value = FSRString::new_value(&arg);
        let obj_id = thread.garbage_collect.new_object(value, GlobalObj::StringCls.get_id());
        fsr_args.push(obj_id);
    }
    let value = FSRList::new_value(fsr_args);
    let res = thread
        .garbage_collect
        .new_object(value, GlobalObj::ListCls.get_id());
    Ok(FSRRetValue::GlobalId(res))
}


impl FSROs {
    pub fn get_class() -> FSRClass {
        let mut cls = FSRClass::new("Os");
        cls.init_method();
        cls
    }

    pub fn new_module(thread: &mut FSRThreadRuntime) -> FSRValue<'static> {
        let mut module = FSRModule::new_module("os");
        register_fn!(module, thread, "get_pid", fsr_fn_get_pid);
        register_fn!(module, thread, "platform", fsr_fn_platform);
        register_fn!(module, thread, "os_version", fsr_fn_os_version);
        register_fn!(module, thread, "get_environ", fsr_fn_get_environ);
        register_fn!(module, thread, "command", fsr_fn_command);
        register_fn!(module, thread, "cpu_arch", fsr_fn_cpu_arch);
        register_fn!(module, thread, "get_args", fsr_fn_get_args);
        FSRValue::Module(Box::new(module))
    }
}
