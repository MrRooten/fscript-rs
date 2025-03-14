// use for reused object

use std::{borrow::Cow, collections::LinkedList};

use crate::backend::types::base::ObjId;

use super::thread::CallFrame;

pub struct FrameFreeList<'a> {
    list: LinkedList<CallFrame<'a>>
}

impl<'a> FrameFreeList<'a> {
    pub fn new() -> Self {
        Self {
            list: LinkedList::new()
        }
    }

    pub fn push(&mut self, mut frame: CallFrame<'a>) {
        self.list.push_back(frame);
    }

    pub fn new_frame(&mut self, name: &'a str, module: Option<ObjId>) -> CallFrame<'a> {
        if let Some(mut frame) = self.list.pop_front() {
            frame.module = module;
            return frame;
        }

        CallFrame::new(Cow::Borrowed(name), module)
    }
}