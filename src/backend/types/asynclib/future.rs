use crate::backend::{types::base::ObjId, vm::thread::CallFrame};

pub enum FSRFutureState {
    Running,
    Suspended,
    Completed,
}

struct FutureStatus<'a> {
    code: ObjId,
    ip: (usize, usize),
    callframe: Vec<CallFrame<'a>>
}

pub struct FSRFuture<'a> {
    status: FutureStatus<'a>,
    state: FSRFutureState,

}