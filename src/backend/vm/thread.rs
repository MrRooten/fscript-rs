use std::collections::{HashMap, LinkedList};

use crate::backend::{
    compiler::bytecode::{ArgType, Bytecode, BytecodeArg, BytecodeOperator},
    types::base::FSRObject,
};

use super::runtime::FSRVM;

struct CallState {
    var_map: HashMap<u64, u64>,
}

impl CallState {
    pub fn get_var(&self, name: &str) -> Option<&FSRObject> {
        unimplemented!()
    }
}

pub struct FSRThreadRuntime {
    call_stack: Vec<CallState>,
}

impl FSRThreadRuntime {
    pub fn new() -> FSRThreadRuntime {
        unimplemented!()
    }

    fn run_expr(expr: &LinkedList<BytecodeArg>, ip: &mut usize, stack: &mut CallState, vm: &mut FSRVM) {
        // let mut exp_stack = vec![];
        for arg in expr {
            if arg.get_operator() == &BytecodeOperator::Load {
                if let ArgType::Variable(id, name) = arg.get_arg() {

                }
            } else if arg.get_operator() == &BytecodeOperator::Assign {

            } else if arg.get_operator() == &BytecodeOperator::BinaryAdd {

            } 
        }
    }

    pub fn start(&self, bytecode: Bytecode, vm: &mut FSRVM) {
        let mut ip = 0;
        loop {
            let expr = match bytecode.get(ip) {
                Some(s) => s,
                None => {
                    break;
                }
            };

            //Self::run_expr(expr, &mut ip);
        }
    }
}
