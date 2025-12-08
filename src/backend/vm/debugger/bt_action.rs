use crate::backend::{types::base::FSRObject, vm::debugger::CommandAction};

pub struct BtAction {}

impl CommandAction for BtAction {
    fn action(
        &self,
        thread_rt: &mut crate::backend::vm::thread::FSRThreadRuntime,
        args: &[&str],
    ) -> Result<(), crate::utils::error::FSRError> {
        fn print_frame(idx: usize, frame: &crate::backend::vm::thread::CallFrame) {
            let code = FSRObject::id_to_obj(frame.code).as_code();
            let pos = code
                .get_expr(frame.ip.0)
                .and_then(|v| v.get(frame.ip.1.saturating_sub(1)))
                .map(|expr| expr.get_pos().as_human())
                .unwrap();
            println!("{}: {}, offset: {:?}", idx, frame.as_printable_str(), pos);
        }

        let cur = thread_rt.get_cur_frame();
        print_frame(0, cur);

        for (i, frame) in thread_rt.call_frames.iter().rev().enumerate() {
            print_frame(i + 1, frame);
        }

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        "bt"
    }
}
