use std::io::{Write, stdin, stdout};

use crate::backend::vm::thread::FSRThreadRuntime;

pub enum FSRFlag {
    Debugger,
}

pub struct FSRDebugger {}

impl FSRDebugger {
    pub fn cont_prog(&mut self, thread_rt: &mut FSRThreadRuntime) {}

    pub fn new() -> Self {
        Self {}
    }

    pub fn take_control(&mut self, thread_rt: &mut FSRThreadRuntime) {
        loop {
            print!("> ");
            let _ = stdout().flush();
            let mut command = String::new();
            
            stdin()
                .read_line(&mut command)
                .expect("Did not enter a correct string");

            if command.eq("quit") {
                break;
            }
        }
    }
}
