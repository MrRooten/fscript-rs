#![allow(clippy::ptr_arg)]

use std::{
    borrow::Cow, collections::{HashMap, HashSet}, sync::atomic::AtomicU64
};

use crate::{
    backend::{
        compiler::bytecode::{ArgType, Bytecode, BytecodeArg, BytecodeOperator},
        types::{
            base::{FSRObject, FSRRetValue, FSRValue},
            class::FSRClass,
            class_inst::FSRClassInst,
            fn_def::{FSRFn, FSRFnInner}, list::FSRList,
        },
    },
    utils::error::{FSRErrCode, FSRError},
};

use super::runtime::FSRVM;

pub struct CallState<'a> {
    var_map: HashMap<u64, u64>,
    const_map: HashMap<u64, u64>,
    reverse_ip: (usize, usize),
    args: Vec<u64>,
    cur_cls: Option<FSRClass<'a>>,
    ret_val: Option<u64>,
    exp: Option<Vec<SValue<'a>>>,
    #[allow(unused)]
    name: &'a str,
}

impl<'a> CallState<'a> {
    pub fn get_var(&self, id: &u64) -> Option<&u64> {
        self.var_map.get(id)
    }

    pub fn insert_var(&mut self, id: &u64, obj_id: u64) {
        if self.var_map.contains_key(id) {
            let origin_obj = FSRObject::id_to_obj(obj_id);
            origin_obj.ref_dec();
        }
        self.var_map.insert(*id, obj_id);
    }

    pub fn has_var(&self, id: &u64) -> bool {
        return self.var_map.get(id).is_some();
    }

    pub fn has_const(&self, id: &u64) -> bool {
        return self.const_map.get(id).is_some();
    }

    pub fn insert_const(&mut self, id: &u64, obj_id: u64) {
        self.const_map.insert(*id, obj_id);
    }

    pub fn set_reverse_ip(&mut self, ip: (usize, usize)) {
        self.reverse_ip = ip;
    }

    pub fn new(name: &'a str) -> Self {
        Self {
            var_map: HashMap::new(),
            const_map: HashMap::new(),
            reverse_ip: (0, 0),
            args: Vec::new(),
            cur_cls: None,
            ret_val: None,
            exp: None,
            name,
        }
    }
}

#[derive(Debug, Clone)]
enum SValue<'a> {
    Stack((u64, &'a String)),
    Attr((u64, &'a String)),
    Global(u64),
    #[allow(dead_code)]
    Object(FSRObject<'a>)
}

impl SValue<'_> {
    pub fn get_global_id(&self, thread: &FSRThreadRuntime) -> u64 {
        let state = thread.get_cur_stack();
        let vm = thread.get_vm();
        match self {
            SValue::Stack(s) => {
                if let Some(id) = state.get_var(&s.0) {
                    *id
                } else {
                    *vm.get_global_obj_by_name(s.1).unwrap()
                }
            }
            SValue::Global(id) => *id,
            SValue::Attr((id, _)) => *id,
            SValue::Object(obj) => {
                FSRObject::obj_to_id(obj)
            },
        }
    }

    #[allow(unused)]
    pub fn get_object(&self) -> Option<&FSRObject> {
        if let SValue::Object(obj) = self {
            return Some(obj);
        }

        None
    }
}

pub struct ThreadContext<'a> {
    exp: Vec<SValue<'a>>,
    ip: (usize, usize),
    vm: &'a mut FSRVM<'a>,
    is_attr: bool,
    last_if_test    : Vec<bool>
}

impl ThreadContext<'_> {
    pub fn false_last_if_test(&mut self) {
        let l = self.last_if_test.len() - 1;
        self.last_if_test[l] = false;
    }

    pub fn true_last_if_test(&mut self) {
        let l = self.last_if_test.len() - 1;
        self.last_if_test[l] = true;
    }

    pub fn peek_last_if_test(&self) -> bool {
        if self.last_if_test.is_empty() {
            return false;
        }

        self.last_if_test[self.last_if_test.len() - 1]
    }

    pub fn push_last_if_test(&mut self, test: bool) {
        self.last_if_test.push(test)
    }

    pub fn pop_last_if_test(&mut self) {
        self.last_if_test.pop();
    }
}

type BytecodeFn<'a> = fn(
    &mut FSRThreadRuntime<'a>,
    // exp: &mut Vec<SValue<'a>>,
    
    // ip: &mut (usize, usize),
    // vm: &mut FSRVM<'a>,
    // is_attr: &mut bool,
    context: &mut ThreadContext<'a>,
    bytecode: &BytecodeArg,
    bc: &'a Bytecode,
) -> Result<bool, FSRError>;

