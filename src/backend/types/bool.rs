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
        object.set_cls(FSRGlobalObjId::BoolCls as ObjId);
        object.set_value(FSRValue::Bool(i));
        object
    }
}