// use for reused object


use crate::backend::types::base::ObjId;

use super::thread::CallFrame;

pub struct FrameFreeList {
    list: Vec<Box<CallFrame>>
}

impl FrameFreeList {
    pub fn new_list() -> Self {
        Self {
            list: Vec::new()
        }
    }

    #[inline]
    pub fn free(&mut self, frame: Box<CallFrame>) {
        self.list.push(frame);
    }

    #[inline]
    pub fn new_frame(&mut self, code: ObjId, fn_obj: ObjId) -> Box<CallFrame> {
        if let Some(mut frame) = self.list.pop() {
            frame.clear();
            frame.code = code;
            frame.fn_obj = fn_obj;
            return frame;
        }

        Box::new(CallFrame::new(code, fn_obj))
    }
}