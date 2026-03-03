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

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn free(&mut self, frame: Box<CallFrame>) {
        self.list.push(frame);
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn new_frame(&mut self, code: ObjId, fn_obj: ObjId, max_local_id: u64) -> Box<CallFrame> {
        if let Some(mut frame) = self.list.pop() {
            frame.clear();
            frame.code = code;
            frame.fn_id = fn_obj;
            frame.ip = (0, 0);
            frame.local_var.reserve(max_local_id);
            return frame;
        }

        let mut frame = CallFrame::new(code, fn_obj);
        frame.local_var.reserve(max_local_id);
        Box::new(frame)
    }
}