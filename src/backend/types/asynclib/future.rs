use crate::{
    backend::{
        types::{
            any::{AnyDebugSend, AnyType, GetReference},
            base::{FSRObject, FSRRetValue, FSRValue, ObjId},
            class::FSRClass,
            fn_def::FSRFn,
        },
        vm::thread::{CallFrame, FSCodeContext, FSRThreadRuntime},
    },
    utils::error::FSRError,
};
use std::fmt::Debug;
pub enum FSRFutureState {
    Running,
    Suspended,
    Completed,
}

// struct FutureStatus<'a> {
//     callframe: Vec<CallFrame<'a>>,
//     context: Vec<FSCodeContext>,
// }

pub struct FSRFuture<'a> {
    //status: FutureStatus<'a>,
    state: FSRFutureState,
    fn_obj: ObjId,
    pub(crate) frame: Option<Box<CallFrame<'a>>>,
}

impl Debug for FSRFuture<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FSRFuture ")
    }
}

impl<'a> GetReference for FSRFuture<'a> {
    fn get_reference(
        &self,
        full: bool,
        worklist: &mut Vec<ObjId>,
        is_add: &mut bool,
    ) -> Box<dyn Iterator<Item = ObjId> + 'a> {
        Box::new(vec![self.fn_obj].into_iter())
    }

    fn set_undirty(&mut self) {}
}

impl AnyDebugSend for FSRFuture<'static> {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
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
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let self_object = FSRObject::id_to_mut_obj(args[0]).expect("not a valid object");
    let fn_obj_code;
    if let FSRValue::Future(future) = &mut self_object.value {
        let fn_obj = FSRObject::id_to_obj(future.fn_obj).as_fn();
        let mut frame = future.frame.take().expect("future frame is None");
        for arg in args.iter().rev() {
            frame.args.push(*arg);
        }
        frame.future = Some(args[0]);
        thread.push_frame(frame);
        fn_obj_code = fn_obj.code;    
    } else {
        panic!("poll_future called on a non-future object");
    }
    
    let res = thread.poll_fn(fn_obj_code);

    
    return res.map(|x| {
        FSRRetValue::GlobalId(x)
    })
}

impl<'a> FSRFuture<'a> {
    pub fn get_class() -> FSRClass<'static> {
        let mut cls = FSRClass::new("Future");
        let poll = FSRFn::from_rust_fn_static(poll_future, "future_poll");
        cls.insert_attr("poll_future", poll);
        cls
    }

    pub fn new_value(fn_obj: ObjId, frame: Box<CallFrame<'a>>) -> FSRValue<'a> {
        let v = FSRFuture {
            state: FSRFutureState::Suspended,
            fn_obj,
            frame: Some(frame),
        };

        FSRValue::Future(Box::new(v))
    }
}
