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
    AttrId((u64, &'a String)),
    GlobalId(u64),
}

impl SValue<'_> {
    pub fn get_value(&self) -> u64 {
        match self {
            SValue::StackId(i) => i.0.clone(),
            SValue::GlobalId(i) => i.clone(),
            SValue::AttrId(_) => todo!(),
        }
    }

    pub fn get_global_id(&self, state: &CallState, vm: &FSRVM) -> u64 {
        match self {
            SValue::StackId(s) => {
                if let Some(id) = state.get_var(&s.0) {
                    id.clone()
                } else {
                    vm.get_global_obj_by_name(s.1).unwrap().clone()
                }
            },
            SValue::GlobalId(id) => {
                id.clone()
            },
            SValue::AttrId(_) => todo!(),
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
        };

        let id = vm.register_object(obj);
        return id
    }

    fn load_string_const(s: String, vm: &mut FSRVM<'a>) -> u64 {
        let obj = FSRObject {
            obj_id: 0,
            value: FSRValue::String(s),
            cls: "String",
        };

        let id = vm.register_object(obj);        
        return id;
    }

    fn get_cur_stack(&mut self) -> &mut CallState {
        let l = self.call_stack.len();
        return self.call_stack.get_mut(l-1).unwrap();
    }

    fn compare(left: u64, right: u64, op: &str, vm: &mut FSRVM<'a>, state: &mut CallState) -> bool {
        let left_obj = vm.get_obj_by_id(&left).unwrap().borrow();
        let right_obj = vm.get_obj_by_id(&right).unwrap();
        let res;
        if op.eq(">") {
            res = FSRObject::invoke_method("__gt__", vec![left_obj, right_obj.borrow()], state, vm);
        } else if op.eq("<") {
            res = FSRObject::invoke_method("__lt__", vec![left_obj, right_obj.borrow()], state, vm);
        } else if op.eq(">=") {
            res = FSRObject::invoke_method("__gte__", vec![left_obj, right_obj.borrow()], state, vm);
        } else if op.eq("<=") {
            res = FSRObject::invoke_method("__lte__", vec![left_obj, right_obj.borrow()], state, vm);
        } else if op.eq("==") {
            res = FSRObject::invoke_method("__eq__", vec![left_obj, right_obj.borrow()], state, vm);
        } else if op.eq("!=") {
            res = FSRObject::invoke_method("__neq__", vec![left_obj, right_obj.borrow()], state, vm);
        } else {
            unimplemented!()
        }

        if let FSRValue::Bool(b) = res.unwrap().value {
            return b;
        }
        unimplemented!()
    }

    fn process(&mut self, exp: &mut Vec<SValue>, bytecode: &BytecodeArg, mut state: &mut CallState, ip: &mut usize, vm: &mut FSRVM<'a>, is_attr: &mut bool) -> bool {
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
            //let object = obj1.borrow_mut().invoke("__add__", vec![obj2]);
            let object = FSRObject::invoke_method("__add__", vec![obj1.borrow(),obj2.borrow()], state, vm).unwrap();
            let res_id = vm.register_object(object);
            exp.push(SValue::GlobalId(res_id));
        }
        else if bytecode.get_operator() == &BytecodeOperator::BinaryMul {
            let v1 = exp.pop().unwrap().get_global_id(state, vm);
            let v2 = exp.pop().unwrap().get_global_id(state, vm);
            let obj1 = vm.get_obj_by_id(&v1).unwrap();
            let obj2 = vm.get_obj_by_id(&v2).unwrap();
            //let object = obj1.borrow_mut().invoke("__add__", vec![obj2]);
            let object = FSRObject::invoke_method("__mul__", vec![obj1.borrow(),obj2.borrow()], state, vm).unwrap();
            let res_id = vm.register_object(object);
            exp.push(SValue::GlobalId(res_id));
        }
        else if bytecode.get_operator() == &BytecodeOperator::BinaryDot {
            let attr_id = match exp.pop().unwrap() {
                SValue::StackId(id) => unimplemented!(),
                SValue::GlobalId(_) => unimplemented!(),
                SValue::AttrId(id) => id,
            };
            let v1 = exp.pop().unwrap().get_global_id(state, vm);
            

            let obj1 = vm.get_obj_by_id(&v1).unwrap().borrow();
            let name = attr_id.1;
            let id = obj1.get_attr(name, vm).unwrap();
            exp.push(SValue::GlobalId(v1));
            exp.push(SValue::GlobalId(id));
            *is_attr = true;
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
                _ => {
                    unimplemented!()
                }
            };

            
            
            if let ArgType::CallArgsNumber(n) = bytecode.get_arg() {
                let mut args = vec![];
                let n = *n;
                let mut i = 0;
                if *is_attr {
                    let obj_id = exp.pop().unwrap().get_global_id(state, vm);
                    let obj = vm.get_obj_by_id(&obj_id).unwrap();
                    args.push(obj.borrow());
                    *is_attr = false;
                    while i < n {
                        let a_id = exp.pop().unwrap().get_global_id(state, vm);
                        let obj = vm.get_obj_by_id(&a_id).unwrap();
                        args.push(obj.borrow());
                        i += 1;
                    }
                    self.call_stack.push(CallState::new());
                    let fn_obj = vm.get_obj_by_id(&fn_id).unwrap();
                    let v = unsafe { &mut *ptr };
                    let v = fn_obj.borrow().call(args, state, v).unwrap();
                    let id = vm.register_object(v);
                    exp.push(SValue::GlobalId(id));
                    
                } else {
                    while i < n {
                        let a_id = exp.pop().unwrap().get_global_id(state, vm);
                        let obj = vm.get_obj_by_id(&a_id).unwrap();
                        args.push(obj.borrow());
                        i += 1;
                    }
                    
                    self.call_stack.push(CallState::new());
                    
                    let fn_obj = vm.get_obj_by_id(&fn_id).unwrap();
                    //fn_obj.borrow().call(stack, vm);
                    let v = unsafe { &mut *ptr };
                    let v = fn_obj.borrow().call(args, state, v).unwrap();
                    let id = vm.register_object(v);
                    exp.push(SValue::GlobalId(id));
                }
                
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
                _ => {
                    unimplemented!()
                }
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
                _ => {
                    unimplemented!()
                }
            };
            if test_val == vm.get_false_id() || test_val == vm.get_none_id() {
                if let ArgType::WhileTest(n) = bytecode.get_arg() {
                    *ip += n.clone() as usize;
                }
            }
        }
        else if bytecode.get_operator() == &BytecodeOperator::WhileBlockEnd {
            if let ArgType::WhileEnd(n) = bytecode.get_arg() {
                *ip -= n.clone() as usize;
                return true;
            }
        }
        else if bytecode.get_operator() == &BytecodeOperator::CompareTest {
            if let ArgType::Compare(op) = bytecode.get_arg() {
                let right = exp.pop().unwrap().get_global_id(state, vm);
                let left = exp.pop().unwrap().get_global_id(state, vm);
                let v = Self::compare(left, right, op, vm, state);
                if v {
                    exp.push(SValue::GlobalId(vm.get_true_id()))
                } else {
                    exp.push(SValue::GlobalId(vm.get_false_id()))
                }
            }
        }
        
        else {
            
        }

        return false
    }

    fn run_expr(&'a mut self, expr: &LinkedList<BytecodeArg>, ip: &mut usize, vm: &mut FSRVM<'a>) {
        
        let mut exp_stack = vec![];
        
        let mut i = 0;
        let mut is_attr = false;
        for arg in expr {
            let stack = self.get_cur_stack();
            let ptr = stack as *mut CallState;
            let s = unsafe {&mut *ptr};
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
                else if let ArgType::Attr(id, name) = arg.get_arg() {
                    exp_stack.push(SValue::AttrId((id.clone(), name)));
                }
            } 
            // else if arg.get_operator() == &BytecodeOperator::Push {
            //     self.call_stack.push(CallState::new());
            // }
            // else if arg.get_operator() == &BytecodeOperator::Pop {
            //     self.call_stack.pop();
            // }
            else {
                
                let v = self.process(&mut exp_stack, arg, s, ip, vm, &mut is_attr);
                if v {
                    return ;
                }
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
