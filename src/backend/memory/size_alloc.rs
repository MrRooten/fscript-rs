
use crate::backend::{
    types::base::{AtomicObjId, FSRObject, FSRValue, ObjId}
    ,
    vm::thread::{AttrArgs, FSCodeContext},
};


#[allow(clippy::vec_box)]
pub struct FSRObjectAllocator<'a> {
    object_bins: Vec<Box<FSRObject<'a>>>,
    box_attr_bins: Vec<Box<AttrArgs<'a>>>,
    code_context_bins: Vec<Box<FSCodeContext>>,
}

#[allow(clippy::new_without_default)]
impl<'a> FSRObjectAllocator<'a> {
    pub fn new() -> Self {
        Self {
            object_bins: vec![],
            box_attr_bins: vec![],
            code_context_bins: vec![],
        }
    }

    #[inline(always)]
    pub fn new_object(&mut self, value: FSRValue<'a>, cls: ObjId) -> Box<FSRObject<'a>> {
        // self.allocator_count.fetch_add(1, Ordering::Relaxed);
        if let Some(mut s) = self.object_bins.pop() {
            s.cls = cls;
            s.value = value;
            //s.ref_count.store(0, Ordering::Relaxed);
            return s;
        }

        Box::new(FSRObject::new_inst(value, cls))
    }

    #[inline(always)]
    pub fn free_object(&mut self, obj: Box<FSRObject<'a>>) {
        self.object_bins.push(obj);
    }

    pub fn new_box_attr(
        &mut self,
        attr_id: u64,
        father: ObjId,
        attr: Option<&'a AtomicObjId>,
        name: &'a str,
        call_method: bool,
    ) -> Box<AttrArgs<'a>> {
        if let Some(mut s) = self.box_attr_bins.pop() {
            s.attr_id = attr_id;
            s.father = father;
            s.attr_object_id = attr;
            s.name = name;
            s.call_method = call_method;
            return s;
        }

        AttrArgs::new(attr_id, father, attr, name, call_method)
    }

    pub fn free_box_attr(&mut self, obj: Box<AttrArgs<'a>>) {
        self.box_attr_bins.push(obj);
    }

    pub fn new_code_context(
        &mut self,
        code: ObjId,
    ) -> Box<FSCodeContext> {
        if let Some(mut s) = self.code_context_bins.pop() {
            s.code = code;
            s.call_end = 1;
            return s;
        }

        Box::new(FSCodeContext::new_context(code))
    }

    pub fn free_code_context(&mut self, mut obj: Box<FSCodeContext>) {
        obj.clear();
        self.code_context_bins.push(obj);
    }
}
