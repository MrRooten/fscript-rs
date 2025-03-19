// use for reused object

use std::{borrow::Cow, collections::{LinkedList, VecDeque}};

use crate::backend::types::base::ObjId;

use super::thread::CallFrame;

pub struct FrameFreeList<'a> {
    list: VecDeque<CallFrame<'a>>
}

impl<'a> FrameFreeList<'a> {
    pub fn new_list() -> Self {
        Self {
            list: VecDeque::new()
        }
    }

    #[inline]
    pub fn free(&mut self, mut frame: CallFrame<'a>) {
        self.list.push_back(frame);
    }

    #[inline]
    pub fn new_frame(&mut self, name: &'a str, module: Option<ObjId>) -> CallFrame<'a> {
        if let Some(mut frame) = self.list.pop_front() {
            frame.clear();
            frame.module = module;
            return frame;
        }

        CallFrame::new(name, module)
    }
}