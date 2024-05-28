#![allow(clippy::ptr_arg)]

use std::{collections::hash_set::Iter, rc::Rc};
#[cfg(feature = "perf")]
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    ops::AddAssign,
    sync::atomic::AtomicU64,
    time::{Duration, Instant},
};

#[cfg(not(feature = "perf"))]
use std::{
    borrow::Cow,
    collections::HashSet,
    sync::atomic::AtomicU64,
};


use crate::{
    backend::{
        compiler::bytecode::{ArgType, BinaryOffset, Bytecode, BytecodeArg, BytecodeOperator},
        types::{
            base::{FSRGlobalObjId, FSRObject, FSRRetValue, FSRValue},
            class::FSRClass,
            class_inst::FSRClassInst,
            fn_def::{FSRFn, FSRFnInner},
            list::FSRList,
            module::FSRModule,
        },
    },
    utils::error::{FSRErrCode, FSRError},
};

use super::runtime::FSRVM;

#[allow(unused)]
struct TempHashMap {
    vs      : Vec<u64>,
    iters   : HashSet<u64>,
    iter    : u64
}

struct TempIterator<'a> {
    vs      : &'a Vec<u64>,
    iter    : Iter<'a, u64>
}

#[allow(unused)]
impl TempHashMap {
    #[inline(always)]
    pub fn get(&self, i: &u64) -> Option<&u64> {
        self.vs.get(*i as usize)
    }

    #[inline(always)]
    pub fn insert(&mut self, i: u64, v: u64) {
        self.iters.insert(i);
        self.vs[i as usize] = v;
    }

    #[inline(always)]
    pub fn contains_key(&self, i: &u64) -> bool {
        if self.vs.get(*i as usize).is_none() {
            return false;
        }

        self.vs[*i as usize] != 0
    }

    pub fn new() -> Self {
        Self {
            vs: vec![0;100],
            iters: HashSet::new(),
            iter: 0,
        }
    }

    pub fn iter(&self) -> TempIterator {
        TempIterator {
            vs: &self.vs,
            iter: self.iters.iter(),
        }
    }
}



impl<'a> Iterator for TempIterator<'a> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(s) = self.iter.next() {
            let v = *s as usize;
            Some(*self.vs.get(v).unwrap())
        } else {
            None
        }

    }
}

pub struct CallState<'a> {
    var_map:  TempHashMap,
    const_map: TempHashMap,
    reverse_ip: (usize, usize),
    args: Vec<u64>,
    cur_cls: Option<FSRClass<'a>>,
    ret_val: Option<u64>,
    exp: Option<Vec<Rc<SValue<'a>>>>,
    #[allow(unused)]
    name: Cow<'a, str>,
}

impl<'a> CallState<'a> {
    pub fn get_var(&self, id: &u64) -> Option<&u64> {
        if let Some(s) = self.var_map.get(id) {
            if s == &0 {
                return None
            }

            return Some(s)
        }

        None
        
    }

    pub fn insert_var(&mut self, id: &u64, obj_id: u64) {
        if self.var_map.contains_key(id) {
            let to_be_dec = self.var_map.get(id).unwrap();
            let origin_obj = FSRObject::id_to_obj(*to_be_dec);
            origin_obj.ref_dec();

            if origin_obj.count_ref() == 0 {
                //println!("Will Drop: {:#?}", origin_obj);
                FSRObject::drop_object(*to_be_dec);
            }
        }
        self.var_map.insert(*id, obj_id);
    }

    pub fn has_var(&self, id: &u64) -> bool {
        self.var_map.contains_key(id)
    }

    pub fn has_const(&self, id: &u64) -> bool {
        self.const_map.contains_key(id)
    }

    pub fn insert_const(&mut self, id: &u64, obj_id: u64) {
        self.const_map.insert(*id, obj_id);
    }

    pub fn set_reverse_ip(&mut self, ip: (usize, usize)) {
        self.reverse_ip = ip;
    }

    pub fn new(name: &'a Cow<str>) -> Self {
        Self {
            var_map: TempHashMap::new(),
            const_map: TempHashMap::new(),
            reverse_ip: (0, 0),
            args: Vec::new(),
            cur_cls: None,
            ret_val: None,
            exp: None,
            name: name.clone(),
        }
    }
}

