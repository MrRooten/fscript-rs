use crate::backend::vm::virtual_machine::get_object_by_global_id;

use super::{base::{FSRGlobalObjId, FSRObject, FSRValue, ObjId}, class::FSRClass};

pub struct FSRNone {

}

impl FSRNone {
    pub fn get_class() -> FSRClass<'static> {
        let mut cls = FSRClass::new("None");
        cls
    }
}