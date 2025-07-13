
use crate::{
    backend::{
        types::{
            base::{FSRObject, FSRRetValue, FSRValue, GlobalObj, ObjId}, class::FSRClass, fn_def::FSRFn, integer::FSRInteger, module::FSRModule, string::FSRString
        },
        vm::thread::FSRThreadRuntime,
    },
    utils::error::FSRError,
};

pub struct FSROs {}

pub fn fsr_fn_get_os(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId,
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
    code: ObjId,
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
    code: ObjId,
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
    code: ObjId,
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
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
        panic!("get_environ expects 1 argument, got {}", len);
    }
}

pub fn fsr_fn_command(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId,
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
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

impl FSROs {
    pub fn get_class() -> FSRClass<'static> {
        let mut cls = FSRClass::new("Os");
        cls.init_method();
        let get_os = FSRFn::from_rust_fn_static(fsr_fn_get_os, "get_os");
        cls.insert_attr("get_os", get_os);
        let os_version = FSRFn::from_rust_fn_static(fsr_fn_os_version, "os_version");
        cls.insert_attr("os_version", os_version);
        let get_pid = FSRFn::from_rust_fn_static(fsr_fn_get_pid, "get_pid");
        cls.insert_attr("get_pid", get_pid);
        cls
    }

    pub fn new_module(thread: &mut FSRThreadRuntime) -> FSRValue<'static> {
        let mut module = FSRModule::new_module("os");
        let get_os = FSRFn::from_rust_fn_static_value(fsr_fn_get_os, "get_os");
        let get_os_fn_id = thread
            .garbage_collect
            .new_object(get_os, GlobalObj::FnCls.get_id());
        module.register_object("get_os", get_os_fn_id);
        let os_version = FSRFn::from_rust_fn_static_value(fsr_fn_os_version, "os_version");
        let os_version_fn_id = thread
            .garbage_collect
            .new_object(os_version, GlobalObj::FnCls.get_id());
        module.register_object("os_version", os_version_fn_id);
        let get_pid = FSRFn::from_rust_fn_static_value(fsr_fn_get_pid, "get_pid");
        let get_pid_fn_id = thread
            .garbage_collect
            .new_object(get_pid, GlobalObj::FnCls.get_id());
        module.register_object("get_pid", get_pid_fn_id);
        let get_environ = FSRFn::from_rust_fn_static_value(fsr_fn_get_environ, "get_environ");
        let get_environ_fn_id = thread
            .garbage_collect
            .new_object(get_environ, GlobalObj::FnCls.get_id());
        module.register_object("get_environ", get_environ_fn_id);
        let command = FSRFn::from_rust_fn_static_value(fsr_fn_command, "command");
        let command_fn_id = thread
            .garbage_collect
            .new_object(command, GlobalObj::FnCls.get_id());
        module.register_object("command", command_fn_id);
        FSRValue::Module(Box::new(module))
    }
}
