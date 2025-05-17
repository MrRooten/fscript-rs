use crate::backend::vm::virtual_machine::get_object_by_global_id;

use super::{base::{FSRGlobalObjId, FSRObject, FSRValue, ObjId}, class::FSRClass};

pub struct FSRBool {

}

impl<'a> FSRBool {
    pub fn get_class() -> FSRClass<'a> {
        let mut cls = FSRClass::new("Bool");
        cls
    }

    pub fn new_inst(i: bool) -> FSRObject<'a> {
        let mut object = FSRObject::new();
        object.set_cls(get_object_by_global_id(FSRGlobalObjId::BoolCls) as ObjId);
        object.set_value(FSRValue::Bool(i));
        object
    }
}