#[derive(Default)]
pub struct FSRThreadRuntime<'a> {
    call_stack: Vec<CallState<'a>>,
    bytecode_map: HashMap<BytecodeOperator, BytecodeFn<'a>>,
    vm_ptr      : Option<*mut FSRVM<'a>>
}



impl<'a, 'b:'a> FSRThreadRuntime<'a> {
    pub fn get_vm(&self) -> &FSRVM<'a> {
        unsafe { &*self.vm_ptr.unwrap() }
    }

    pub fn get_mut_vm(&mut self) -> &'a mut FSRVM<'a> {
        unsafe { &mut *self.vm_ptr.unwrap() }
    }

    pub fn new() -> FSRThreadRuntime<'a> {
        let mut map: HashMap<BytecodeOperator, BytecodeFn> = HashMap::new();
        map.insert(BytecodeOperator::Assign, Self::assign_process);
        map.insert(BytecodeOperator::BinaryAdd, Self::binary_add_process);
        map.insert(BytecodeOperator::BinaryDot, Self::binary_dot_process);
        map.insert(BytecodeOperator::BinaryMul, Self::binary_mul_process);
        map.insert(BytecodeOperator::Call, Self::call_process);
        map.insert(BytecodeOperator::IfTest, Self::if_test_process);
        map.insert(BytecodeOperator::WhileTest, Self::while_test_process);
        map.insert(BytecodeOperator::DefineFn, Self::define_fn);
        map.insert(BytecodeOperator::EndDefineFn, Self::end_define_fn);
        map.insert(BytecodeOperator::CompareTest, Self::compare_test);
        map.insert(BytecodeOperator::ReturnValue, Self::ret_value);
        map.insert(BytecodeOperator::WhileBlockEnd, Self::while_block_end);
        map.insert(BytecodeOperator::AssignArgs, Self::assign_args);
        map.insert(BytecodeOperator::ClassDef, Self::class_def);
        map.insert(BytecodeOperator::EndDefineClass, Self::end_class_def);
        map.insert(BytecodeOperator::LoadList, Self::load_list);
        map.insert(BytecodeOperator::Else, Self::else_process);
        map.insert(BytecodeOperator::ElseIf, Self::else_if_match);
        map.insert(BytecodeOperator::ElseIfTest, Self::else_if_test_process);
        map.insert(BytecodeOperator::IfBlockEnd, Self::if_end);

        Self {
            call_stack: vec![CallState::new("base")],
            bytecode_map: map,
            vm_ptr: None
        }
    }

    pub fn set_vm(&mut self, vm: &mut FSRVM<'a>) {
        self.vm_ptr = Some(vm as *mut FSRVM<'a>);
    }

    fn load_integer_const(i: &i64, vm: &mut FSRVM) -> u64 {
        let obj = FSRObject {
            obj_id: 0,
            value: FSRValue::Integer(*i),
            cls: "Integer",
            ref_count: AtomicU64::new(0),
        };

        vm.register_object(obj)
    }

    fn load_string_const(s: String, vm: &mut FSRVM<'a>) -> u64 {
        let obj = FSRObject {
            obj_id: 0,
            value: FSRValue::String(Cow::Owned(s)),
            cls: "String",
            ref_count: AtomicU64::new(0),
        };

        vm.register_object(obj)
    }

    pub fn get_cur_mut_stack(&mut self) -> &mut CallState<'a> {
        let l = self.call_stack.len();
        return self.call_stack.get_mut(l - 1).unwrap();
    }

    fn get_cur_stack(&self) -> &CallState<'a> {
        let l = self.call_stack.len();
        return self.call_stack.get(l - 1).unwrap();
    }

    fn compare(left: u64, right: u64, op: &str, thread: &mut Self) -> bool {
        let res;

        if op.eq(">") {
            res = FSRObject::invoke_method("__gt__", vec![left, right], thread);
        } else if op.eq("<") {
            res = FSRObject::invoke_method("__lt__", vec![left, right], thread);
        } else if op.eq(">=") {
            res = FSRObject::invoke_method("__gte__", vec![left, right], thread);
        } else if op.eq("<=") {
            res = FSRObject::invoke_method("__lte__", vec![left, right], thread);
        } else if op.eq("==") {
            res = FSRObject::invoke_method("__eq__", vec![left, right], thread);
        } else if op.eq("!=") {
            res = FSRObject::invoke_method("__neq__", vec![left, right], thread);
        } else {
            unimplemented!()
        }
        if let FSRRetValue::GlobalId(id) = &res.unwrap() {
            return id == &1;
        }
        unimplemented!()
    }

    fn pop_stack(&mut self, _vm: &mut FSRVM, escape: Option<HashSet<u64>>) {
        let v = self.call_stack.pop().unwrap();
        for kv in v.var_map {
            let obj = FSRObject::id_to_obj(kv.1);
            if let Some(l) = &escape {
                if l.contains(&kv.1) {
                    obj.ref_dec();
                    continue;
                }
            }
            
            obj.ref_dec();
            if obj.count_ref() == 0 {
                println!("Delete Object: {:#?}", obj);
            }
            //vm.check_delete(kv.1);
        }
    }

    fn assign_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        

        //Assign variable name
        let assign_id = match context.exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new("", FSRErrCode::EmptyExpStack));
            }
        };

        //To Assign obj or Dot Father object
        let obj_id = match context.exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new("", FSRErrCode::EmptyExpStack));
            }
        };

        if context.is_attr {
            let to_assign_obj = match context.exp.pop() {
                Some(s) => s,
                None => {
                    return Err(FSRError::new("", FSRErrCode::EmptyExpStack));
                }
            };

            let to_assign_obj_id = to_assign_obj.get_global_id(self);
            let father = obj_id.get_global_id(self);
            let father = FSRObject::id_to_mut_obj(father);
            if let SValue::Attr((_, attr_name)) = assign_id {
                let obj = FSRObject::id_to_mut_obj(to_assign_obj_id);
                obj.ref_add();
                // println!("{:#?}", obj);
                father.set_attr(attr_name, to_assign_obj_id);
            }
            context.is_attr = false;
        } else {
            let obj_id = obj_id.get_global_id(self);
            let to_assign_obj = FSRObject::id_to_mut_obj(obj_id);
            to_assign_obj.ref_add();
            let state = self.get_cur_mut_stack();
            if let SValue::Stack((var_id, _)) = assign_id {
                state.insert_var(&var_id, obj_id);
            }
        }
        // if let SValue::Global(id) = obj_id {
        //     let obj = FSRObject::id_to_mut_obj(id);
        //     //obj.ref_add();
        //     if let SValue::Stack(s) = &assign_id {
        //         if let Some(cur_cls) = &mut state.cur_cls {
        //             cur_cls.insert_attr_id(s.1, id);
        //             return Ok(false);
        //         }
        //     } else if let SValue::Attr((_, attr_name)) = &assign_id {
        //         let real_obj = match context.exp.pop() {
        //             Some(s) => s.get_global_id(self),
        //             None => {
        //                 return Err(FSRError::new("", FSRErrCode::EmptyExpStack));
        //             }
        //         };
        //         context.is_attr = false;
        //         if let FSRValue::ClassInst(inst) = &mut obj.value {
        //             let read = FSRObject::id_to_obj(real_obj);
        //             println!("{:#?}", read);
        //             inst.set_attr(attr_name, real_obj);
        //         }
        //         return Ok(false);
        //     } 
        //     state.insert_var(&assign_id.get_value(), id);
        // } else if let SValue::Stack(s_id) = obj_id {
        //     let id = match state.get_var(&s_id.0) {
        //         Some(s) => *s,
        //         None => *context.vm.get_global_obj_by_name(s_id.1).unwrap(),
        //     };
        //     if context.is_attr {
        //         let obj_id = context.exp.pop().unwrap().get_global_id(self);
        //         context.is_attr = false;
        //         let obj = FSRObject::id_to_mut_obj(obj_id);
        //         let to_assign_obj = FSRObject::id_to_mut_obj(id);
        //         to_assign_obj.ref_add();
        //         if let FSRValue::ClassInst(inst) = &mut obj.value {
        //             inst.set_attr(s_id.1, id);
        //         }
        //     } else {
        //         let obj = FSRObject::id_to_mut_obj(id);
        //         obj.ref_add();
        //         state.insert_var(&assign_id.get_value(), id);
        //     }
        // } else if let SValue::Object(object) = obj_id {
        //     let id: u64 = context.vm.register_object(object);
        //     state.insert_var(&assign_id.get_value(), id);
        // }

        Ok(false)
    }

    fn binary_add_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let _state = self.get_cur_mut_stack();
        let v1 = match context.exp.pop() {
            Some(s) => s.get_global_id(self),
            None => {
                return Err(FSRError::new(
                    "error in binary add 1",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };
        let v2 = match context.exp.pop() {
            Some(s) => s.get_global_id(self),
            None => {
                return Err(FSRError::new(
                    "error in binary mul 2",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };
        //let object = obj1.borrow_mut().invoke("__add__", vec![obj2]);
        let res = FSRObject::invoke_method("__add__", vec![v1, v2], self)?;
        match res {
            FSRRetValue::Value(object) => {
                let res_id = context.vm.register_object(object);
                context.exp.push(SValue::Global(res_id));
            }
            FSRRetValue::GlobalId(res_id) => {
                context.exp.push(SValue::Global(res_id));
            }
        };

        Ok(false)
    }

    fn binary_mul_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let _state = self.get_cur_mut_stack();
        let v1 = match context.exp.pop() {
            Some(s) => s.get_global_id(self),
            None => {
                return Err(FSRError::new(
                    "error in binary mul 1",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };
        let v2 = match context.exp.pop() {
            Some(s) => s.get_global_id(self),
            None => {
                return Err(FSRError::new(
                    "error in binary mul 2",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };
        //let object = obj1.borrow_mut().invoke("__add__", vec![obj2]);
        let res = FSRObject::invoke_method("__mul__", vec![v1, v2], self)?;
        match res {
            FSRRetValue::Value(object) => {
                let res_id = context.vm.register_object(object);
                context.exp.push(SValue::Global(res_id));
            }
            FSRRetValue::GlobalId(res_id) => {
                context.exp.push(SValue::Global(res_id));
            }
        };
        Ok(false)
    }

    fn binary_dot_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let _state = self.get_cur_mut_stack();
        let attr_id = match context.exp.pop().unwrap() {
            SValue::Stack(_) => unimplemented!(),
            SValue::Global(_) => unimplemented!(),
            SValue::Attr(id) => id,
            SValue::Object(_) => todo!(),
        };
        let dot_father = match context.exp.pop() {
            Some(s) => s.get_global_id(self),
            None => {
                return Err(FSRError::new(
                    "error in dot operator",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        if context.is_attr {
            // pop last father
            context.exp.pop();
        }

        let dot_father_obj = FSRObject::id_to_obj(dot_father);
        let name = attr_id.1;
        let id = dot_father_obj.get_attr(name, context.vm);
        if let Some(id) = id {
            // let obj = FSRObject::id_to_obj(id);
            // println!("{:#?}", obj);
            context.exp.push(SValue::Global(dot_father));
            context.exp.push(SValue::Attr((id, name)));
        } else {
            context.exp.push(SValue::Global(dot_father));
            context.exp.push(SValue::Attr((attr_id.0, name)));
        }

        context.is_attr = true;
        Ok(false)
    }

    #[inline]
    fn call_process_set_args(
        args_num: usize,
        thread: &Self,
        exp: &mut Vec<SValue<'a>>,
        args: &mut Vec<u64>
    ) {
        let mut i = 0;
        while i < args_num {
            let a_id = exp.pop().unwrap().get_global_id(thread);
            args.push(a_id);
            i += 1;
        }
    }

    #[inline]
    fn save_ip_to_callstate(
        &mut self,
        args_num: usize,
        exp: &mut Vec<SValue<'a>>,
        args: &mut Vec<u64>,
        ip: &mut (usize, usize),
    ) {
        Self::call_process_set_args(args_num, self, exp, args);
        let state = self.get_cur_mut_stack();
        state.set_reverse_ip(*ip);
        state.exp = Some(exp.clone());
    }

    fn call_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        //let ptr = vm as *mut FSRVM;
        let fn_id = match context.exp.pop().unwrap() {
            SValue::Stack(s) => {
                let state = self.get_cur_mut_stack();
                if let Some(id) = state.get_var(&s.0) {
                    *id
                } else {
                    *context.vm.get_global_obj_by_name(s.1).unwrap()
                }
            }
            SValue::Global(id) => id,
            SValue::Attr((id, _)) => id,
            SValue::Object(_) => todo!(),
        };

        if let ArgType::CallArgsNumber(n) = bytecode.get_arg() {
            let fn_obj = FSRObject::id_to_obj(fn_id);

            let mut args = vec![];
            let n = *n;
            

            if fn_obj.is_fsr_cls() {
                // New a object if fn_obj is fsr_cls
                
                let mut self_obj = FSRObject::new();
                self_obj.set_cls(fn_obj.get_fsr_class_name());
                self_obj.set_value(FSRValue::ClassInst(FSRClassInst::new(
                    fn_obj.get_fsr_class_name(),
                )));
                let self_id = context.vm.register_object(self_obj);

                // set self as fisrt args and call __new__ method to initialize object
                args.push(self_id);
                context.is_attr = true;
                Self::call_process_set_args(n, self, &mut context.exp, &mut args);
                let state = self.get_cur_mut_stack();
                state.set_reverse_ip(context.ip);
                state.exp = Some(context.exp.clone());
                self.call_stack.push(CallState::new("__new__"));
                context.exp.clear();
                let self_obj = FSRObject::id_to_obj(self_id);
                let self_new = self_obj.get_cls_attr("__new__", context.vm);


                if let Some(id) = self_new {
                    for arg in args.iter().rev() {
                        let obj = FSRObject::id_to_obj(*arg);
                        obj.ref_add();
                        self.get_cur_mut_stack().args.push(*arg);
                    }
                    let new_obj = FSRObject::id_to_obj(id);
                    let offset = new_obj.get_fsr_offset().1;
                    context.ip = (offset.0, 0);
                    return Ok(true);
                } else {
                    panic!("not existed method ")
                }
            } else if context.is_attr {
                
                let obj_id = context.exp.pop().unwrap().get_global_id(self);
                args.push(obj_id);

                context.is_attr = false;
                Self::call_process_set_args(n, self, &mut context.exp, &mut args);

                if fn_obj.is_fsr_function() {
                    let state = self.get_cur_mut_stack();
                    //Save callstate
                    state.set_reverse_ip(context.ip);
                    state.exp = Some(context.exp.clone());
                    self.call_stack.push(CallState::new("tmp"));
                    //Clear exp stack
                    context.exp.clear();

                    for arg in args.iter().rev() {
                        let obj = FSRObject::id_to_obj(*arg);
                        obj.ref_add();
                        self.get_cur_mut_stack().args.push(*arg);
                    }
                    let offset = fn_obj.get_fsr_offset().1;
                    context.ip = (offset.0, 0);
                    return Ok(true);
                } else {
                    let v = fn_obj.call(args, self).unwrap();

                    if let FSRRetValue::Value(v) = v {
                        let id = context.vm.register_object(v);
                        context.exp.push(SValue::Global(id));
                    } else if let FSRRetValue::GlobalId(id) = v {
                        context.exp.push(SValue::Global(id));
                    }
                }
            } else {
                self.save_ip_to_callstate(n, &mut context.exp, &mut args, &mut context.ip);
                self.call_stack.push(CallState::new("tmp2"));
                context.exp.clear();
                
                if fn_obj.is_fsr_function() {
                    for arg in args.iter().rev() {
                        let obj = FSRObject::id_to_obj(*arg);
                        obj.ref_add();
                        self.get_cur_mut_stack().args.push(*arg);
                    }
                    //let offset = fn_obj.get_fsr_offset();
                    let offset = fn_obj.get_fsr_offset().1;
                    context.ip = (offset.0, 0);
                    return Ok(true);
                } else {
                    let v = fn_obj.call(args, self).unwrap();
                    
                    if let FSRRetValue::Value(v) = v {
                        let id = context.vm.register_object(v);
                        context.exp.push(SValue::Global(id));
                        let vm = self.get_mut_vm();
                        let mut esp = HashSet::new();
                        esp.insert(id);
                        self.pop_stack(vm, Some(esp));
                    } else if let FSRRetValue::GlobalId(id) = v {
                        context.exp.push(SValue::Global(id));
                        let vm = self.get_mut_vm();
                        let mut esp = HashSet::new();
                        esp.insert(id);
                        self.pop_stack(vm, Some(esp));
                    }
                    
                }
            }
        }

        Ok(false)
    }

    fn if_test_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let state = self.get_cur_mut_stack();
        let test_val = match context.exp.pop().unwrap() {
            SValue::Stack(s) => {
                if let Some(id) = state.get_var(&s.0) {
                    *id
                } else {
                    *context.vm.get_global_obj_by_name(s.1).unwrap()
                }
            }
            SValue::Global(id) => id,
            _ => {
                unimplemented!()
            }
        };
        if test_val == context.vm.get_false_id() || test_val == context.vm.get_none_id() {
            if let ArgType::IfTestNext(n) = bytecode.get_arg() {
                context.ip = (context.ip.0 + n.0 as usize + 1_usize, 0);
                context.push_last_if_test(false);
                return Ok(true);
            }
        }
        context.push_last_if_test(true);
        Ok(false)
    }

    fn if_end(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        context.pop_last_if_test();
        Ok(false)
    }

    fn else_if_test_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let state = self.get_cur_mut_stack();
        let test_val = match context.exp.pop().unwrap() {
            SValue::Stack(s) => {
                if let Some(id) = state.get_var(&s.0) {
                    *id
                } else {
                    *context.vm.get_global_obj_by_name(s.1).unwrap()
                }
            }
            SValue::Global(id) => id,
            _ => {
                unimplemented!()
            }
        };
        if test_val == context.vm.get_false_id() || test_val == context.vm.get_none_id() {
            if let ArgType::IfTestNext(n) = bytecode.get_arg() {
                context.ip = (context.ip.0 + n.0 as usize + 1_usize, 0);
                context.false_last_if_test();
                return Ok(true);
            }
        }
        context.true_last_if_test();
        Ok(false)
    }

    fn else_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        if context.peek_last_if_test() {
            if let ArgType::IfTestNext(n) = bytecode.get_arg() {
                context.ip = (context.ip.0 + n.0 as usize + 1_usize, 0);
                return Ok(true);
            }
        } 
        
        context.false_last_if_test();
        Ok(false)
    }


    fn else_if_match(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        if context.peek_last_if_test() {
            if let ArgType::IfTestNext(n) = bytecode.get_arg() {
                context.ip = (context.ip.0 + n.0 as usize + 1_usize, 0);
                return Ok(true);
            }
        }
        context.false_last_if_test();
        Ok(false)
    }

    fn while_test_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let state = self.get_cur_mut_stack();
        let test_val = match context.exp.pop().unwrap() {
            SValue::Stack(s) => {
                if let Some(id) = state.get_var(&s.0) {
                    *id
                } else {
                    *context.vm.get_global_obj_by_name(s.1).unwrap()
                }
            }
            SValue::Global(id) => id,
            _ => {
                unimplemented!()
            }
        };
        if test_val == context.vm.get_false_id() || test_val == context.vm.get_none_id() {
            if let ArgType::WhileTest(n) = bytecode.get_arg() {
                context.ip = (context.ip.0 + *n as usize + 1, 0);
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn define_fn(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
        bc: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let state = self.get_cur_mut_stack();
        let name = match context.exp.pop().unwrap() {
            SValue::Stack(id) => id,
            SValue::Attr(_) => panic!(),
            SValue::Global(_) => panic!(),
            SValue::Object(_) => todo!(),
        };

        if let ArgType::DefineFnArgs(n, arg_len) = bytecode.get_arg() {
            let mut args = vec![];
            for _ in 0..*arg_len {
                let v = match context.exp.pop().unwrap() {
                    SValue::Stack(id) => id,
                    SValue::Attr(_) => panic!(),
                    SValue::Global(_) => panic!(),
                    SValue::Object(_) => todo!(),
                };
                args.push(v.1.to_string());
            }
            let fn_obj = FSRFn::from_fsr_fn("main", (context.ip.0 + 1, 0), args, bc);
            let fn_id = context.vm.register_object(fn_obj);
            if let Some(cur_cls) = &mut state.cur_cls {
                cur_cls.insert_attr_id(name.1, fn_id);
                context.ip = (context.ip.0 + *n as usize + 2, 0);
                return Ok(true);
            }
            context.vm.register_global_object(name.1, fn_id);
            context.ip = (context.ip.0 + *n as usize + 2, 0);
            return Ok(true);
        }
        Ok(false)
    }

    fn end_define_fn(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let vm = self.get_mut_vm();
        self.pop_stack(vm, None);
        let cur = self.get_cur_mut_stack();
        context.ip = (cur.reverse_ip.0, cur.reverse_ip.1 + 1);
        Ok(true)
    }

    fn compare_test(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        //let state = self.get_cur_stack();
        if let ArgType::Compare(op) = bytecode.get_arg() {
            let right = context.exp.pop().unwrap().get_global_id(self);
            let left = context.exp.pop().unwrap().get_global_id(self);
            let v = Self::compare(left, right, op, self);
            if v {
                context.exp.push(SValue::Global(context.vm.get_true_id()))
            } else {
                context.exp.push(SValue::Global(context.vm.get_false_id()))
            }
        }

        Ok(false)
    }

    fn ret_value(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        
        let v = context.exp.pop().unwrap().get_global_id(self);
        let mut esp = HashSet::new();
        esp.insert(v);
        let vm = self.get_mut_vm();
        self.pop_stack(vm, Some(esp));
        let cur = self.get_cur_mut_stack();
        //exp.push(SValue::GlobalId(v));
        cur.ret_val = Some(v);
        context.ip = (cur.reverse_ip.0, cur.reverse_ip.1);
        
        Ok(true)
    }

    fn while_block_end(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        if let ArgType::WhileEnd(n) = bytecode.get_arg() {
            context.ip = (context.ip.0 - *n as usize, 0);
            return Ok(true);
        }

        Ok(false)
    }

    fn assign_args(
        self: &mut FSRThreadRuntime<'a>,
        _context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let state = self.get_cur_mut_stack();
        let v = state.args.pop().unwrap();
        if let ArgType::Variable(s_id, _) = bytecode.get_arg() {
            state.insert_var(s_id, v);
        }
        Ok(false)
    }

    fn load_list(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        if let ArgType::LoadListNumber(n) = bytecode.get_arg() {
            let mut list = vec![];
            let n = *n;
            for _ in 0..n {
                let v = context.exp.pop().unwrap().get_global_id(self);
                list.push(v);
            }

            let list = FSRList::new_object(list);
            let id = context.vm.register_object(list);
            context.exp.push(SValue::Global(id));
        }

        Ok(false)
    }

    fn class_def(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let state = self.get_cur_mut_stack();
        let id = match context.exp.pop().unwrap() {
            SValue::Stack(i) => i,
            SValue::Attr(_) => panic!(),
            SValue::Global(_) => panic!(),
            SValue::Object(_) => todo!(),
        };

        let new_cls = FSRClass::new(id.1);
        state.cur_cls = Some(new_cls);

        Ok(false)
    }

    fn end_class_def(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        _: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let state = self.get_cur_mut_stack();
        let mut cls_obj = FSRObject::new();
        cls_obj.set_cls("Class");
        let obj = state.cur_cls.take().unwrap();
        let name = obj.get_name().to_string();
        cls_obj.set_value(FSRValue::Class(obj));
        let obj_id = context.vm.register_object(cls_obj);
        context.vm.register_global_object(&name, obj_id);
        Ok(false)
    }

    fn process(
        &mut self,
        context: &mut ThreadContext<'a>,
        bytecode: &'a BytecodeArg,
        bc: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let bc_op = self.bytecode_map.get(bytecode.get_operator()).unwrap();
        let v = bc_op(self, context, bytecode, bc)?;
        if v {
            return Ok(v);
        }

        Ok(false)
    }

    fn load_var(
        exp_stack: &mut Vec<SValue<'a>>,
        arg: &'a BytecodeArg,
        vm: &mut FSRVM<'a>,
        s: &mut CallState,
    ) {
        
        //*is_attr = false;
        if let ArgType::Variable(id, name) = arg.get_arg() {
            exp_stack.push(SValue::Stack((*id, name)));
        } else if let ArgType::ConstInteger(id, i) = arg.get_arg() {
            let int_const = Self::load_integer_const(i, vm);
            s.insert_const(id, int_const);
            exp_stack.push(SValue::Global(int_const));
        } else if let ArgType::ConstString(id, i) = arg.get_arg() {
            let string_const = Self::load_string_const(i.clone(), vm);
            s.insert_const(id, string_const);
            exp_stack.push(SValue::Global(string_const));
        } else if let ArgType::Attr(id, name) = arg.get_arg() {
            exp_stack.push(SValue::Attr((*id, name)));
        }
    }

    #[inline]
    fn set_exp_stack_ret(&mut self, exp_stack: &mut Vec<SValue<'a>>) {
        let stack = self.get_cur_mut_stack();
        if stack.exp.is_some() {
            *exp_stack = stack.exp.take().unwrap();
            stack.exp = None;
        }

        if stack.ret_val.is_some() {
            exp_stack.push(SValue::Global(stack.ret_val.unwrap()));
            stack.ret_val = None;
        }
    }

    fn run_expr(
        &mut self,
        expr: &'a Vec<BytecodeArg>,
        context: &mut ThreadContext<'a>,
        bc: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let mut v;
        
        while context.ip.1 < expr.len() {
            let arg = &expr[context.ip.1];
            // let t = format!("{:?} => {:?}", context.ip, arg);
            // println!("{:?}", context.exp);
            // println!("{}",t);
            context.ip.1 += 1;

            self.set_exp_stack_ret(&mut context.exp);

            if arg.get_operator() == &BytecodeOperator::Load {
                let state = self.get_cur_mut_stack();
                Self::load_var(&mut context.exp, arg, context.vm, state);
            } else {
                v = self.process(context, arg, bc)?;
                if self.get_cur_stack().ret_val.is_some() {
                    return Ok(true);
                }
                if v {
                    context.exp.clear();
                    return Ok(false);
                }
            }
        }

        context.ip.0 += 1;
        context.ip.1 = 0;
        context.exp.clear();
        context.is_attr = false;
        Ok(false)
    }

    pub fn start(&'a mut self, bytecode: &'a Bytecode, vm: &'a mut FSRVM<'a>) -> Result<(), FSRError> {
        self.set_vm(vm);
        let mut context = ThreadContext {
            exp: vec![],
            ip: (0, 0),
            vm,
            is_attr: false,
            last_if_test: vec![],
        };
        while let Some(expr) = bytecode.get(context.ip) {
            self.run_expr(expr, &mut context, bytecode)?;
        }

        Ok(())
    }

    pub fn call_fn(&mut self, fn_def: &'a FSRFnInner, args: Vec<u64>) -> Result<u64, FSRError> {
        let mut context = ThreadContext {
            exp: vec![],
            ip: fn_def.get_ip(),
            is_attr: false,
            vm: self.get_mut_vm(),
            last_if_test: vec![false],
        };
        {
            //self.save_ip_to_callstate(args.len(), &mut context.exp, &mut args, &mut context.ip);
            self.call_stack.push(CallState::new("__str__"));
            context.exp.clear();
            

            for arg in args.iter().rev() {
                self.get_cur_mut_stack().args.push(*arg);
            }
            //let offset = fn_obj.get_fsr_offset();
            let offset = fn_def.get_ip();
            context.ip = (offset.0, 0);
        }
        
        while let Some(expr) = fn_def.get_bytecode().get(context.ip) {
            let v = self.run_expr(expr, &mut context, fn_def.get_bytecode())?;
            if v {
                break;
            }
        }

        let cur = self.get_cur_mut_stack();
        let ret_val = cur.ret_val;
        // let s = ret_val.unwrap();
        // let v = FSRObject::id_to_obj(s);
        // println!("{:#?}", v);
        match ret_val {
            Some(s) => {
                Ok(s)
            },
            None => {
                Ok(0)
            }
        }
    }


}
