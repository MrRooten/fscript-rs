use std::{
    cell::RefCell,
    collections::{HashMap, LinkedList},
    sync::atomic::AtomicU64,
};

use crate::{
    backend::{
        compiler::bytecode::{ArgType, Bytecode, BytecodeArg, BytecodeOperator},
        types::{
            base::{FSRObject, FSRRetValue, FSRValue},
            class::FSRClass,
            class_inst::FSRClassInst,
            fn_def::FSRFn,
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
            name: name,
        }
    }
}

#[derive(Debug, Clone)]
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
            }
            SValue::GlobalId(id) => id.clone(),
            SValue::AttrId((id, name)) => *id,
        }
    }
}

type BytecodeFn<'a> = fn (
    &mut FSRThreadRuntime<'a>,
    exp: &mut Vec<SValue<'a>>,
    bytecode: &BytecodeArg,
    state: &mut CallState<'a>,
    ip: &mut (usize, usize),
    vm: &mut FSRVM<'a>,
    is_attr: &mut bool,
) -> Result<bool, FSRError>;

pub struct FSRThreadRuntime<'a> {
    call_stack: Vec<CallState<'a>>,
    bytecode_map: HashMap<BytecodeOperator, BytecodeFn<'a>>
}

impl<'a> FSRThreadRuntime<'a> {
    pub fn new() -> FSRThreadRuntime<'a> {
        let mut map: HashMap<BytecodeOperator, BytecodeFn> = HashMap::new();
        map.insert(BytecodeOperator::Assign, FSRThreadRuntime::assign_process);
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

