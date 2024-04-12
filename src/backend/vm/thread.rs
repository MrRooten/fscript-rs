use std::{cell::RefCell, collections::{HashMap, LinkedList}, sync::atomic::AtomicU64};

use crate::backend::{
    compiler::bytecode::{ArgType, Bytecode, BytecodeArg, BytecodeOperator},
    types::{base::{FSRObject, FSRValue}, class::FSRClass, fn_def::FSRFn},
};

use super::runtime::FSRVM;

pub struct CallState<'a> {
    var_map: HashMap<u64, u64>,
    const_map   : HashMap<u64, u64>,
    reverse_ip  : usize,
    args        : Vec<u64>,
    ret_val     : Option<u64>,
    cur_cls     : Option<FSRClass<'a>>,
}

impl CallState<'_> {
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

    pub fn set_reverse_ip(&mut self, ip: usize) {
        self.reverse_ip = ip;
    }

    pub fn new() -> Self {
        Self {
            var_map: HashMap::new(),
            const_map: HashMap::new(),
            reverse_ip: 0,
            args: Vec::new(),
            ret_val: None,
            cur_cls: None,
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

pub struct FSRThreadRuntime<'a> {
    call_stack: Vec<CallState<'a>>,
}

impl<'a> FSRThreadRuntime<'a> {
    pub fn new() -> FSRThreadRuntime<'a> {
        Self {
            call_stack: vec![CallState::new()],
        }
    }

    fn load_integer_const(i: &i64, vm: &mut FSRVM) -> u64 {
        let obj = FSRObject {
            obj_id: 0,
            value: FSRValue::Integer(i.clone()),
            cls: "Integer",
            ref_count: AtomicU64::new(0)
        };

        let id = vm.register_object(obj);
        return id
    }

    fn load_string_const(s: String, vm: &mut FSRVM<'a>) -> u64 {
        let obj = FSRObject {
            obj_id: 0,
            value: FSRValue::String(s),
            cls: "String",
            ref_count: AtomicU64::new(0)
        };

        let id = vm.register_object(obj);        
        return id;
    }

    fn get_cur_stack(&mut self) -> &mut CallState<'a> {
        let l = self.call_stack.len();
        return self.call_stack.get_mut(l-1).unwrap();
    }

    fn compare(left: u64, right: u64, op: &str, vm: &mut FSRVM<'a>, state: &mut CallState) -> bool {
        let res;
        if op.eq(">") {
            res = FSRObject::invoke_method("__gt__", vec![left, right], state, vm);
        } else if op.eq("<") {
            res = FSRObject::invoke_method("__lt__", vec![left, right], state, vm);
        } else if op.eq(">=") {
            res = FSRObject::invoke_method("__gte__", vec![left, right], state, vm);
        } else if op.eq("<=") {
            res = FSRObject::invoke_method("__lte__", vec![left, right], state, vm);
        } else if op.eq("==") {
            res = FSRObject::invoke_method("__eq__", vec![left, right], state, vm);
        } else if op.eq("!=") {
            res = FSRObject::invoke_method("__neq__", vec![left, right], state, vm);
        } else {
            unimplemented!()
        }

        if let FSRValue::Bool(b) = res.unwrap().value {
            return b;
        }
        unimplemented!()
    }

    fn process(&mut self, exp: &mut Vec<SValue<'a>>, bytecode: &BytecodeArg, mut state: &mut CallState<'a>, ip: &mut usize, vm: &mut FSRVM<'a>, is_attr: &mut bool) -> bool {
        if bytecode.get_operator() == &BytecodeOperator::Assign {
            //if let ArgType::Variable(v, name) = bytecode.get_arg() {
            let assign_id = exp.pop().unwrap();
            let obj_id = exp.pop().unwrap();
            if let SValue::GlobalId(id) = obj_id {
                if let SValue::StackId(s) = &assign_id {
                    if let Some(cur_cls) = &mut state.cur_cls {
                        cur_cls.insert_attr_id(s.1, id);
                        return false;
                    }
                }
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
            //let object = obj1.borrow_mut().invoke("__add__", vec![obj2]);
            let object = FSRObject::invoke_method("__add__", vec![v1, v2], state, vm).unwrap();
            let res_id = vm.register_object(object);
            exp.push(SValue::GlobalId(res_id));
        }
        else if bytecode.get_operator() == &BytecodeOperator::BinaryMul {
            let v1 = exp.pop().unwrap().get_global_id(state, vm);
            let v2 = exp.pop().unwrap().get_global_id(state, vm);
            //let object = obj1.borrow_mut().invoke("__add__", vec![obj2]);
            let object = FSRObject::invoke_method("__mul__", vec![v1, v2], state, vm).unwrap();
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
            

            let obj1 = FSRObject::id_to_obj(v1);
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
                    args.push(obj_id);
                    *is_attr = false;
                    while i < n {
                        let a_id = exp.pop().unwrap().get_global_id(state, vm);
                        args.push(a_id);
                        i += 1;
                    }
                    state.set_reverse_ip(*ip);
                    self.call_stack.push(CallState::new());
                    let fn_obj = FSRObject::id_to_obj(fn_id);
                    let v = unsafe { &mut *ptr };
                    if fn_obj.is_fsr_function() {
                        let offset = fn_obj.get_fsr_offset().1;
                        *ip = offset as usize;
                    } else {
                        let v = fn_obj.call(args, state, v).unwrap();
                    
                        let id = vm.register_object(v);
                        exp.push(SValue::GlobalId(id));
                        self.call_stack.pop();
                    }
                    
                } else {
                    while i < n {
                        let a_id = exp.pop().unwrap().get_global_id(state, vm);
                        args.push(a_id);
                        i += 1;
                    }
                    state.set_reverse_ip(*ip);
                    self.call_stack.push(CallState::new());
                    
                    let fn_obj = FSRObject::id_to_obj(fn_id);

                    let v = unsafe { &mut *ptr };
                    if fn_obj.is_fsr_function() {
                        for arg in args {
                            self.get_cur_stack().args.push(arg);
                        }
                        let offset = fn_obj.get_fsr_offset();
                        let offset = fn_obj.get_fsr_offset().1;
                        *ip = offset as usize;
                        return true;
                    } else {
                        let v = fn_obj.call(args, state, v).unwrap();
                    
                        let id = vm.register_object(v);
                        exp.push(SValue::GlobalId(id));
                        self.call_stack.pop();
                    }
                    
                }
                
            } else {

            }

            
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
        else if bytecode.get_operator() == &BytecodeOperator::DefineFn {
            let name = match exp.pop().unwrap() {
                SValue::StackId(id) => id,
                SValue::AttrId(_) => panic!(),
                SValue::GlobalId(_) => panic!(),
            };


            if let ArgType::DefineFnArgs(n, arg_len) = bytecode.get_arg() {
                let mut args = vec![];
                for i in 0..*arg_len {
                    let v = match exp.pop().unwrap() {
                        SValue::StackId(id) => id,
                        SValue::AttrId(_) => panic!(),
                        SValue::GlobalId(_) => panic!(),
                    };
                    args.push(v.1.to_string());
                }
                let fn_obj = FSRFn::from_fsr_fn("main", (*ip + 1) as u64, args);
                let fn_id = vm.register_object(fn_obj);
                if let Some(cur_cls) = &mut state.cur_cls  {
                    cur_cls.insert_attr_id(name.1, fn_id);
                    *ip += *n as usize + 2;
                    return true;
                }
                vm.register_global_object(name.1, fn_id);
                *ip += *n as usize + 2;
                return true;
            }
            
        }
        else if bytecode.get_operator() == &BytecodeOperator::AssignArgs {
            let v = state.args.pop().unwrap();
            if let ArgType::Variable(s_id, name) = bytecode.get_arg() {
                state.insert_var(s_id, v);
            }
        }
        else if bytecode.get_operator() == &BytecodeOperator::EndDefineFn {
            self.call_stack.pop();
            let cur = self.get_cur_stack();
            *ip = cur.reverse_ip + 1;
            return true;
        }
        else if bytecode.get_operator() == &BytecodeOperator::ReturnValue {
            self.call_stack.pop();
            let cur = self.get_cur_stack();
            let v = exp.pop().unwrap().get_global_id(state, vm);
            exp.push(SValue::GlobalId(v));
            *ip = cur.reverse_ip + 1;
            return true;
        }
        else if bytecode.get_operator() == &BytecodeOperator::ClassDef {
            let id = match exp.pop().unwrap() {
                SValue::StackId(i) => i,
                SValue::AttrId(_) => panic!(),
                SValue::GlobalId(_) => panic!(),
            };

            let new_cls = FSRClass::new(id.1);
            state.cur_cls = Some(new_cls);
        }
        else if bytecode.get_operator() == &BytecodeOperator::EndDefineClass {
            let mut cls_obj = FSRObject::new();
            cls_obj.set_cls("Class");
            let obj = state.cur_cls.take().unwrap();
            let name = obj.get_name().to_string();
            cls_obj.set_value(FSRValue::Class(obj));
            let obj_id = vm.register_object(cls_obj);
            vm.register_global_object(&name, obj_id);
            return false;
        }
        else {
            
        }

        return false
    }

    fn run_expr(&'a mut self, expr: &'a LinkedList<BytecodeArg>, ip: &mut usize, vm: &mut FSRVM<'a>) {
        
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

    pub fn start(&'a mut self, bytecode: &'a Bytecode, vm: &'a mut FSRVM<'a>) {
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