#[derive(Debug, Clone)]
enum SValue<'a> {
    Stack((u64, &'a String)),
    Attr((u64, &'a String)),
    Global(u64),
    #[allow(dead_code)]
    Object(FSRObject<'a>),
}

impl<'a> SValue<'a> {
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
            SValue::Object(obj) => FSRObject::obj_to_id(obj),
        }
    }

    #[allow(unused)]
    pub fn get_object(&self) -> Option<&FSRObject<'a>> {
        if let SValue::Object(obj) = self {
            return Some(obj);
        }

        None
    }
}

pub struct ThreadContext<'a> {
    exp: Vec<Rc<SValue<'a>>>,
    ip: (usize, usize),
    vm: &'a mut FSRVM<'a>,
    is_attr: bool,
    last_if_test: Vec<bool>,
    break_line: Vec<usize>,
    continue_line: Vec<usize>,
    for_iter_obj: Vec<u64>,
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

#[derive(Default, Debug)]
pub struct VecMap<'a> {
    inner: Vec<BytecodeFn<'a>>,
    #[cfg(feature = "perf")]
    start: HashMap<BytecodeOperator, Instant>,
    #[cfg(feature = "perf")]
    acumulate: HashMap<BytecodeOperator, Duration>,
    #[cfg(feature = "perf")]
    count: HashMap<BytecodeOperator, u64>,
}

impl<'a> VecMap<'a> {
    pub fn insert(&mut self, k: BytecodeOperator, f: BytecodeFn<'a>) {
        if k as usize != self.inner.len() {
            panic!()
        }

        self.inner.push(f);
    }

    #[inline(always)]
    pub fn get(&self, k: &BytecodeOperator) -> Option<&BytecodeFn<'a>> {
        self.inner.get(*k as usize)
    }

    #[cfg(feature = "perf")]
    pub fn start_time(&mut self, k: &BytecodeOperator) {
        self.start.insert(*k, Instant::now());
        if !self.count.contains_key(k) {
            self.count.insert(*k, 0);
        }

        if let Some(s) = self.count.get_mut(k) {
            s.add_assign(1);
        }
    }

    #[cfg(feature = "perf")]
    pub fn end_time(&mut self, k: &BytecodeOperator) {
        let v = self.start.get(k).unwrap();
        let now = Instant::now();
        let diff = now - *v;
        if !self.acumulate.contains_key(k) {
            self.acumulate.insert(*k, Duration::new(0, 0));
        }

        if let Some(s) = self.acumulate.get_mut(k) {
            s.add_assign(diff);
        }
    }
}

// pub struct ExpStack<'a> {
//     values      : Vec<Rc<SValue<'a>>>,
//     pos         : i64
// }

// impl<'a> ExpStack<'a> {
//     pub fn push(&mut self, value: Rc<SValue<'a>>) {
//         self.pos += 1;
//         if (self.pos as usize) < self.values.len() {
//             self.values[self.pos as usize] = value;
//         } else {
//             self.values.push(value);
//         }
//     }

//     pub fn pop(&mut self) -> Rc<SValue<'a>> {
//         let v = self.values[self.pos as usize].clone();
//         self.pos -= 1;
//         v
//     }
// }

#[derive(Default)]
pub struct FSRThreadRuntime<'a> {
    call_stack: Vec<CallState<'a>>,
    bytecode_map: VecMap<'a>,
    vm_ptr: Option<*mut FSRVM<'a>>,
}