        Self {
            call_stack: vec![CallState::new("base")],
            bytecode_map: map,
        }
    }

    fn load_integer_const(i: &i64, vm: &mut FSRVM) -> u64 {
        let obj = FSRObject {
            obj_id: 0,
            value: FSRValue::Integer(i.clone()),
            cls: "Integer",
            ref_count: AtomicU64::new(0),
        };

        let id = vm.register_object(obj);
        return id;
    }

    fn load_string_const(s: String, vm: &mut FSRVM<'a>) -> u64 {
        let obj = FSRObject {
            obj_id: 0,
            value: FSRValue::String(s),
            cls: "String",
            ref_count: AtomicU64::new(0),
        };

        let id = vm.register_object(obj);
        return id;
    }

    fn get_cur_stack(&mut self) -> &mut CallState<'a> {
        let l = self.call_stack.len();
        return self.call_stack.get_mut(l - 1).unwrap();
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
        if let FSRRetValue::GlobalId(id) = &res.unwrap() {
            if id == &1 {
                return true;
            } else {
                return false;
            }
        }
        unimplemented!()
    }

    fn pop_stack(&mut self) {
        let v = self.call_stack.pop().unwrap();
        for kv in v.var_map {
            let obj = FSRObject::id_to_obj(kv.1);
            obj.ref_dec();
        }
    }

    fn assign_process(
        self: &mut FSRThreadRuntime<'a>,
        exp: &mut Vec<SValue<'a>>,
        bytecode: &BytecodeArg,
        state: &mut CallState<'a>,
        ip: &mut (usize, usize),
        vm: &mut FSRVM<'a>,
        is_attr: &mut bool,
    ) -> Result<bool, FSRError> {
        let assign_id = match exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new("", FSRErrCode::EmptyExpStack));
            }
        };
        let obj_id = match exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new("", FSRErrCode::EmptyExpStack));
            }
        };
        if let SValue::GlobalId(id) = obj_id {
            let obj = FSRObject::id_to_mut_obj(id);
            obj.ref_add();
            if let SValue::StackId(s) = &assign_id {
                if let Some(cur_cls) = &mut state.cur_cls {
                    cur_cls.insert_attr_id(s.1, id);
                    return Ok(false);
                }
            } else if let SValue::AttrId((_, attr_name)) = &assign_id {
                let real_obj = match exp.pop() {
                    Some(s) => s.get_global_id(state, vm),
                    None => {
                        return Err(FSRError::new("", FSRErrCode::EmptyExpStack));
                    }
                };
                *is_attr = false;
                if let FSRValue::ClassInst(inst) = &mut obj.value {
                    inst.set_attr(&attr_name, real_obj);
                }
                return Ok(false);
            }
            state.insert_var(&assign_id.get_value(), id);
        } else if let SValue::StackId(s_id) = obj_id {
            let id = match state.get_var(&s_id.0) {
                Some(s) => s.clone(),
                None => vm.get_global_obj_by_name(s_id.1).unwrap().clone(),
            };
            if *is_attr == true {
                let obj_id = exp.pop().unwrap().get_global_id(state, vm);
                *is_attr = false;
                let obj = FSRObject::id_to_mut_obj(obj_id);
                let to_assign_obj = FSRObject::id_to_mut_obj(id);
                to_assign_obj.ref_add();
                if let FSRValue::ClassInst(inst) = &mut obj.value {
                    inst.set_attr(s_id.1, id);
                }
            } else {
                let obj = FSRObject::id_to_mut_obj(id);
                obj.ref_add();
                state.insert_var(&assign_id.get_value(), id);
            }
        }

        return Ok(false);
    }

    fn binary_add_process(
        self: &mut FSRThreadRuntime<'a>,
        exp: &mut Vec<SValue<'a>>,
        bytecode: &BytecodeArg,
        state: &mut CallState<'a>,
        ip: &mut (usize, usize),
        vm: &mut FSRVM<'a>,
        is_attr: &mut bool,
    ) -> Result<bool, FSRError> {
        let v1 = match exp.pop() {
            Some(s) => s.get_global_id(state, vm),
            None => {
                return Err(FSRError::new(
                    "error in binary add 1",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };
        let v2 = match exp.pop() {
            Some(s) => s.get_global_id(state, vm),
            None => {
                return Err(FSRError::new(
                    "error in binary mul 2",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };
        //let object = obj1.borrow_mut().invoke("__add__", vec![obj2]);
        let res = FSRObject::invoke_method("__add__", vec![v1, v2], state, vm)?;
        match res {
            FSRRetValue::Value(object) => {
                let res_id = vm.register_object(object);
                exp.push(SValue::GlobalId(res_id));
            }
            FSRRetValue::GlobalId(res_id) => {
                exp.push(SValue::GlobalId(res_id));
            }
        };

        return Ok(false);
    }

    fn binary_mul_process(
        self: &mut FSRThreadRuntime<'a>,
        exp: &mut Vec<SValue<'a>>,
        bytecode: &BytecodeArg,
        state: &mut CallState<'a>,
        ip: &mut (usize, usize),
        vm: &mut FSRVM<'a>,
        is_attr: &mut bool,
    ) -> Result<bool, FSRError> {
        let v1 = match exp.pop() {
            Some(s) => s.get_global_id(state, vm),
            None => {
                return Err(FSRError::new(
                    "error in binary mul 1",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };
        let v2 = match exp.pop() {
            Some(s) => s.get_global_id(state, vm),
            None => {
                return Err(FSRError::new(
                    "error in binary mul 2",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };
        //let object = obj1.borrow_mut().invoke("__add__", vec![obj2]);
        let res = FSRObject::invoke_method("__mul__", vec![v1, v2], state, vm)?;
        match res {
            FSRRetValue::Value(object) => {
                let res_id = vm.register_object(object);
                exp.push(SValue::GlobalId(res_id));
            }
            FSRRetValue::GlobalId(res_id) => {
                exp.push(SValue::GlobalId(res_id));
            }
        };
        return Ok(false);
    }

    fn binary_dot_process(
        self: &mut FSRThreadRuntime<'a>,
        exp: &mut Vec<SValue<'a>>,
        bytecode: &BytecodeArg,
        state: &mut CallState<'a>,
        ip: &mut (usize, usize),
        vm: &mut FSRVM<'a>,
        is_attr: &mut bool,
    ) -> Result<bool, FSRError> {
        let attr_id = match exp.pop().unwrap() {
            SValue::StackId(i_) => unimplemented!(),
            SValue::GlobalId(_) => unimplemented!(),
            SValue::AttrId(id) => id,
        };
        let dot_father = match exp.pop() {
            Some(s) => s.get_global_id(state, vm),
            None => {
                return Err(FSRError::new(
                    "error in dot operator",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let dot_father_obj = FSRObject::id_to_obj(dot_father);
        let name = attr_id.1;
        let id = dot_father_obj.get_attr(name, vm);
        if id.is_none() {
            exp.push(SValue::GlobalId(dot_father));
            exp.push(SValue::AttrId((attr_id.0, name)));
        } else {
            let id = id.unwrap();
            exp.push(SValue::GlobalId(dot_father));
            exp.push(SValue::AttrId((id, name)));
        }

        *is_attr = true;

        return Ok(false);
    }

    fn call_process(
        self: &mut FSRThreadRuntime<'a>,
        exp: &mut Vec<SValue<'a>>,
        bytecode: &BytecodeArg,
        state: &mut CallState<'a>,
        ip: &mut (usize, usize),
        vm: &mut FSRVM<'a>,
        is_attr: &mut bool,
    ) -> Result<bool, FSRError> {
        let ptr = vm as *mut FSRVM;
        let fn_id = match exp.pop().unwrap() {
            SValue::StackId(s) => {
                if let Some(id) = state.get_var(&s.0) {
                    id.clone()
                } else {
                    vm.get_global_obj_by_name(s.1).unwrap().clone()
                }
            }
            SValue::GlobalId(id) => id,
            SValue::AttrId((id, fname)) => id,
        };

        if let ArgType::CallArgsNumber(n) = bytecode.get_arg() {
            let fn_obj = FSRObject::id_to_obj(fn_id);

            let mut args = vec![];
            let n = *n;
            let mut i = 0;
            if fn_obj.is_fsr_cls() {
                let mut self_obj = FSRObject::new();
                self_obj.set_cls(fn_obj.get_fsr_class_name());
                self_obj.set_value(FSRValue::ClassInst(FSRClassInst::new(
                    fn_obj.get_fsr_class_name(),
                )));
                let self_id = vm.register_object(self_obj);
                args.push(self_id);
                *is_attr = true;
                while i < n {
                    let a_id = exp.pop().unwrap().get_global_id(state, vm);
                    args.push(a_id);
                    i += 1;
                }
                state.set_reverse_ip(*ip);
                state.exp = Some(exp.clone());
                self.call_stack.push(CallState::new("__new__"));
                exp.clear();

                let self_obj = FSRObject::id_to_obj(self_id);
                let self_new = self_obj.get_cls_attr("__new__", vm);
                if let Some(id) = self_new {
                    for arg in args.iter().rev() {
                        self.get_cur_stack().args.push(*arg);
                    }
                    let new_obj = FSRObject::id_to_obj(id);

                    let offset = new_obj.get_fsr_offset().1;
                    *ip = (offset.0 as usize, 0);
                    return Ok(true);
                } else {
                    panic!("not existed method ")
                }
            } else if *is_attr {
                let obj_id = exp.pop().unwrap().get_global_id(state, vm);
                args.push(obj_id);
                *is_attr = false;
                while i < n {
                    let a_id = exp.pop().unwrap().get_global_id(state, vm);
                    args.push(a_id);
                    i += 1;
                }

                let v = unsafe { &mut *ptr };
                if fn_obj.is_fsr_function() {
                    state.set_reverse_ip(*ip);
                    state.exp = Some(exp.clone());
                    self.call_stack.push(CallState::new("tmp"));
                    exp.clear();
                    for arg in args.iter().rev() {
                        self.get_cur_stack().args.push(*arg);
                    }
                    let offset = fn_obj.get_fsr_offset().1;
                    *ip = (offset.0 as usize, 0);
                    return Ok(true);
                } else {
                    let v = fn_obj.call(args, state, v).unwrap();

                    if let FSRRetValue::Value(v) = v {
                        let id = vm.register_object(v);
                        exp.push(SValue::GlobalId(id));
                    } else if let FSRRetValue::GlobalId(id) = v {
                        exp.push(SValue::GlobalId(id));
                    }
                }
            } else {
                while i < n {
                    let a_id = exp.pop().unwrap().get_global_id(state, vm);
                    // println!("load object as args: {:?}", FSRObject::id_to_obj(a_id));
                    args.push(a_id);
                    i += 1;
                }
                state.set_reverse_ip(*ip);
                state.exp = Some(exp.clone());
                self.call_stack.push(CallState::new("tmp2"));
                exp.clear();

                let v = unsafe { &mut *ptr };
                if fn_obj.is_fsr_function() {
                    for arg in args.iter().rev() {
                        self.get_cur_stack().args.push(*arg);
                    }
                    //let offset = fn_obj.get_fsr_offset();
                    let offset = fn_obj.get_fsr_offset().1;
                    *ip = (offset.0 as usize, 0);
                    return Ok(true);
                } else {
                    let v = fn_obj.call(args, state, v).unwrap();
                    if let FSRRetValue::Value(v) = v {
                        let id = vm.register_object(v);
                        exp.push(SValue::GlobalId(id));
                    } else if let FSRRetValue::GlobalId(id) = v {
                        exp.push(SValue::GlobalId(id));
                    }

                    self.pop_stack();
                }
            }
        } else {
        }

        return Ok(false);
    }

    fn if_test_process(
        self: &mut FSRThreadRuntime<'a>,
        exp: &mut Vec<SValue<'a>>,
        bytecode: &BytecodeArg,
        state: &mut CallState<'a>,
        ip: &mut (usize, usize),
        vm: &mut FSRVM<'a>,
        is_attr: &mut bool,
    ) -> Result<bool, FSRError> {
        let test_val = match exp.pop().unwrap() {
            SValue::StackId(s) => {
                if let Some(id) = state.get_var(&s.0) {
                    id.clone()
                } else {
                    vm.get_global_obj_by_name(s.1).unwrap().clone()
                }
            }
            SValue::GlobalId(id) => id,
            _ => {
                unimplemented!()
            }
        };
        if test_val == vm.get_false_id() || test_val == vm.get_none_id() {
            if let ArgType::IfTestNext(n) = bytecode.get_arg() {
                *ip = (ip.0 + n.clone() as usize, 0);
                return Ok(true);
            }
        }

        return Ok(false);
    }

    fn while_test_process(
        self: &mut FSRThreadRuntime<'a>,
        exp: &mut Vec<SValue<'a>>,
        bytecode: &BytecodeArg,
        state: &mut CallState<'a>,
        ip: &mut (usize, usize),
        vm: &mut FSRVM<'a>,
        is_attr: &mut bool,
    ) -> Result<bool, FSRError> {
        let test_val = match exp.pop().unwrap() {
            SValue::StackId(s) => {
                if let Some(id) = state.get_var(&s.0) {
                    id.clone()
                } else {
                    vm.get_global_obj_by_name(s.1).unwrap().clone()
                }
            }
            SValue::GlobalId(id) => id,
            _ => {
                unimplemented!()
            }
        };
        if test_val == vm.get_false_id() || test_val == vm.get_none_id() {
            if let ArgType::WhileTest(n) = bytecode.get_arg() {
                *ip = (ip.0 + n.clone() as usize + 1, 0);
                return Ok(true);
            }
        }
        return Ok(false);
    }

    fn define_fn(
        &mut self,
        exp: &mut Vec<SValue<'a>>,
        bytecode: &BytecodeArg,
        state: &mut CallState<'a>,
        ip: &mut (usize, usize),
        vm: &mut FSRVM<'a>,
        is_attr: &mut bool,
    ) -> Result<bool, FSRError> {
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
            let fn_obj = FSRFn::from_fsr_fn("main", (ip.0 as u64 + 1, 0), args);
            let fn_id = vm.register_object(fn_obj);
            if let Some(cur_cls) = &mut state.cur_cls {
                cur_cls.insert_attr_id(name.1, fn_id);
                *ip = (ip.0 + *n as usize + 2, 0);
                return Ok(true);
            }
            vm.register_global_object(name.1, fn_id);
            *ip = (ip.0 + *n as usize + 2, 0);
            return Ok(true);
        }
        return Ok(false);
    }

    fn end_define_fn(
        self: &mut FSRThreadRuntime<'a>,
        exp: &mut Vec<SValue<'a>>,
        bytecode: &BytecodeArg,
        state: &mut CallState<'a>,
        ip: &mut (usize, usize),
        vm: &mut FSRVM<'a>,
        is_attr: &mut bool,
    ) -> Result<bool, FSRError> {
        self.pop_stack();
        let cur = self.get_cur_stack();
        *ip = (cur.reverse_ip.0, cur.reverse_ip.1 + 1);
        return Ok(true);
    }

    fn compare_test(
        &mut self,
        exp: &mut Vec<SValue<'a>>,
        bytecode: &BytecodeArg,
        state: &mut CallState<'a>,
        ip: &mut (usize, usize),
        vm: &mut FSRVM<'a>,
        is_attr: &mut bool,
    ) -> Result<bool, FSRError> {
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

        return Ok(false);
    }

    fn ret_value(
        self: &mut FSRThreadRuntime<'a>,
        exp: &mut Vec<SValue<'a>>,
        bytecode: &BytecodeArg,
        state: &mut CallState<'a>,
        ip: &mut (usize, usize),
        vm: &mut FSRVM<'a>,
        is_attr: &mut bool,
    ) -> Result<bool, FSRError> {
        let v = exp.pop().unwrap().get_global_id(state, vm);
        self.pop_stack();
        let cur = self.get_cur_stack();
        //exp.push(SValue::GlobalId(v));
        cur.ret_val = Some(v);
        *ip = (cur.reverse_ip.0, cur.reverse_ip.1);
        return Ok(true);
    }

    fn while_block_end(
        self: &mut FSRThreadRuntime<'a>,
        exp: &mut Vec<SValue<'a>>,
        bytecode: &BytecodeArg,
        state: &mut CallState<'a>,
        ip: &mut (usize, usize),
        vm: &mut FSRVM<'a>,
        is_attr: &mut bool,
    ) -> Result<bool, FSRError> {
        if let ArgType::WhileEnd(n) = bytecode.get_arg() {
            *ip = (ip.0 - n.clone() as usize, 0);
            return Ok(true);
        }

        return Ok(false);
    }

    fn assign_args(
        self: &mut FSRThreadRuntime<'a>,
        exp: &mut Vec<SValue<'a>>,
        bytecode: &BytecodeArg,
        state: &mut CallState<'a>,
        ip: &mut (usize, usize),
        vm: &mut FSRVM<'a>,
        is_attr: &mut bool,
    ) -> Result<bool, FSRError> {
        let v = state.args.pop().unwrap();
        if let ArgType::Variable(s_id, _) = bytecode.get_arg() {
            state.insert_var(s_id, v);
        }
        return Ok(false);
    }

    fn class_def(
        self: &mut FSRThreadRuntime<'a>,
        exp: &mut Vec<SValue<'a>>,
        bytecode: &BytecodeArg,
        state: &mut CallState<'a>,
        ip: &mut (usize, usize),
        vm: &mut FSRVM<'a>,
        is_attr: &mut bool,
    ) -> Result<bool, FSRError> {
        let id = match exp.pop().unwrap() {
            SValue::StackId(i) => i,
            SValue::AttrId(_) => panic!(),
            SValue::GlobalId(_) => panic!(),
        };

        let new_cls = FSRClass::new(id.1);
        state.cur_cls = Some(new_cls);

        return Ok(false);
    }

    fn end_class_def(
        self: &mut FSRThreadRuntime<'a>,
        exp: &mut Vec<SValue<'a>>,
        bytecode: &BytecodeArg,
        state: &mut CallState<'a>,
        ip: &mut (usize, usize),
        vm: &mut FSRVM<'a>,
        is_attr: &mut bool,
    ) -> Result<bool, FSRError> {
        let mut cls_obj = FSRObject::new();
        cls_obj.set_cls("Class");
        let obj = state.cur_cls.take().unwrap();
        let name = obj.get_name().to_string();
        cls_obj.set_value(FSRValue::Class(obj));
        let obj_id = vm.register_object(cls_obj);
        vm.register_global_object(&name, obj_id);
        return Ok(false);
    }

    fn process(
        &mut self,
        exp: &mut Vec<SValue<'a>>,
        bytecode: &BytecodeArg,
        state: &mut CallState<'a>,
        ip: &mut (usize, usize),
        vm: &mut FSRVM<'a>,
        is_attr: &mut bool,
    ) -> Result<bool, FSRError> {
        let bc_op = self.bytecode_map.get(bytecode.get_operator()).unwrap();
        let v = bc_op(self, exp, bytecode, state, ip, vm, is_attr)?;
        if v == true {
            return Ok(v);
        }

        return Ok(false);
    }

    fn run_expr(
        &'a mut self,
        expr: &'a Vec<BytecodeArg>,
        ip: &mut (usize, usize),
        vm: &mut FSRVM<'a>,
    ) -> Result<(), FSRError> {
        let mut exp_stack = vec![];

        let mut is_attr = false;
        let mut v = false;
        while ip.1 < expr.len() {
            let arg = &expr[ip.1];
            ip.1 += 1;
            //println!("IP: {:?} => {:?}", ip, arg);
            let stack = self.get_cur_stack();
            if stack.exp.is_some() {
                exp_stack = stack.exp.take().unwrap();
                stack.exp = None;
            }

            if stack.ret_val.is_some() {
                exp_stack.push(SValue::GlobalId(stack.ret_val.unwrap()));
                stack.ret_val = None;
            }
            let ptr = stack as *mut CallState;
            let s = unsafe { &mut *ptr };
            if arg.get_operator() == &BytecodeOperator::Load {
                is_attr = false;
                let s = unsafe { &mut *ptr };
                if let ArgType::Variable(id, name) = arg.get_arg() {
                    exp_stack.push(SValue::StackId((id.clone(), name)));
                } else if let ArgType::ConstInteger(id, i) = arg.get_arg() {
                    let int_const = Self::load_integer_const(i, vm);
                    s.insert_const(id, int_const.clone());
                    exp_stack.push(SValue::GlobalId(int_const));
                } else if let ArgType::ConstString(id, i) = arg.get_arg() {
                    let string_const = Self::load_string_const(i.clone(), vm);
                    s.insert_const(id, string_const.clone());
                    exp_stack.push(SValue::GlobalId(string_const));
                } else if let ArgType::Attr(id, name) = arg.get_arg() {
                    exp_stack.push(SValue::AttrId((id.clone(), name)));
                }
            } else {
                v = self.process(&mut exp_stack, arg, s, ip, vm, &mut is_attr)?;
                if v == true {
                    return Ok(());
                }
            }
        }

        ip.0 += 1;
        ip.1 = 0;
        return Ok(());
    }

    pub fn start(&'a mut self, bytecode: &'a Bytecode, vm: &'a mut FSRVM<'a>) {
        let mut ip = (0, 0);
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
