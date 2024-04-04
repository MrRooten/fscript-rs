#![allow(unused)]

use super::runtime::VMCallState;



#[derive(Debug)]
pub struct FSRThread<'a> {
    thread_id       : u64,
    call_stack: Vec<VMCallState<'a>>
}