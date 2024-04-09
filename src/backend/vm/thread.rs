use std::collections::{HashMap, LinkedList};

use crate::backend::{
    compiler::bytecode::{ArgType, Bytecode, BytecodeArg, BytecodeOperator},
    types::base::FSRObject,
};

use super::runtime::FSRVM;

struct CallState {
    var_map: HashMap<u64, u64>,
    const_map   : HashMap<u64, u64>
}

impl CallState {
    pub fn get_var(&self, id: &u64) -> Option<&u64> {
        self.var_map.get(id)
    }

    pub fn insert_var(&mut self, id: &u64, obj_id: u64) {
        self.var_map.insert(id.clone(), obj_id);
    }

    pub fn has_var(&self, id: &u64) -> bool {
        return self.var_map.get(id).is_some();
    }

    pub fn has_const(&self, id: &u64) -> bool {
        return self.const_map.get(id).is_some();
    }

    pub fn insert_const(&mut self, id: &u64, obj_id: u64) {
        self.const_map.insert(id.clone(), obj_id);
    }
}

pub struct FSRThreadRuntime {
    call_stack: Vec<CallState>,
}

impl FSRThreadRuntime {
    pub fn new() -> FSRThreadRuntime {
        unimplemented!()
    }

    fn load_integer_const(i: &i64, vm: &mut FSRVM) -> u64 {
        unimplemented!()
    }

    fn get_cur_stack(&mut self) -> &mut CallState {
        unimplemented!()
    }

    fn process(exp: &mut Vec<u64>, bytecode: &BytecodeArg, stack: &mut CallState) {
        if bytecode.get_operator() == &BytecodeOperator::Assign {
            if let ArgType::Variable(v, name) = bytecode.get_arg() {
                let assign_id = exp.pop().unwrap();
                
            }
            
        }
    }

    fn run_expr(&mut self, expr: &LinkedList<BytecodeArg>, ip: &mut usize, vm: &mut FSRVM) {
        let stack = self.get_cur_stack();
        let mut exp_stack = vec![];
        for arg in expr {
            if arg.get_operator() == &BytecodeOperator::Load {
                if let ArgType::Variable(id, name) = arg.get_arg() {

                }
                else if let ArgType::ConstInteger(id, i) = arg.get_arg() {
                    let int_const = Self::load_integer_const(i, vm);
                    stack.insert_const(id, int_const.clone());
                    exp_stack.push(int_const);
                }
                else if let ArgType::ConstString(id, s) = arg.get_arg() {

                }
            } else {

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
