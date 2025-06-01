use crate::backend::types::base::{FSRObject, ObjId};

pub extern "C" fn get_constant(code: ObjId, index: u64) -> ObjId {
    let module = FSRObject::id_to_obj(code).as_code().module;
    let module = FSRObject::id_to_obj(module).as_module();
    let constant = module.get_const(index as usize).unwrap();
    constant
}