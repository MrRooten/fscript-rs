use std::{collections::HashMap, io::{Write, stdin, stdout}};

use crate::backend::vm::{debugger::{CommandAction, ba_action::BaAction, bc_action::BcAction, bt_action::BtAction, cont_action::ContAction}, thread::FSRThreadRuntime};

pub enum FSRFlag {
    Debugger,
}

pub struct FSRDebugger {
    commands: HashMap<String, Box<dyn CommandAction>>
}

impl FSRDebugger {
    pub fn cont_prog(&mut self, thread_rt: &mut FSRThreadRuntime) {}

    pub fn new() -> Self {
        let mut commands: HashMap<String, Box<dyn CommandAction>> = HashMap::new();
        commands.insert("continue".to_string(), Box::new(ContAction {}));
        commands.insert("bc".to_string(), Box::new(BcAction {}));
        commands.insert("bt".to_string(), Box::new(BtAction {}));
        commands.insert("ba".to_string(), Box::new(BaAction {}));
        Self {
            commands
        }
    }

    pub fn take_control(&mut self, thread_rt: &mut FSRThreadRuntime) {
        loop {
            print!("> ");
            let _ = stdout().flush();
            let mut command = String::new();
            
            stdin()
                .read_line(&mut command)
                .expect("Did not enter a correct string");
            let command = command.trim();
            let args = command.split(" ").filter(|x| !x.is_empty()).collect::<Vec<_>>();
            let command = args[0];
            let args = &args[1..];
            if command.eq("quit") || command.eq("exit") {
                break;
            }

            let action = match self.commands.get(command) {
                Some(s) => s,
                None => {
                    println!("Not support command: {}", command);
                    continue;
                }
            };

            action.action(thread_rt, args);
        }
    }
}
