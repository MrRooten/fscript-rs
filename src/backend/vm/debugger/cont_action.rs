use crate::backend::vm::debugger::CommandAction;

pub struct ContAction {}

impl CommandAction for ContAction {
    fn action(
        &self,
        thread_rt: &mut crate::backend::vm::thread::FSRThreadRuntime,
        args: &[&str]
    ) -> Result<(), crate::utils::error::FSRError> {
        todo!()
    }
    
    fn get_name(&self) -> &'static str {
        "continue"
    }
}
