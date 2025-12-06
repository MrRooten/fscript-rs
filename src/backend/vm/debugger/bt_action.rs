use crate::backend::{types::base::FSRObject, vm::debugger::CommandAction};

pub struct BtAction {}

impl CommandAction for BtAction {
    fn action(
        &self,
        thread_rt: &mut crate::backend::vm::thread::FSRThreadRuntime,
        args: &[&str]
    ) -> Result<(), crate::utils::error::FSRError> {
        println!("0: {}, offset: {:?}", thread_rt.get_cur_frame().as_printable_str(), thread_rt.get_cur_frame().ip);
        for frame in thread_rt.call_frames.iter().rev().enumerate() {
            println!("{}: {}, offset: {:?}", frame.0 + 1, frame.1.as_printable_str(), frame.1.ip);
        }
        Ok(())
    }

    fn get_name(&self) -> &'static str {
        "bt"
    }
}
