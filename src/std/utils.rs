use crate::{
    backend::{
        types::{
            base::{FSRObject, FSRRetValue, FSRValue}, module::FSRModule}
        ,
        vm::thread::FSRThreadRuntime,
    },
    utils::error::{FSRErrCode, FSRError},
};

pub fn fsr_fn_assert<'a>(
    args: &[u64],
    _thread: &mut FSRThreadRuntime<'a>,
    _module: Option<&FSRModule>
) -> Result<FSRRetValue<'a>, FSRError> {
    let value = FSRObject::id_to_obj(args[0]);
    if value.is_false() {
        panic!("assert error")
    }
    return Ok(FSRRetValue::GlobalId(0));
}


pub fn fsr_fn_export<'a>(
    args: &[u64],
    _thread: &mut FSRThreadRuntime<'a>,
    module: Option<&FSRModule>
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
        s.register_object(name, obj);
    }
    return Ok(FSRRetValue::GlobalId(0));
}