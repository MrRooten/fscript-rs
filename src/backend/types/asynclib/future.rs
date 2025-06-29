use crate::backend::{types::{any::{AnyDebugSend, GetReference}, base::ObjId}, vm::thread::{CallFrame, FSCodeContext}};
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
    frame: Box<CallFrame<'a>>,
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

    fn set_undirty(&mut self) {
        
    }
}

impl AnyDebugSend for FSRFuture<'static> {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}