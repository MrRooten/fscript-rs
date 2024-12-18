use crate::{
    backend::{
        types::
            base::{FSRObject, FSRRetValue, FSRValue}
        ,
        vm::thread::FSRThreadRuntime,
    },
    utils::error::FSRError,
};

pub fn fsr_fn_assert<'a>(
    args: &[u64],
    thread: &mut FSRThreadRuntime<'a>
) -> Result<FSRRetValue<'a>, FSRError> {
    let value = FSRObject::id_to_obj(args[0]);
    if value.is_false() {
        panic!("assert error")
    }
    return Ok(FSRRetValue::GlobalId(0));
}