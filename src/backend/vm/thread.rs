use std::{cell::RefCell, collections::{HashMap, LinkedList}};

use crate::{backend::{
    compiler::bytecode::{ArgType, Bytecode, BytecodeArg, BytecodeOperator},
    types::{base::{FSRObject, FSRValue}, string::FSRString},
}, frontend::ast::token::call};

use super::runtime::FSRVM;

pub struct CallState {
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

    pub fn new() -> Self {
        Self {
            var_map: HashMap::new(),
            const_map: HashMap::new(),
        }
    }
}

enum SValue<'a> {
    StackId((u64, &'a String)),
    GlobalId(u64)
}

impl SValue<'_> {
    pub fn get_value(&self) -> u64 {
        match self {
            SValue::StackId(i) => i.0.clone(),
            SValue::GlobalId(i) => i.clone(),
        }
    }

    pub fn get_global_id(&self, stack: &CallState, vm: &FSRVM) -> u64 {
        match self {
            SValue::StackId(i) => stack.get_var(&i.0).unwrap().clone(),
            SValue::GlobalId(i) => i.clone(),
        }
    }
}

pub struct FSRThreadRuntime {
    call_stack: Vec<CallState>,
}

impl<'a> FSRThreadRuntime {
    pub fn new() -> FSRThreadRuntime {
        Self {
            call_stack: vec![CallState::new()],
        }
    }

    fn load_integer_const(i: &i64, vm: &mut FSRVM) -> u64 {
        let obj = FSRObject {
            obj_id: 0,
            value: FSRValue::Integer(i.clone()),
            cls: "Integer",
            attrs: HashMap::new(),
        };

        let id = vm.register_object(obj);
        return id
    }

    fn load_string_const(s: String, vm: &mut FSRVM<'a>) -> u64 {
        let obj = FSRObject {
            obj_id: 0,
            value: FSRValue::String(s),
            cls: "String",
            attrs: HashMap::new(),
        };

        let id = vm.register_object(obj);        
        return id;
    }

    fn get_cur_stack(&mut self) -> &mut CallState {
        let l = self.call_stack.len();
        return self.call_stack.get_mut(l-1).unwrap();
    }

    fn process(&mut self, exp: &mut Vec<SValue>, bytecode: &BytecodeArg, state: &'a mut CallState, ip: &mut usize, vm: &mut FSRVM<'a>) -> bool {
        if bytecode.get_operator() == &BytecodeOperator::Assign {
            //if let ArgType::Variable(v, name) = bytecode.get_arg() {
            let assign_id = exp.pop().unwrap();
            let obj_id = exp.pop().unwrap();
            if let SValue::GlobalId(id) = obj_id {
                state.insert_var(&assign_id.get_value(), id);
            }
            else if let SValue::StackId(s_id) = obj_id {
                let id = match state.get_var(&s_id.0) {
                    Some(s) => s.clone(),
                    None => {
                        vm.get_global_obj_by_name(s_id.1).unwrap().clone()
                    }
                };
                state.insert_var(&assign_id.get_value(), id);
            }
        }
        else if bytecode.get_operator() == &BytecodeOperator::BinaryAdd {
            let v1 = exp.pop().unwrap().get_global_id(state, vm);
            let v2 = exp.pop().unwrap().get_global_id(state, vm);
            let obj1 = vm.get_obj_by_id(&v1).unwrap();
            let obj2 = vm.get_obj_by_id(&v2).unwrap();
            let object = obj1.borrow_mut().invoke("__add__", vec![obj2]);
            let res_id = vm.register_object(object);
            exp.push(SValue::GlobalId(res_id));
        }
        else if bytecode.get_operator() == &BytecodeOperator::Call {
            let ptr = vm as *mut FSRVM;
            let fn_id = match exp.pop().unwrap() {
                SValue::StackId(s) => {
                    if let Some(id) = state.get_var(&s.0) {
                        id.clone()
                    } else {
                        vm.get_global_obj_by_name(s.1).unwrap().clone()
                    }
                },
                SValue::GlobalId(id) => {
                    id
                },
            };

            
            
            if let ArgType::CallArgsNumber(n) = bytecode.get_arg() {
                let mut args = vec![];
                let n = *n;
                let mut i = 0;
                while i < n {
                    let a_id = exp.pop().unwrap().get_global_id(state, vm);
                    //let obj = vm.get_obj_by_id(&a_id).unwrap();
                    args.push(a_id);
                    i += 1;
                }
                self.call_stack.push(CallState::new());
                
                let fn_obj = vm.get_obj_by_id(&fn_id).unwrap();
                //fn_obj.borrow().call(stack, vm);
                let v = unsafe { &mut *ptr };
                let v = fn_obj.borrow().call(args, state, v).unwrap();
                exp.push(SValue::GlobalId(v));
            } else {

            }

            self.call_stack.pop();

        }
        else if bytecode.get_operator() == &BytecodeOperator::IfTest {
            let test_val = match exp.pop().unwrap() {
                SValue::StackId(s) => {
                    if let Some(id) = state.get_var(&s.0) {
                        id.clone()
                    } else {
                        vm.get_global_obj_by_name(s.1).unwrap().clone()
                    }
                },
                SValue::GlobalId(id) => {
                    id
                },
            };
            if test_val == vm.get_false_id() || test_val == vm.get_none_id() {
                if let ArgType::IfTestNext(n) = bytecode.get_arg() {
                    *ip += n.clone() as usize;
                    return true;
                }
            }
        }
        else if bytecode.get_operator() == &BytecodeOperator::WhileTest {
            let test_val = match exp.pop().unwrap() {
                SValue::StackId(s) => {
                    if let Some(id) = state.get_var(&s.0) {
                        id.clone()
                    } else {
                        vm.get_global_obj_by_name(s.1).unwrap().clone()
                    }
                },
                SValue::GlobalId(id) => {
                    id
                },
            };
            if test_val == vm.get_false_id() || test_val == vm.get_none_id() {
                if let ArgType::WhileTest(n) = bytecode.get_arg() {
                    *ip += n.clone() as usize;
                    return true;
                }
            }
        }
        else if bytecode.get_operator() == &BytecodeOperator::WhileBlockEnd {
            if let ArgType::WhileEnd(n) = bytecode.get_arg() {
                *ip -= n.clone() as usize + 1;
                return true;
            }
        }
        else {
            
        }

        return false
    }

    fn run_expr(&'a mut self, expr: &LinkedList<BytecodeArg>, ip: &mut usize, vm: &mut FSRVM<'a>) {
        let stack = self.get_cur_stack();
        let mut exp_stack = vec![];
        let ptr = stack as *mut CallState;
        let mut i = 0;
        for arg in expr {
            i += 1;
            if arg.get_operator() == &BytecodeOperator::Load {
                let s = unsafe {&mut *ptr};
                if let ArgType::Variable(id, name) = arg.get_arg() {
                    exp_stack.push(SValue::StackId((id.clone(), name)));
                }
                else if let ArgType::ConstInteger(id, i) = arg.get_arg() {
                    let int_const = Self::load_integer_const(i, vm);
                    s.insert_const(id, int_const.clone());
                    exp_stack.push(SValue::GlobalId(int_const));
                }
                else if let ArgType::ConstString(id, i) = arg.get_arg() {
                    let string_const = Self::load_string_const(i.clone(), vm);
                    s.insert_const(id, string_const.clone());
                    exp_stack.push(SValue::GlobalId(string_const));
                }
            } 
            // else if arg.get_operator() == &BytecodeOperator::Push {
            //     self.call_stack.push(CallState::new());
            // }
            // else if arg.get_operator() == &BytecodeOperator::Pop {
            //     self.call_stack.pop();
            // }
            else {
                let s = unsafe {&mut *ptr};
                self.process(&mut exp_stack, arg, s, ip, vm);
            }
        }

        *ip += 1;
    }

    pub fn start(&'a mut self, bytecode: Bytecode, vm: &'a mut FSRVM<'a>) {
        let mut ip = 0;
        let p = self as *mut Self;
        loop {
            let expr = match bytecode.get(ip) {
                Some(s) => s,
                None => {
                    break;
                }
            };
            let s = unsafe { &mut *p };
            s.run_expr(expr, &mut ip, vm);
        }
    }
}