impl<'a, 'b: 'a> FSRThreadRuntime<'a> {
    pub fn get_vm(&self) -> &FSRVM<'a> {
        unsafe { &*self.vm_ptr.unwrap() }
    }

    pub fn get_mut_vm(&mut self) -> &'a mut FSRVM<'a> {
        unsafe { &mut *self.vm_ptr.unwrap() }
    }

    pub fn new() -> FSRThreadRuntime<'a> {
        let mut map = VecMap::default();
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
        map.insert(BytecodeOperator::Break, Self::break_process);
        map.insert(BytecodeOperator::Continue, Self::continue_process);
        map.insert(BytecodeOperator::LoadForIter, Self::load_for_iter);
        map.insert(BytecodeOperator::ForBlockEnd, Self::for_block_end);
        map.insert(BytecodeOperator::PushForNext, Self::push_for_next);
        map.insert(BytecodeOperator::SpecialLoadFor, Self::special_load_for);
        map.insert(BytecodeOperator::AndJump, Self::process_logic_and);
        map.insert(BytecodeOperator::OrJump, Self::process_logic_or);

        Self {
            call_stack: vec![CallState::new(&Cow::Borrowed("base"))],
            bytecode_map: map,
            vm_ptr: None,
        }
    }

    pub fn set_vm(&mut self, vm: &mut FSRVM<'a>) {
        self.vm_ptr = Some(vm as *mut FSRVM<'a>);
    }

    fn load_integer_const(i: &i64, vm: &mut FSRVM) -> u64 {
        // let obj = FSRObject {
        //     obj_id: 0,
        //     value: FSRValue::Integer(*i),
        //     cls: "Integer",
        //     ref_count: AtomicU64::new(0),
        // };

        // vm.register_object(obj)
        
        vm.get_integer(*i)
    }

    fn load_string_const(s: String, _vm: &mut FSRVM<'a>) -> u64 {
        let obj = FSRObject {
            obj_id: 0,
            value: FSRValue::String(Cow::Owned(s)),
            cls: FSRGlobalObjId::StringCls as u64,
            ref_count: AtomicU64::new(0),
        };

        FSRVM::register_object(obj)
    }

    #[inline(always)]
    pub fn get_cur_mut_stack(&mut self) -> &mut CallState<'a> {
        let l = self.call_stack.len();
        return self.call_stack.get_mut(l - 1).unwrap();
    }

    fn get_cur_stack(&self) -> &CallState<'a> {
        let l = self.call_stack.len();
        return self.call_stack.get(l - 1).unwrap();
    }

    #[inline]
    fn compare(left: u64, right: u64, op: &str, thread: &mut Self) -> bool {
        let res;

        if op.eq(">") {
            res = FSRObject::invoke_offset_method(BinaryOffset::Greater, &vec![left, right], thread);
        } else if op.eq("<") {
            res = FSRObject::invoke_offset_method(BinaryOffset::Less, &vec![left, right], thread);
        } else if op.eq(">=") {
            res = FSRObject::invoke_offset_method(BinaryOffset::GreatEqual, &vec![left, right], thread);
        } else if op.eq("<=") {
            res = FSRObject::invoke_offset_method(BinaryOffset::LessEqual, &vec![left, right], thread);
        } else if op.eq("==") {
            res = FSRObject::invoke_offset_method(BinaryOffset::Equal, &vec![left, right], thread);
        } else if op.eq("!=") {
            res = FSRObject::invoke_offset_method(BinaryOffset::NotEqual, &vec![left, right], thread);
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
        for kv in v.var_map.iter() {
            let obj = FSRObject::id_to_obj(kv);
            if let Some(l) = &escape {
                if l.contains(&kv) {
                    obj.ref_dec();
                    continue;
                }
            }

            obj.ref_dec();
            if obj.count_ref() == 0 {
                // println!("Delete Object: {:#?}", obj);
            }
            //vm.check_delete(kv.1);
        }
    }

    fn assign_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        if let ArgType::Variable(var_id, _) = bytecode.get_arg() {
            let obj_id = match context.exp.pop() {
                Some(s) => s,
                None => {
                    return Err(FSRError::new("", FSRErrCode::EmptyExpStack));
                }
            };
            let obj_id = obj_id.get_global_id(self);
            if !FSRObject::is_sp_object(obj_id) {
                let to_assign_obj = FSRObject::id_to_mut_obj(obj_id);
                to_assign_obj.ref_add();
                let state = self.get_cur_mut_stack();
                state.insert_var(var_id, obj_id);
            } else {
                let state = self.get_cur_mut_stack();
                state.insert_var(var_id, obj_id);
            }

            return Ok(false);
        }

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
            if !FSRObject::is_sp_object(to_assign_obj_id) {
                let father = obj_id.get_global_id(self);
                let father = FSRObject::id_to_mut_obj(father);
                if let SValue::Attr((_, attr_name)) = *assign_id {
                    let obj = FSRObject::id_to_mut_obj(to_assign_obj_id);
                    obj.ref_add();
                    father.set_attr(attr_name, to_assign_obj_id);
                }
                context.is_attr = false;
            } else {
                let father = obj_id.get_global_id(self);
                let father = FSRObject::id_to_mut_obj(father);
                if let SValue::Attr((_, attr_name)) = *assign_id {
                    father.set_attr(attr_name, to_assign_obj_id);
                }
                context.is_attr = false;
            }
        } else {
            let obj_id = obj_id.get_global_id(self);
            if !FSRObject::is_sp_object(obj_id) {
                let to_assign_obj = FSRObject::id_to_mut_obj(obj_id);
                to_assign_obj.ref_add();
                let state = self.get_cur_mut_stack();
                if let SValue::Stack((var_id, _)) = *assign_id {
                    state.insert_var(&var_id, obj_id);
                }
            } else {
                let state = self.get_cur_mut_stack();
                if let SValue::Stack((var_id, _)) = *assign_id {
                    state.insert_var(&var_id, obj_id);
                }
            }
        }
        

        

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
        let res = FSRObject::invoke_offset_method(BinaryOffset::Add, &vec![v1, v2], self)?;
        match res {
            FSRRetValue::Value(object) => {
                let res_id = FSRVM::register_object(object);
                context.exp.push(Rc::new(SValue::Global(res_id)));
            }
            FSRRetValue::GlobalId(res_id) => {
                context.exp.push(Rc::new(SValue::Global(res_id)));
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
        let res = FSRObject::invoke_offset_method(BinaryOffset::Mul, &vec![v1, v2], self)?;
        match res {
            FSRRetValue::Value(object) => {
                let res_id = FSRVM::register_object(object);
                context.exp.push(Rc::new(SValue::Global(res_id)));
            }
            FSRRetValue::GlobalId(res_id) => {
                context.exp.push(Rc::new(SValue::Global(res_id)));
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
        let attr_id = match *context.exp.pop().unwrap() {
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
            context.exp.push(Rc::new(SValue::Global(dot_father)));
            context.exp.push(Rc::new(SValue::Attr((id, name))));
        } else {
            context.exp.push(Rc::new(SValue::Global(dot_father)));
            context.exp.push(Rc::new(SValue::Attr((attr_id.0, name))));
        }

        context.is_attr = true;
        Ok(false)
    }

    #[inline]
    fn call_process_set_args(
        args_num: usize,
        thread: &Self,
        exp: &mut Vec<Rc<SValue<'a>>>,
        args: &mut Vec<u64>,
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
        exp: &mut Vec<Rc<SValue<'a>>>,
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
        let fn_id = match *context.exp.pop().unwrap() {
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
                self_obj.set_cls(*context.vm.get_global_obj_by_name(fn_obj.get_fsr_class_name()).unwrap());
                self_obj.set_value(FSRValue::ClassInst(FSRClassInst::new(
                    fn_obj.get_fsr_class_name(),
                )));
                //println!("{:#?}", self_obj);
                let self_id = FSRVM::register_object(self_obj);

                // set self as fisrt args and call __new__ method to initialize object
                args.push(self_id);
                context.is_attr = true;
                Self::call_process_set_args(n, self, &mut context.exp, &mut args);
                let state = self.get_cur_mut_stack();
                state.set_reverse_ip(context.ip);
                state.exp = Some(context.exp.clone());
                self.call_stack
                    .push(CallState::new(&Cow::Borrowed("__new__")));
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
                    self.call_stack.push(CallState::new(&Cow::Borrowed("tmp")));
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
                    let v = fn_obj.call(&args, self).unwrap();

                    if let FSRRetValue::Value(v) = v {
                        let id = FSRVM::register_object(v);
                        context.exp.push(Rc::new(SValue::Global(id)));
                    } else if let FSRRetValue::GlobalId(id) = v {
                        context.exp.push(Rc::new(SValue::Global(id)));
                    }
                }
            } else {
                self.save_ip_to_callstate(n, &mut context.exp, &mut args, &mut context.ip);
                self.call_stack.push(CallState::new(&Cow::Borrowed("tmp2")));
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
                    let v = fn_obj.call(&args, self).unwrap();

                    if let FSRRetValue::Value(v) = v {
                        let id = FSRVM::register_object(v);
                        context.exp.push(Rc::new(SValue::Global(id)));
                        let vm = self.get_mut_vm();
                        let mut esp = HashSet::new();
                        esp.insert(id);
                        self.pop_stack(vm, Some(esp));
                    } else if let FSRRetValue::GlobalId(id) = v {
                        context.exp.push(Rc::new(SValue::Global(id)));
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
        let test_val = match *context.exp.pop().unwrap() {
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
        let test_val = match *context.exp.pop().unwrap() {
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

    fn break_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let break_line = context.break_line.pop().unwrap();
        context.continue_line.pop();
        context.ip = (break_line, 0);
        Ok(true)
    }

    fn continue_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let l = context.continue_line.len();
        let continue_line = context.continue_line[l - 1];
        context.ip = (continue_line, 0);
        Ok(true)
    }

    fn load_for_iter(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let iter_obj = context.exp.pop().unwrap().get_global_id(self);
        if let ArgType::ForLine(n) = bytecode.get_arg() {
            context.break_line.push(context.ip.0 + *n as usize);
            context.continue_line.push(context.ip.0 + 1);
        }
        context.for_iter_obj.push(iter_obj);
        Ok(false)
    }

    fn push_for_next(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let id = context.exp.last().unwrap().get_global_id(self);
        if id == 0 {
            let break_line = context.break_line.pop().unwrap();
            context.continue_line.pop();
            context.ip = (break_line, 0);
            return Ok(true);
        }
        Ok(false)
    }

    fn while_test_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let state = self.get_cur_mut_stack();
        let test_val = match *context.exp.pop().unwrap() {
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
                context.continue_line.pop();
                return Ok(true);
            }
        }
        if let ArgType::WhileTest(n) = bytecode.get_arg() {
            context.break_line.push(context.ip.0 + *n as usize + 1);
            context.continue_line.push(context.ip.0);
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
        let name = match *context.exp.pop().unwrap() {
            SValue::Stack(id) => id,
            SValue::Attr(_) => panic!(),
            SValue::Global(_) => panic!(),
            SValue::Object(_) => todo!(),
        };

        if let ArgType::DefineFnArgs(n, arg_len) = bytecode.get_arg() {
            let mut args = vec![];
            for _ in 0..*arg_len {
                let v = match *context.exp.pop().unwrap() {
                    SValue::Stack(id) => id,
                    SValue::Attr(_) => panic!(),
                    SValue::Global(_) => panic!(),
                    SValue::Object(_) => todo!(),
                };
                args.push(v.1.to_string());
            }
            let fn_obj = FSRFn::from_fsr_fn("main", (context.ip.0 + 1, 0), args, bc);
            let fn_id = FSRVM::register_object(fn_obj);
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
                context.exp.push(Rc::new(SValue::Global(context.vm.get_true_id())))
            } else {
                context.exp.push(Rc::new(SValue::Global(context.vm.get_false_id())))
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

    fn for_block_end(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        if let ArgType::ForEnd(n) = bytecode.get_arg() {
            context.ip = (context.ip.0 - *n as usize, 0);
            return Ok(true);
        }

        Ok(false)
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
            let id = FSRVM::register_object(list);
            context.exp.push(Rc::new(SValue::Global(id)));
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
        let id = match *context.exp.pop().unwrap() {
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
        cls_obj.set_cls(FSRGlobalObjId::ClassCls as u64);
        let obj = state.cur_cls.take().unwrap();
        let name = obj.get_name().to_string();
        cls_obj.set_value(FSRValue::Class(obj));
        let obj_id = FSRVM::register_object(cls_obj);
        context.vm.register_global_object(&name, obj_id);
        Ok(false)
    }

    fn special_load_for(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        _: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        context
            .exp
            .push(Rc::new(SValue::Global(*context.for_iter_obj.last().unwrap())));
        Ok(false)
    }

    fn process_logic_and(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bc: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let first = context.exp.pop().unwrap().get_global_id(self);
        if first == 0 || first == 2 {
            if let ArgType::AddOffset(offset) = bc.get_arg() {
                context.ip.1 += *offset;
                context.exp.push(Rc::new(SValue::Global(2)));
            }
        }

        Ok(false)
    }

    fn process_logic_or(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bc: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let first = context.exp.pop().unwrap().get_global_id(self);
        if first != 0 && first != 2 {
            if let ArgType::AddOffset(offset) = bc.get_arg() {
                context.ip.1 += *offset;
                context.exp.push(Rc::new(SValue::Global(1)));
            }
        }

        Ok(false)
    }

    #[inline(always)]
    fn process(
        &mut self,
        context: &mut ThreadContext<'a>,
        bytecode: &'a BytecodeArg,
        bc: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let op = bytecode.get_operator();
        #[cfg(feature = "perf")]
        self.bytecode_map.start_time(op);
        let bc_op = self.bytecode_map.get(op).unwrap();
        let v = bc_op(self, context, bytecode, bc)?;
        #[cfg(feature = "perf")]
        self.bytecode_map.end_time(op);
        if v {
            return Ok(v);
        }

        Ok(false)
    }

    #[inline(always)]
    fn load_var(
        exp_stack: &mut Vec<Rc<SValue<'a>>>,
        arg: &'a BytecodeArg,
        vm: &mut FSRVM<'a>,
        s: &mut CallState,
    ) {
        //*is_attr = false;
        if let ArgType::Variable(id, name) = arg.get_arg() {
            exp_stack.push(Rc::new(SValue::Stack((*id, name))));
        } else if let ArgType::ConstInteger(_, i) = arg.get_arg() {
            let int_const = Self::load_integer_const(i, vm);
            exp_stack.push(Rc::new(SValue::Global(int_const)));
        } else if let ArgType::ConstString(id, i) = arg.get_arg() {
            let string_const = Self::load_string_const(i.clone(), vm);
            s.insert_const(id, string_const);
            exp_stack.push(Rc::new(SValue::Global(string_const)));
        } else if let ArgType::Attr(id, name) = arg.get_arg() {
            exp_stack.push(Rc::new(SValue::Attr((*id, name))));
        }
    }

    #[inline(always)]
    fn set_exp_stack_ret(&mut self, exp_stack: &mut Vec<Rc<SValue<'a>>>) {
        let stack = self.get_cur_mut_stack();
        if stack.exp.is_some() {
            *exp_stack = stack.exp.take().unwrap();
            stack.exp = None;
        }

        if stack.ret_val.is_some() {
            exp_stack.push(Rc::new(SValue::Global(stack.ret_val.unwrap())));
            stack.ret_val = None;
        }
    }

    #[inline(always)]
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
            //println!("{:?}", context.exp);
            // println!("{}",t);
            context.ip.1 += 1;

            self.set_exp_stack_ret(&mut context.exp);

            if arg.get_operator() == &BytecodeOperator::Load {
                #[cfg(feature = "perf")]
                self.bytecode_map.start_time(&BytecodeOperator::Load);
                let state = self.get_cur_mut_stack();
                Self::load_var(&mut context.exp, arg, context.vm, state);
                #[cfg(feature = "perf")]
                self.bytecode_map.end_time(&BytecodeOperator::Load);
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

    pub fn start(
        &'a mut self,
        module: &'a FSRModule<'a>,
        vm: &'a mut FSRVM<'a>,
    ) -> Result<(), FSRError> {
        self.set_vm(vm);
        let mut context = ThreadContext {
            exp: Vec::with_capacity(10),
            ip: (0, 0),
            vm,
            is_attr: false,
            last_if_test: vec![],
            break_line: vec![],
            continue_line: vec![],
            for_iter_obj: vec![],
        };
        while let Some(expr) = module.get_bc(&context.ip) {
            self.run_expr(expr, &mut context, module.get_bytecode())?;
        }

        #[cfg(feature = "perf")]
        println!("{:#?}", self.bytecode_map.acumulate);

        #[cfg(feature = "perf")]
        println!("{:#?}", self.bytecode_map.count);
        Ok(())
    }

    pub fn call_fn(&mut self, fn_def: &'a FSRFnInner, args: &Vec<u64>) -> Result<u64, FSRError> {
        let mut context = ThreadContext {
            exp: Vec::with_capacity(10),
            ip: fn_def.get_ip(),
            is_attr: false,
            vm: self.get_mut_vm(),
            last_if_test: vec![],
            break_line: vec![],
            continue_line: vec![],
            for_iter_obj: vec![],
        };
        {
            //self.save_ip_to_callstate(args.len(), &mut context.exp, &mut args, &mut context.ip);
            self.call_stack.push(CallState::new(fn_def.get_name()));
            context.exp.clear();

            for arg in args.iter().rev() {
                self.get_cur_mut_stack().args.push(*arg);
            }
            //let offset = fn_obj.get_fsr_offset();
            let offset = fn_def.get_ip();
            context.ip = (offset.0, 0);
        }

        while let Some(expr) = fn_def.get_bytecode().get(&context.ip) {
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
            Some(s) => Ok(s),
            None => Ok(0),
        }
    }
}
