use crate::backend::types::base::{FSRObject, FSRValue, ObjId};

pub struct TCMemoryManager<'a> {
    object: Vec<Box<FSRObject<'a>>>,
}

impl<'a> TCMemoryManager<'a> {
    pub fn new() -> Self {
        TCMemoryManager {
            object: Vec::new(),
        }
    }

    pub fn new_object(&mut self, value: FSRValue<'a>, cls: ObjId) -> ObjId {
        let obj = Box::new(FSRObject::new_inst(value, cls));

        let id = FSRObject::obj_to_id(&obj);
        self.object.push(obj);
        id
    }

    pub fn shrink(&mut self) {
        self.object
            .retain(|obj| if obj.free { false } else { true });
    }
}
