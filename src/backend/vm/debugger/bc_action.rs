

use crate::backend::{types::base::FSRObject, vm::debugger::CommandAction};

pub struct BcAction {}

impl CommandAction for BcAction {
    fn action(
        &self,
        thread_rt: &mut crate::backend::vm::thread::FSRThreadRuntime,
    ) -> Result<(), crate::utils::error::FSRError> {
        let code = thread_rt.get_cur_frame().code;
        let code = FSRObject::id_to_obj(code).as_code();
        println!("{:#?}", code.get_bytecode().bytecode);
        Ok(())
    }
    
    fn get_name(&self) -> &'static str {
        "bc"
    }
}
