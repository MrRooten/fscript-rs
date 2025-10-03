use crate::{
    backend::{
        types::{
            any::{ExtensionTrait, FSRExtension},
            base::{FSRObject, FSRRetValue, FSRValue, ObjId},
            class::FSRClass,
            fn_def::FSRFn,
        },
        vm::thread::{CallFrame, FSCodeContext, FSRThreadRuntime},
    }, to_rs_list, utils::error::FSRError
};
use std::{fmt::Debug, future};

#[derive(Debug, PartialEq)]
pub enum FSRFutureState {
    Running,
    Suspended,
    Completed,
}

// struct FutureStatus<'a> {
//     callframe: Vec<CallFrame>,
//     context: Vec<FSCodeContext>,
// }

pub struct FSRFuture {
    //status: FutureStatus<'a>,
    pub(crate) state: FSRFutureState,
    pub(crate) fn_obj: ObjId,
    pub(crate) frame: Option<Box<CallFrame>>,
    result: Option<ObjId>,
}

impl Debug for FSRFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FSRFuture {{ state: {:?}, fn_obj: {} }}",
            self.state, self.fn_obj
        )
    }
}

impl FSRFuture {
    pub fn get_reference(
        &self,
        full: bool,
        worklist: &mut Vec<ObjId>,
        is_add: &mut bool,
    ) -> Box<dyn Iterator<Item = ObjId>> {
        let mut add_list = vec![];
        if let Some(frame) = self.frame.as_ref() {
            FSRThreadRuntime::process_callframe(&mut add_list, frame);
        }
        add_list.push(self.fn_obj);
        Box::new(add_list.into_iter())
    }
}

pub fn poll_future(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId,
) -> Result<FSRRetValue, FSRError> {
    if len != 1 {
        return Err(FSRError::new(
            "sorted_value requires exactly 1 argument",
            crate::utils::error::FSRErrCode::RuntimeError,
        ));
    }
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_mut_obj(args[0]).expect("not a valid object");
    let res = if let FSRValue::Future(future) = &mut self_object.value {
        if future.state == FSRFutureState::Completed {
            return Ok(FSRRetValue::GlobalId(FSRObject::none_id()));
        }
        let fn_obj = FSRObject::id_to_obj(future.fn_obj).as_fn();
        let mut frame = future.frame.take().expect("future frame is None");
        for arg in args.iter().rev() {
            frame.args.push(*arg);
        }
        frame.future = Some(args[0]);
        thread.push_frame(frame, fn_obj.const_map.clone());
        let res = thread.poll_fn(future.fn_obj);
        res
    } else {
        panic!("poll_future called on a non-future object");
    };

    return res.map(|x| FSRRetValue::GlobalId(x));
}

pub fn next_obj(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId,
) -> Result<FSRRetValue, FSRError> {
    poll_future(args, len, thread, code)
}

impl FSRFuture {
    pub fn get_class() -> FSRClass {
        let mut cls = FSRClass::new("Future");
        let poll = FSRFn::from_rust_fn_static(poll_future, "future_poll");
        cls.insert_attr("poll", poll);
        let next_obj = FSRFn::from_rust_fn_static(next_obj, "future_next");
        cls.insert_offset_attr(
            crate::backend::compiler::bytecode::BinaryOffset::NextObject,
            next_obj,
        );
        cls
    }

    pub fn new_value<'a>(fn_obj: ObjId, frame: Box<CallFrame>) -> FSRValue<'a> {
        let v = FSRFuture {
            state: FSRFutureState::Suspended,
            fn_obj,
            frame: Some(frame),
            result: None,
        };

        FSRValue::Future(Box::new(v))
    }

    pub fn set_completed(&mut self) {
        self.state = FSRFutureState::Completed;
    }

    pub fn set_result(&mut self, obj: ObjId) {
        self.result = Some(obj);
    }

    pub fn take_reuslt(&mut self) -> Option<ObjId> {
        self.result.take()
    }

    pub fn set_suspend(&mut self) {
        self.state = FSRFutureState::Suspended;
    }

    pub fn set_running(&mut self) {
        self.state = FSRFutureState::Running;
    }
}
