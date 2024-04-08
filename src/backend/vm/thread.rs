use std::collections::LinkedList;

use crate::backend::compiler::bytecode::{ArgType, Bytecode, BytecodeArg, BytecodeOperator};

struct CallState {

}

pub struct FSRThreadRuntime {
    call_stack      : Vec<CallState>
}

impl FSRThreadRuntime {
    pub fn new() -> FSRThreadRuntime {
        unimplemented!()
    }

    fn run_expr(expr :&LinkedList<BytecodeArg>, ip: &mut usize) {
        for arg in expr {
            if arg.get_operator() == &BytecodeOperator::Load {
                if let ArgType::Variable(id, name) = arg.get_arg() {

                }
            }
        }
    }

    pub fn start(&self, bytecode: Bytecode) {
        let mut ip = 0;
        loop {
            let expr = match bytecode.get(ip) {
                Some(s) => s,
                None => {
                    break;
                }
            };

            Self::run_expr(expr, &mut ip);
        }
    }
}

