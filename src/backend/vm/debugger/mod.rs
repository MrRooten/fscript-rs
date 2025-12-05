use crate::{backend::vm::thread::FSRThreadRuntime, utils::error::FSRError};

pub mod debug;
pub mod cont_action;
pub mod bc_action;

pub trait CommandAction {
    fn action(&self, thread_rt: &mut FSRThreadRuntime) -> Result<(), FSRError>;

    fn get_name(&self) -> &'static str;
}