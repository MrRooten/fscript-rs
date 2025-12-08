
use crate::backend::{compiler::bytecode::FSRDbgFlag, types::base::FSRObject, vm::debugger::CommandAction};

pub struct BaAction {}

impl CommandAction for BaAction {
    fn action(
        &self,
        thread_rt: &mut crate::backend::vm::thread::FSRThreadRuntime,
        args: &[&str]
    ) -> Result<(), crate::utils::error::FSRError> {
        let code = thread_rt.get_cur_frame().code;
        let code = FSRObject::id_to_obj(code).as_code();
        let line = args[0];
        let line: usize = line.parse().unwrap();
        let expr = code.get_expr(line as usize).unwrap();
        expr[0].set_dbg(FSRDbgFlag::Keep);
        Ok(())
    }
    
    fn get_name(&self) -> &'static str {
        "ba"
    }
}
