
use crate::backend::{
    types::base::{AtomicObjId, FSRObject, FSRValue, ObjId}
    ,
    vm::thread::{FSCodeContext},
};


#[allow(clippy::vec_box)]
pub struct FSRObjectAllocator<'a> {
    object_bins: Vec<Box<FSRObject<'a>>>,
    code_context_bins: Vec<Box<FSCodeContext>>,
}

#[allow(clippy::new_without_default)]
impl<'a> FSRObjectAllocator<'a> {
    pub fn new() -> Self {
        Self {
            object_bins: vec![],
            code_context_bins: vec![],
        }
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn new_object(&mut self, value: FSRValue<'a>, cls: ObjId) -> Box<FSRObject<'a>> {
        // self.allocator_count.fetch_add(1, Ordering::Relaxed);
        if let Some(mut s) = self.object_bins.pop() {
            let cls = FSRObject::id_to_obj(cls).as_class();
            s.cls = cls;
            s.value = value;
            //s.ref_count.store(0, Ordering::Relaxed);
            return s;
        }

        Box::new(FSRObject::new_inst(value, cls))
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn free_object(&mut self, obj: Box<FSRObject<'a>>) {
        self.object_bins.push(obj);
    }


    
}
