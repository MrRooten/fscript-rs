// use for reused object


use crate::backend::types::base::ObjId;

use super::thread::CallFrame;

pub struct FrameFreeList<'a> {
    list: Vec<Box<CallFrame<'a>>>
}

impl<'a> FrameFreeList<'a> {
    pub fn new_list() -> Self {
        Self {
            list: Vec::new()
        }
    }

    #[inline]
    pub fn free(&mut self, frame: Box<CallFrame<'a>>) {
        self.list.push(frame);
    }

    #[inline]
    pub fn new_frame(&mut self, name: &'a str, module: ObjId, fn_obj: ObjId) -> Box<CallFrame<'a>> {
        if let Some(mut frame) = self.list.pop() {
            frame.clear();
            frame.code = module;
            frame.fn_obj = fn_obj;
            return frame;
        }

        Box::new(CallFrame::new(name, module, fn_obj))
    }
}