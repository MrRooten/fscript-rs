use crate::{
    backend::{
        types::{
            base::{FSRObject, FSRRetValue, FSRValue, GlobalObj, ObjId},
            class::FSRClass,
            module::FSRModule,
            string::{FSRString, fsr_fn_format_string},
        },
        vm::{thread::FSRThreadRuntime, virtual_machine::gid},
    }, register_attr, to_rs_list, utils::error::FSRError
};

pub fn fsr_fn_chr(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);
    if let FSRValue::Integer(chr_value) = &self_object.value {
        if let Some(c) = std::char::from_u32(*chr_value as u32) {
            let fsr_str = FSRString::new_value(c.to_string());
            let str_obj = thread.garbage_collect.new_object(
                fsr_str,
                gid(GlobalObj::StringCls),
            );
            return Ok(FSRRetValue::GlobalId(str_obj));
        }
    }
    Err(FSRError::new(
        "Invalid argument for strip",
        crate::utils::error::FSRErrCode::NotValidArgs,
    ))
}

pub fn fsr_fn_ord(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);
    if let FSRValue::String(s) = &self_object.value {
        let mut c = s.as_str().chars().next().unwrap();
        let ord_value = c as u32 as i64;
        let int_obj = thread.garbage_collect.new_object(
            FSRValue::Integer(ord_value),
            gid(GlobalObj::IntegerCls),
        );
        return Ok(FSRRetValue::GlobalId(int_obj));
    }
    Err(FSRError::new(
        "Invalid argument for ord",
        crate::utils::error::FSRErrCode::NotValidArgs,
    ))
}

pub struct FSRStringModule {}

impl FSRStringModule {
    pub fn new_module(thread: &mut FSRThreadRuntime) -> FSRValue<'static> {
        let mut module = FSRModule::new_module("str");
        register_attr!(module, thread, "format", fsr_fn_format_string);
        register_attr!(module, thread, "chr", fsr_fn_chr);
        register_attr!(module, thread, "ord", fsr_fn_ord);
        FSRValue::Module(Box::new(module))
    }
}
