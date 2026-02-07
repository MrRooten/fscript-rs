
use rand::Rng;

use crate::{
    backend::{
        types::{
            base::{FSRObject, FSRRetValue, FSRValue, ObjId},
            fn_def::FSRFn,
            module::FSRModule,
        },
        vm::{thread::FSRThreadRuntime, virtual_machine::gid},
    }, register_class, register_attr, std::fs::{dir::FSRDir, file::{FSRInnerFile, fsr_fn_is_dir, fsr_fn_is_file}}, to_rs_list, utils::error::{FSRErrCode, FSRError}
};

fn rand_int(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    if len != 2 {
        return Err(FSRError::new(
            "rand_int args error",
            FSRErrCode::NotValidArgs,
        ));
    }
    let lower = FSRObject::id_to_obj(args[0]);
    let upper = FSRObject::id_to_obj(args[1]);

    let (lower_int, upper_int) = match (&lower.value, &upper.value) {
        (FSRValue::Integer(l), FSRValue::Integer(u)) => (*l, *u),
        _ => {
            return Err(FSRError::new(
                "rand_int requires integer arguments",
                FSRErrCode::NotValidArgs,
            ));
        }
    };

    let rand_value = if lower_int >= upper_int {
        lower_int
    } else {
        rand::rng().random_range(lower_int..upper_int)
    };

    let obj_id = thread.garbage_collect.get_integer(rand_value);
    
    Ok(FSRRetValue::GlobalId(obj_id))
}

pub struct FSRRandModule {}

impl FSRRandModule {
    pub fn new_module(thread: &mut FSRThreadRuntime) -> FSRValue<'static> {
        let mut module = FSRModule::new_module("rand");
        // register_class!(module, thread, "File", FSRInnerFile::get_class());
        // register_class!(module, thread, "Dir", FSRDir::get_class());
        // register_fn!(module, thread, "is_file", fsr_fn_is_file);
        // register_fn!(module, thread, "is_dir", fsr_fn_is_dir);
        register_attr!(module, thread, "rand_int", rand_int);
        FSRValue::Module(Box::new(module))
    }
}