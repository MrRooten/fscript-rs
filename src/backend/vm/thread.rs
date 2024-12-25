#![allow(clippy::ptr_arg)]

use std::vec;
#[cfg(feature = "perf")]
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    ops::AddAssign,
    sync::atomic::AtomicU64,
    time::{Duration, Instant},
};

#[cfg(not(feature = "perf"))]
use std::borrow::Cow;

use crate::{
    backend::{
        compiler::bytecode::{ArgType, BinaryOffset, Bytecode, BytecodeArg, BytecodeOperator},
        types::{
            base::{FSRGlobalObjId, FSRObject, FSRRetValue, FSRValue, ObjId},
            class::FSRClass,
            class_inst::FSRClassInst,
            fn_def::{FSRFn, FSRFnInner},
            integer::FSRInteger,
            list::FSRList,
            module::FSRModule,
            string::FSRString,
        },
    },
    utils::error::{FSRErrCode, FSRError},
};

use super::runtime::FSRVM;

pub struct Bitmap {
    data: Vec<u8>,
    size: usize, // 位图的大小（位数）
}

impl Bitmap {
    /// 创建一个新的位图，大小为 size 位。
    pub fn new(size: usize) -> Self {
        let byte_size = (size + 7) / 8; // 每字节存储 8 位
        Bitmap {
            data: vec![0; byte_size],
            size,
        }
    }

    /// 设置位图中指定的位为 1。
    pub fn set(&mut self, index: usize) {
        if index >= self.size {
            return;
        }
        let byte_index = index / 8;
        let bit_index = index % 8;
        self.data[byte_index] |= 1 << bit_index;
    }

    /// 清除位图中指定的位（设置为 0）。
    pub fn clear(&mut self, index: usize) {
        if index >= self.size {
            return;
        }
        let byte_index = index / 8;
        let bit_index = index % 8;
        self.data[byte_index] &= !(1 << bit_index);
    }

    /// 检查位图中指定的位是否为 1。
    pub fn get(&self, index: usize) -> bool {
        if index >= self.size {
            return false;
        }
        let byte_index = index / 8;
        let bit_index = index % 8;
        (self.data[byte_index] & (1 << bit_index)) != 0
    }

    pub fn iter_ones(&self) -> Vec<usize> {
        let mut result = Vec::new();
        for (byte_index, &byte) in self.data.iter().enumerate() {
            for bit_index in 0..8 {
                let bit_position = byte_index * 8 + bit_index;
                if bit_position >= self.size {
                    break; // 超过位图大小时停止
                }
                if byte & (1 << bit_index) != 0 {
                    result.push(bit_position);
                }
            }
        }
        result
    }
}

#[derive(Debug)]
pub struct TempHashMap {
    vs: Vec<ObjId>,
}

pub struct TempIterator<'a> {
    vs: core::slice::Iter<'a, ObjId>,
}

#[allow(clippy::new_without_default)]
#[allow(unused)]
impl TempHashMap {
    #[inline(always)]
    pub fn get(&self, i: &u64) -> Option<&ObjId> {
        self.vs.get(*i as usize)
    }

    #[inline(always)]
    pub fn insert(&mut self, i: u64, v: ObjId) {
        if i as usize >= self.vs.len() {
            self.vs.resize(i as usize + 1, 0);
        }
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
            vs: vec![],
        }
    }

    pub fn iter(&self) -> TempIterator {
        TempIterator { vs: self.vs.iter() }
    }
}

impl<'a> Iterator for TempIterator<'a> {
    type Item = ObjId;

    fn next(&mut self) -> Option<Self::Item> {
        for s in self.vs.by_ref() {
            if s != &0 {
                return Some(*s);
            }
        }

        None
    }
}

pub struct CallFrame<'a> {
    var_map: TempHashMap,
    reverse_ip: (usize, usize),
    args: Vec<ObjId>,
    cur_cls: Option<Box<FSRClass<'a>>>,
    ret_val: Option<SValue<'a>>,
    exp: Option<Vec<SValue<'a>>>,
}

impl<'a> CallFrame<'a> {

    pub fn clear(&mut self) {
        self.var_map = TempHashMap::new();
        self.args.clear();
        self.cur_cls = None;
        self.ret_val = None;
        self.exp = None;
    }

    #[inline(always)]
    pub fn get_var(&self, id: &u64) -> Option<&ObjId> {
        if let Some(s) = self.var_map.get(id) {
            if s == &0 {
                return None;
            }

            return Some(s);
        }

        None
    }

    #[inline(always)]
    pub fn insert_var(&mut self, id: &u64, obj_id: ObjId, vm: &FSRVM<'a>, add_ref: bool) {
        if self.var_map.contains_key(id) {
            let to_be_dec = self.var_map.get(id).unwrap();
            let origin_obj = FSRObject::id_to_obj(*to_be_dec);
            origin_obj.ref_dec();
            if origin_obj.count_ref() == 0 {
                vm.allocator.free(*to_be_dec);
                //FSRObject::drop_object(*to_be_dec);
            }
        }
        if add_ref {
            FSRObject::id_to_obj(obj_id).ref_add();
        }
        
        self.var_map.insert(*id, obj_id);
    }

    #[inline(always)]
    pub fn has_var(&self, id: &u64) -> bool {
        self.var_map.contains_key(id)
    }



    pub fn set_reverse_ip(&mut self, ip: (usize, usize)) {
        self.reverse_ip = ip;
    }

    pub fn new(_name: &'a Cow<str>) -> Self {
        Self {
            var_map: TempHashMap::new(),
            reverse_ip: (0, 0),
            args: Vec::new(),
            cur_cls: None,
            ret_val: None,
            exp: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SValue<'a> {
    Stack((u64, &'a String)),
    Attr((ObjId, ObjId, &'a String)),
    Global(ObjId),
    #[allow(dead_code)]
    BoxObject(Box<FSRObject<'a>>),
    
}

impl<'a> SValue<'a> {
    #[inline(always)]
    pub fn is_object(&self) -> bool {
        if let SValue::BoxObject(_) = self {
            return true;
        }

        false
    }
    #[inline(always)]
    pub fn get_global_id(&self, thread: &FSRThreadRuntime) -> Result<ObjId, FSRError> {
        let state = thread.get_cur_stack();
        let vm = thread.get_vm();
        Ok(match self {
            SValue::Stack(s) => {
                if let Some(id) = state.get_var(&s.0) {
                    *id
                } else {
                    match vm.get_global_obj_by_name(s.1) {
                        Some(s) => *s,
                        None => {
                            return Err(FSRError::new(
                                format!(
                                    "not found variable in callstate and gloabl name `{}`",
                                    s.1
                                ),
                                FSRErrCode::NoSuchObject,
                            ));
                        }
                    }
                }
            }
            SValue::Global(id) => *id,
            SValue::Attr((_, id, _)) => *id,
            SValue::BoxObject(obj) => FSRObject::obj_to_id(obj),
        })
    }

    #[allow(unused)]
    pub fn get_object(&self) -> Option<&FSRObject<'a>> {
        if let SValue::BoxObject(obj) = self {
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
    last_if_test: Vec<bool>,
    break_line: Vec<usize>,
    continue_line: Vec<usize>,
    for_iter_obj: Vec<ObjId>,
    #[allow(unused)]
    module_stack: Vec<u64>,
    module: Option<ObjId>,
}

impl ThreadContext<'_> {
    #[inline(always)]
    pub fn false_last_if_test(&mut self) {
        let l = self.last_if_test.len() - 1;
        self.last_if_test[l] = false;
    }

    #[inline(always)]
    pub fn true_last_if_test(&mut self) {
        let l = self.last_if_test.len() - 1;
        self.last_if_test[l] = true;
    }

    #[inline(always)]
    pub fn peek_last_if_test(&self) -> bool {
        if self.last_if_test.is_empty() {
            return false;
        }

        self.last_if_test[self.last_if_test.len() - 1]
    }

    #[inline(always)]
    pub fn push_last_if_test(&mut self, test: bool) {
        self.last_if_test.push(test)
    }

    #[inline(always)]
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
    pub fn insert(&mut self, _k: BytecodeOperator, f: BytecodeFn<'a>) {
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
    call_stack: Vec<CallFrame<'a>>,
    stack_index: usize,

    #[allow(unused)]
    bytecode_map: VecMap<'a>,
    vm_ptr: Option<*mut FSRVM<'a>>,
}

impl<'a> FSRThreadRuntime<'a> {
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
        map.insert(BytecodeOperator::Empty, Self::empty_process);
        map.insert(BytecodeOperator::BinarySub, Self::binary_sub_process);

        Self {
            call_stack: vec![CallFrame::new(&Cow::Borrowed("base"))],
            bytecode_map: map,
            vm_ptr: None,
            stack_index: 0,
        }
    }

    pub fn push_frame(&mut self) {
        self.stack_index += 1;

        if let Some(s) = self.call_stack.get_mut(self.stack_index) {
            s.clear();
        }
    }

    pub fn pop_frame(&mut self) -> Option<&mut CallFrame<'a>> {
        self.stack_index -= 1;
        self.call_stack.get_mut(self.stack_index + 1)
    }

    pub fn set_vm(&mut self, vm: &mut FSRVM<'a>) {
        self.vm_ptr = Some(vm as *mut FSRVM<'a>);
    }

    // fn load_string_const(s: String, _vm: &mut FSRVM<'a>) -> u64 {
    //     let obj = FSRObject {
    //         obj_id: 0,
    //         value: FSRValue::String(Cow::Owned(s)),
    //         cls: FSRGlobalObjId::StringCls as u64,
    //         ref_count: AtomicU64::new(0),
    //     };

    //     FSRVM::register_object(obj)
    // }

    #[inline(always)]
    pub fn get_cur_mut_stack(&mut self) -> &mut CallFrame<'a> {
        return self.call_stack.last_mut().unwrap();
    }

    #[inline(always)]
    fn get_cur_stack(&self) -> &CallFrame<'a> {
        return self.call_stack.last().unwrap();
    }



    #[inline(always)]
    fn compare(left: ObjId, right: ObjId, op: &str, thread: &mut Self) -> Result<bool, FSRError> {
        let res;

        if op.eq(">") {
            res = FSRObject::invoke_offset_method(
                BinaryOffset::Greater,
                &[left, right],
                thread,
                None,
            )?;
        } else if op.eq("<") {
            res = FSRObject::invoke_offset_method(
                BinaryOffset::Less,
                &[left, right],
                thread,
                None,
            )?;
        } else if op.eq(">=") {
            res = FSRObject::invoke_offset_method(
                BinaryOffset::GreatEqual,
                &[left, right],
                thread,
                None,
            )?;
        } else if op.eq("<=") {
            res = FSRObject::invoke_offset_method(
                BinaryOffset::LessEqual,
                &[left, right],
                thread,
                None,
            )?;
        } else if op.eq("==") {
            res = FSRObject::invoke_offset_method(
                BinaryOffset::Equal,
                &[left, right],
                thread,
                None,
            )?;
        } else if op.eq("!=") {
            res = FSRObject::invoke_offset_method(
                BinaryOffset::NotEqual,
                &[left, right],
                thread,
                None,
            )?;
        } else {
            return Err(FSRError::new(
                format!("not support op: `{}`", op),
                FSRErrCode::NotSupportOperator,
            ));
        }
        if let FSRRetValue::GlobalId(id) = &res {
            return Ok(id == &1);
        }
        Err(FSRError::new("not a object", FSRErrCode::NotValidArgs))
    }

    fn pop_stack(&mut self, vm: &mut FSRVM, escape: &[ObjId]) {
        let v = self.call_stack.pop().unwrap();
        for kv in v.var_map.iter() {
            let obj = FSRObject::id_to_obj(kv);

            obj.ref_dec();
            if escape.contains(&kv) {
                
                continue;
            }

            
            if obj.count_ref() == 0 {
                vm.allocator.free(kv);
            }
            //vm.check_delete(kv.1);
        }
    }

    #[inline(always)]
    fn assign_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        if let ArgType::Variable(var_id, _) = bytecode.get_arg() {
            let svalue = match context.exp.pop() {
                Some(s) => s,
                None => {
                    return Err(FSRError::new("", FSRErrCode::EmptyExpStack));
                }
            };

            if let SValue::BoxObject(obj) = svalue {
                let state = self.get_cur_mut_stack();
                let id = FSRVM::leak_object(obj);

                // println!("{:#?}", obj);
                state.insert_var(var_id, id, context.vm, true);
                return Ok(false);
            }
            let obj_id = svalue.get_global_id(self)?;
            let state = self.get_cur_mut_stack();
            if !FSRObject::is_sp_object(obj_id) {
                state.insert_var(var_id, obj_id, context.vm, true);
            } else {
                state.insert_var(var_id, obj_id, context.vm, true);
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

        let to_assign_obj_id = context.exp.pop().unwrap().get_global_id(self).unwrap();

        match assign_id {
            SValue::Stack((var_id, _)) => {
                if !FSRObject::is_sp_object(to_assign_obj_id) {
                    let state = self.get_cur_mut_stack();
                    state.insert_var(&var_id, to_assign_obj_id, context.vm, true);
                    
                } else {
                    let state = self.get_cur_mut_stack();
                    state.insert_var(&var_id, to_assign_obj_id, context.vm, true);
                    
                }
            }
            SValue::Attr(attr) => {
                let father_obj = FSRObject::id_to_mut_obj(attr.0);
                father_obj.set_attr(attr.2, to_assign_obj_id);
            }
            SValue::Global(_) => todo!(),
            SValue::BoxObject(_) => todo!(),
        }

        //To Assign obj or Dot Father object

        // let obj_id = match context.exp.pop() {
        //     Some(s) => s,
        //     None => {
        //         return Err(FSRError::new("", FSRErrCode::EmptyExpStack));
        //     }
        // };
        // if context.is_attr {
        //     let to_assign_obj = match context.exp.pop() {
        //         Some(s) => s,
        //         None => {
        //             return Err(FSRError::new("", FSRErrCode::EmptyExpStack));
        //         }
        //     };

        //     let to_assign_obj_id = to_assign_obj.get_global_id(self)?;
        //     if !FSRObject::is_sp_object(to_assign_obj_id) {
        //         let father = obj_id.get_global_id(self)?;
        //         let father = FSRObject::id_to_mut_obj(father);
        //         if let SValue::Attr((_, _, attr_name)) = assign_id {
        //             if to_assign_obj.is_object() {
        //                 if let SValue::Object(obj) = to_assign_obj {
        //                     let id = FSRVM::leak_object(obj);
        //                     let tmp_to_assign_obj = FSRObject::id_to_obj(id);
        //                     tmp_to_assign_obj.ref_add();
        //                     father.set_attr(attr_name, id);
        //                 } else {
        //                     unimplemented!()
        //                 }
        //             } else {
        //                 father.set_attr(attr_name, to_assign_obj_id);
        //             }
        //         }
        //         context.is_attr = false;
        //     } else {
        //         let father = obj_id.get_global_id(self)?;
        //         let father = FSRObject::id_to_mut_obj(father);
        //         if let SValue::Attr((_, _, attr_name)) = assign_id {
        //             father.set_attr(attr_name, to_assign_obj_id);
        //         }
        //         context.is_attr = false;
        //     }
        //     context.exp.pop();
        // } else {
        //     let obj_id = obj_id.get_global_id(self)?;
        //     if !FSRObject::is_sp_object(obj_id) {
        //         let to_assign_obj = FSRObject::id_to_mut_obj(obj_id);
        //         to_assign_obj.ref_add();
        //         let state = self.get_cur_mut_stack();
        //         if let SValue::Stack((var_id, _)) = assign_id {
        //             state.insert_var(&var_id, obj_id);
        //         }
        //     } else {
        //         let state = self.get_cur_mut_stack();
        //         if let SValue::Stack((var_id, _)) = assign_id {
        //             state.insert_var(&var_id, obj_id);
        //         }
        //     }
        // }

        Ok(false)
    }

    #[inline(always)]
    fn binary_add_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let _state = self.get_cur_mut_stack();
        let v1 = match context.exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary add 1",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let v2 = match context.exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary add 2",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let v1_id = v1.get_global_id(self)?;
        let v2_id = v2.get_global_id(self)?;
        let res =
            FSRObject::invoke_offset_method(BinaryOffset::Add, &[v2_id, v1_id], self, None)?;

        match res {
            FSRRetValue::Value(object) => {
                let res_id = FSRVM::leak_object(object);
                context.exp.push(SValue::Global(res_id));
            }
            FSRRetValue::GlobalId(res_id) => {
                context.exp.push(SValue::Global(res_id));
            }
            FSRRetValue::GlobalIdTemp(res_id) => {
                context.exp.push(SValue::Global(res_id));
            }
        };

        Ok(false)
    }

    #[inline(always)]
    fn binary_sub_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let _state = self.get_cur_mut_stack();
        let v1 = match context.exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary sub 1",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };
        if let SValue::Attr(_) = &v1 {
            context.exp.pop();
            context.is_attr = false;
        }

        let v2 = match context.exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary sub 2",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        if let SValue::Attr(_) = &v2 {
            context.exp.pop();
            context.is_attr = false;
        }

        let v1_id = v1.get_global_id(self)?;
        let v2_id = v2.get_global_id(self)?;
        let res =
            FSRObject::invoke_offset_method(BinaryOffset::Sub, &[v2_id, v1_id], self, None)?;

        match res {
            FSRRetValue::Value(object) => {
                let res_id = FSRVM::leak_object(object);
                context.exp.push(SValue::Global(res_id));
            }
            FSRRetValue::GlobalId(res_id) => {
                context.exp.push(SValue::Global(res_id));
            }
            FSRRetValue::GlobalIdTemp(res_id) => {
                context.exp.push(SValue::Global(res_id));
            }
        };

        Ok(false)
    }

    #[inline(always)]
    fn binary_mul_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let _state = self.get_cur_mut_stack();
        let v1 = match context.exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary mul 1",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        if let SValue::Attr(_) = &v1 {
            context.exp.pop();
            context.is_attr = false;
        }

        let v2 = match context.exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary mul 2",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        if let SValue::Attr(_) = &v2 {
            context.exp.pop();
            context.is_attr = false;
        }

        let v1_id = v1.get_global_id(self)?;
        let v2_id = v2.get_global_id(self)?;

        let res =
            FSRObject::invoke_offset_method(BinaryOffset::Mul, &[v2_id, v1_id], self, None)?;
        match res {
            FSRRetValue::Value(object) => {
                let res_id = FSRVM::leak_object(object);
                context.exp.push(SValue::Global(res_id));
            }
            FSRRetValue::GlobalId(res_id) => {
                context.exp.push(SValue::Global(res_id));
            }
            FSRRetValue::GlobalIdTemp(res_id) => {
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
            SValue::BoxObject(_) => todo!(),
        };
        let dot_father = match context.exp.pop() {
            Some(s) => s.get_global_id(self)?,
            None => {
                return Err(FSRError::new(
                    "error in dot operator",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        // if context.is_attr {
        //     // pop last father
        //     context.is_attr = false;
        //     context.exp.pop();
        // }

        let dot_father_obj = FSRObject::id_to_obj(dot_father);
        let name = attr_id.2;
        let id = dot_father_obj.get_attr(name, context.vm);
        if let Some(id) = id {
            // let obj = FSRObject::id_to_obj(id);
            // println!("{:#?}", obj);
            //context.exp.push(SValue::Global(dot_father));
            context.exp.push(SValue::Attr((dot_father, id, name)));
        } else {
            //context.exp.push(SValue::Global(dot_father));
            context
                .exp
                .push(SValue::Attr((dot_father, attr_id.1, name)));
        }

        Ok(false)
    }

    #[inline]
    fn call_process_set_args(
        args_num: usize,
        thread: &Self,
        exp: &mut Vec<SValue<'a>>,
        args: &mut Vec<ObjId>,
    ) {
        let mut i = 0;
        while i < args_num {
            let arg = exp.pop().unwrap();
            let a_id = match arg {
                SValue::Stack(s) => *thread.get_cur_stack().get_var(&s.0).unwrap(),
                SValue::Global(g) => g,
                SValue::BoxObject(obj) => FSRVM::leak_object(obj),
                SValue::Attr(a) => a.1,
            };
            args.push(a_id);
            i += 1;
        }
    }

    #[inline]
    fn save_ip_to_callstate(
        &mut self,
        exp: &mut Vec<SValue<'a>>,
        ip: &mut (usize, usize),
    ) {
        //Self::call_process_set_args(args_num, self, exp, args);
        let state = self.get_cur_mut_stack();
        state.set_reverse_ip(*ip);
        state.exp = Some(exp.clone());
    }

    #[inline(always)]
    fn process_fsr_cls(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        fn_obj: &FSRObject,
        args: &mut Vec<usize>,
    ) -> Result<bool, FSRError> {
        //let mut args = vec![];
        // New a object if fn_obj is fsr_cls

        let mut self_obj = FSRObject::new();
        self_obj.set_cls(
            *context
                .vm
                .get_global_obj_by_name(fn_obj.get_fsr_class_name())
                .unwrap(),
        );
        self_obj.set_value(FSRValue::ClassInst(Box::new(FSRClassInst::new(
            fn_obj.get_fsr_class_name(),
        ))));
        //println!("{:#?}", self_obj);
        let self_id = FSRVM::register_object(self_obj);

        // set self as fisrt args and call __new__ method to initialize object
        //args.push(self_id);
        args.insert(0, self_id);
        context.is_attr = true;
        //Self::call_process_set_args(n, self, &mut context.exp, &mut args);
        let state = self.get_cur_mut_stack();
        state.set_reverse_ip(context.ip);

        state.exp = Some(context.exp.clone());
        self.call_stack
            .push(CallFrame::new(&Cow::Borrowed("__new__")));
        context.exp.clear();
        let self_obj = FSRObject::id_to_obj(self_id);
        let self_new = self_obj.get_cls_attr("__new__", context.vm);

        if let Some(id) = self_new {
            for arg in args.iter().rev() {
                //obj.ref_add();
                self.get_cur_mut_stack().args.push(*arg);
            }
            let new_obj = FSRObject::id_to_obj(id);
            let offset = new_obj.get_fsr_offset().1;
            context.ip = (offset.0, 0);
            Ok(true)
        } else {
            panic!("not existed method ")
        }
    }

    fn process_fn_is_attr(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        obj_id: ObjId,
        fn_obj: &'a FSRObject<'a>,
        args: &mut Vec<usize>,
    ) -> Result<bool, FSRError> {
        // let obj_id = context.exp.pop().unwrap().get_global_id(self)?;

        args.insert(0, obj_id);
        context.is_attr = false;
        //Self::call_process_set_args(n, self, &mut context.exp, &mut args);
        // println!("{:#?}", fn_obj);
        if fn_obj.is_fsr_function() {
            let state = self.get_cur_mut_stack();
            //Save callstate
            state.set_reverse_ip(context.ip);
            state.exp = Some(context.exp.clone());
            self.call_stack.push(CallFrame::new(&Cow::Borrowed("tmp")));
            //Clear exp stack
            context.exp.clear();

            for arg in args.iter().rev() {
                self.get_cur_mut_stack().args.push(*arg);
            }
            let offset = fn_obj.get_fsr_offset().1;
            context.ip = (offset.0, 0);
            return Ok(true);
        } else {
            let v = fn_obj.call(args, self, context.module).unwrap();

            if let FSRRetValue::Value(v) = v {
                let id = FSRVM::leak_object(v);
                context.exp.push(SValue::Global(id));
            } else if let FSRRetValue::GlobalId(id) = v {
                context.exp.push(SValue::Global(id));
            }
        }
        Ok(false)
    }

    fn call_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let mut args = vec![];
        if let ArgType::CallArgsNumber(n) = bytecode.get_arg() {
            let args_num = *n;
            Self::call_process_set_args(args_num, self, &mut context.exp, &mut args);
            args.reverse();
            // for i in &args {
            //     println!("{:#?}", FSRObject::id_to_obj(*i));
            // }
        }

        // if context.is_attr {
        //     context.is_attr = false;
        //     context.exp.pop();
        // }

        //let ptr = vm as *mut FSRVM;
        let mut object_id: Option<ObjId> = None;
        let fn_id = match context.exp.pop().unwrap() {
            SValue::Stack(s) => {
                let state = self.get_cur_mut_stack();
                if let Some(id) = state.get_var(&s.0) {
                    *id
                } else {
                    match context.vm.get_global_obj_by_name(s.1) {
                        Some(s) => *s,
                        None => {
                            return Err(FSRError::new(
                                format!(
                                    "not found variable in callstate and gloabl name `{}`",
                                    s.1
                                ),
                                FSRErrCode::NoSuchObject,
                            ));
                        }
                    }
                }
            }
            SValue::Global(id) => id,
            SValue::Attr((o, id, _)) => {
                object_id = Some(o);
                id
            }
            SValue::BoxObject(_) => todo!(),
        };

        let fn_obj = FSRObject::id_to_obj(fn_id);

        if fn_obj.is_fsr_cls() {
            let v = Self::process_fsr_cls(self, context, fn_obj, &mut args)?;
            if v {
                return Ok(v);
            }
        } else if object_id.is_some() {
            let v = Self::process_fn_is_attr(self, context, object_id.unwrap(), fn_obj, &mut args)?;
            context.is_attr = false;
            if v {
                return Ok(v);
            }
        } else {
            self.save_ip_to_callstate(&mut context.exp, &mut context.ip);
            self.call_stack.push(CallFrame::new(&Cow::Borrowed("tmp2")));
            context.exp.clear();

            if fn_obj.is_fsr_function() {
                for arg in args.iter().rev() {
                    self.get_cur_mut_stack().args.push(*arg);
                }
                //let offset = fn_obj.get_fsr_offset();
                let offset = fn_obj.get_fsr_offset().1;
                context.ip = (offset.0, 0);
                return Ok(true);
            } else {
                // for arg in args.iter().rev() {
                //     let obj = FSRObject::id_to_obj(*arg);
                //     obj.ref_add();
                // }
                // not going to assign_args
                let v = fn_obj.call(&args, self, context.module).unwrap();

                if let FSRRetValue::Value(v) = v {
                    let id = FSRVM::leak_object(v);
                    context.exp.push(SValue::Global(id));
                    let vm = self.get_mut_vm();
                    self.pop_stack(vm, &[id]);
                } else if let FSRRetValue::GlobalId(id) = v {
                    context.exp.push(SValue::Global(id));
                    let vm = self.get_mut_vm();
                    self.pop_stack(vm, &[id]);
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
        let test_val = match &context.exp.pop().unwrap() {
            SValue::Stack(s) => {
                if let Some(id) = state.get_var(&s.0) {
                    *id
                } else {
                    unimplemented!()
                }
            }
            SValue::Global(id) => *id,
            SValue::BoxObject(obj) => obj.is_true_id(),
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

    #[inline(always)]
    fn if_end(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        context.pop_last_if_test();
        Ok(false)
    }

    #[inline(always)]
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
                    return Err(FSRError::new(
                        format!("Not found variable `{}`", s.1),
                        FSRErrCode::NoSuchObject,
                    ));
                }
            }
            SValue::Global(id) => id,
            SValue::BoxObject(obj) => obj.is_true_id(),
            _ => {
                return Err(FSRError::new(
                    "Not a valid test object",
                    FSRErrCode::NotValidArgs,
                ))
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

    #[inline(always)]
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

    #[inline(always)]
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

    #[inline(always)]
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

    #[inline(always)]
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

    #[inline(always)]
    fn load_for_iter(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let iter_obj = context.exp.pop().unwrap();
        let iter_id = if let SValue::BoxObject(obj) = iter_obj {
            FSRVM::leak_object(obj)
        } else {
            iter_obj.get_global_id(self)?
        };
        // let v = FSRObject::id_to_obj(iter_id);
        // println!("{:#?}", v);
        if let ArgType::ForLine(n) = bytecode.get_arg() {
            context.break_line.push(context.ip.0 + *n as usize);
            context.continue_line.push(context.ip.0 + 1);
        }
        context.for_iter_obj.push(iter_id);
        Ok(false)
    }

    #[inline(always)]
    fn push_for_next(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let id = context.exp.last().unwrap().get_global_id(self)?;
        if id == 0 {
            let break_line = context.break_line.pop().unwrap();
            context.continue_line.pop();
            context.ip = (break_line, 0);
            return Ok(true);
        }
        Ok(false)
    }

    #[inline(always)]
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
                    return Err(FSRError::new(
                        format!("Not found variable `{}`", s.1),
                        FSRErrCode::NoSuchObject,
                    ));
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
            // Avoid repeat add break ip and continue ip
            if let Some(s) = context.break_line.last() {
                if context.ip.0 + *n as usize + 1 != *s {
                    context.break_line.push(context.ip.0 + *n as usize + 1);
                }
            } else {
                context.break_line.push(context.ip.0 + *n as usize + 1);
            }

            if let Some(s) = context.continue_line.last() {
                if context.ip.0 != *s {
                    context.continue_line.push(context.ip.0);
                }
            } else {
                context.continue_line.push(context.ip.0);
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
            SValue::BoxObject(_) => todo!(),
        };

        if let ArgType::DefineFnArgs(n, arg_len) = bytecode.get_arg() {
            let mut args = vec![];
            for _ in 0..*arg_len {
                let v = match context.exp.pop().unwrap() {
                    SValue::Stack(id) => id,
                    SValue::Attr(_) => panic!(),
                    SValue::Global(_) => panic!(),
                    SValue::BoxObject(_) => todo!(),
                };
                args.push(v.1.to_string());
            }
            let fn_obj = FSRFn::from_fsr_fn("main", (context.ip.0 + 1, 0), args, bc, context.module.unwrap());
            fn_obj.ref_add();
            let fn_id = FSRVM::register_object(fn_obj);
            if let Some(cur_cls) = &mut state.cur_cls {
                cur_cls.insert_attr_id(name.1, fn_id);
                context.ip = (context.ip.0 + *n as usize + 2, 0);
                return Ok(true);
            }

            state.insert_var(&name.0, fn_id, context.vm, true);
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
        self.pop_stack(vm, &[]);
        let cur = self.get_cur_mut_stack();
        context.ip = (cur.reverse_ip.0, cur.reverse_ip.1 + 1);
        Ok(true)
    }

    #[inline(always)]
    fn compare_test(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        //let state = self.get_cur_stack();
        if let ArgType::Compare(op) = bytecode.get_arg() {
            let right = context.exp.last().unwrap().get_global_id(self)?;
            let left = context
                .exp
                .get(context.exp.len() - 2)
                .unwrap()
                .get_global_id(self)?;
            let v = Self::compare(left, right, op, self)?;

            context.exp.pop();

            context.exp.pop();

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
        let v = context.exp.pop().unwrap().get_global_id(self)?;
        // let mut esp = HashSet::new();
        // esp.insert(v);
        let vm = self.get_mut_vm();
        self.pop_stack(vm, &[v]);
        let cur = self.get_cur_mut_stack();
        //exp.push(SValue::GlobalId(v));
        cur.ret_val = Some(SValue::Global(v));
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
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let state = self.get_cur_mut_stack();
        let v = state.args.pop().unwrap();
        if let ArgType::Variable(s_id, _) = bytecode.get_arg() {
            state.insert_var(s_id, v, context.vm, true);
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
                let v = context.exp.pop().unwrap();
                let v_id = if let SValue::BoxObject(obj) = v {
                    FSRVM::leak_object(obj)
                } else {
                    v.get_global_id(self)?
                };

                let obj = FSRObject::id_to_obj(v_id);
                obj.ref_add();
                list.push(v_id);
            }

            let list = FSRList::new_object(list);
            let id = FSRVM::register_object(list);
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
            SValue::BoxObject(_) => todo!(),
        };

        let new_cls = FSRClass::new(id.1);
        state.cur_cls = Some(Box::new(new_cls));

        Ok(false)
    }

    fn process_import(
        self: &mut FSRThreadRuntime<'a>,
        bc: &BytecodeArg,
    ) -> Result<bool, FSRError> {
        if let ArgType::Import(v) = bc.get_arg() {
            let code = r#"
            fn out() {
                println('abc')
            }
            "#;

            let module = FSRModule::from_code("test", code)?;
            let obj_id = self.load(Box::new(module))?;

            return Ok(true)
        }
        unimplemented!()
    }

    fn end_class_def(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        _: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let state = self.get_cur_mut_stack();
        let mut cls_obj = FSRObject::new();
        cls_obj.set_cls(FSRGlobalObjId::ClassCls as ObjId);
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
            .push(SValue::Global(*context.for_iter_obj.last().unwrap()));
        Ok(false)
    }

    fn process_logic_and(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bc: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let first = context.exp.pop().unwrap().get_global_id(self)?;
        if first == 0 || first == 2 {
            if let ArgType::AddOffset(offset) = bc.get_arg() {
                context.ip.1 += *offset;
                context.exp.push(SValue::Global(2));
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
        let first = context.exp.pop().unwrap().get_global_id(self)?;
        if first != 0 && first != 2 {
            if let ArgType::AddOffset(offset) = bc.get_arg() {
                context.ip.1 += *offset;
                context.exp.push(SValue::Global(1));
            }
        }

        Ok(false)
    }

    

    fn empty_process(
        self: &mut FSRThreadRuntime<'a>,
        _context: &mut ThreadContext<'a>,
        _bc: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
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

        // let bc_op = self.bytecode_map.get(op).unwrap();
        // let v = bc_op(self, context, bytecode, bc)?;
        let v = match op {
            BytecodeOperator::Assign => Self::assign_process(self, context, bytecode, bc),
            BytecodeOperator::BinaryAdd => Self::binary_add_process(self, context, bytecode, bc),
            BytecodeOperator::BinaryDot => Self::binary_dot_process(self, context, bytecode, bc),
            BytecodeOperator::BinaryMul => Self::binary_mul_process(self, context, bytecode, bc),
            BytecodeOperator::Call => Self::call_process(self, context, bytecode, bc),
            BytecodeOperator::IfTest => Self::if_test_process(self, context, bytecode, bc),
            BytecodeOperator::WhileTest => Self::while_test_process(self, context, bytecode, bc),
            BytecodeOperator::DefineFn => Self::define_fn(self, context, bytecode, bc),
            BytecodeOperator::EndDefineFn => Self::end_define_fn(self, context, bytecode, bc),
            BytecodeOperator::CompareTest => Self::compare_test(self, context, bytecode, bc),
            BytecodeOperator::ReturnValue => Self::ret_value(self, context, bytecode, bc),
            BytecodeOperator::WhileBlockEnd => Self::while_block_end(self, context, bytecode, bc),
            BytecodeOperator::AssignArgs => Self::assign_args(self, context, bytecode, bc),
            BytecodeOperator::ClassDef => Self::class_def(self, context, bytecode, bc),
            BytecodeOperator::EndDefineClass => Self::end_class_def(self, context, bytecode, bc),
            BytecodeOperator::LoadList => Self::load_list(self, context, bytecode, bc),
            BytecodeOperator::Else => Self::else_process(self, context, bytecode, bc),
            BytecodeOperator::ElseIf => Self::else_if_match(self, context, bytecode, bc),
            BytecodeOperator::ElseIfTest => Self::else_if_test_process(self, context, bytecode, bc),
            BytecodeOperator::IfBlockEnd => Self::if_end(self, context, bytecode, bc),
            BytecodeOperator::Break => Self::break_process(self, context, bytecode, bc),
            BytecodeOperator::Continue => Self::continue_process(self, context, bytecode, bc),
            BytecodeOperator::LoadForIter => Self::load_for_iter(self, context, bytecode, bc),
            BytecodeOperator::PushForNext => Self::push_for_next(self, context, bytecode, bc),
            BytecodeOperator::ForBlockEnd => Self::for_block_end(self, context, bytecode, bc),
            BytecodeOperator::SpecialLoadFor => Self::special_load_for(self, context, bytecode, bc),
            BytecodeOperator::AndJump => Self::process_logic_and(self, context, bytecode, bc),
            BytecodeOperator::OrJump => Self::process_logic_or(self, context, bytecode, bc),
            BytecodeOperator::Empty => Self::empty_process(self, context, bytecode, bc),
            BytecodeOperator::BinaryRShift => unimplemented!(),
            BytecodeOperator::BinaryLShift => unimplemented!(),
            BytecodeOperator::StoreFast => unimplemented!(),
            BytecodeOperator::Load => unimplemented!(),
            BytecodeOperator::BinarySub => Self::binary_sub_process(self, context, bytecode, bc),
            BytecodeOperator::Import => Self::process_import(self, bytecode),
        }?;

        #[cfg(feature = "perf")]
        self.bytecode_map.end_time(op);
        if v {
            return Ok(v);
        }

        Ok(false)
    }

    #[inline(always)]
    fn load_var(
        exp_stack: &mut Vec<SValue<'a>>,
        arg: &'a BytecodeArg,
        vm: &mut FSRVM<'a>,
        _s: &mut CallFrame,
        module: Option<ObjId>,
    ) {
        //*is_attr = false;
        
        if let ArgType::Variable(id, name) = arg.get_arg() {
            exp_stack.push(SValue::Stack((*id, name)));
        } else if let ArgType::ConstInteger(c_id, i) = arg.get_arg() {
            //let int_const = Self::load_integer_const(i, vm);
            if let Some(m) = module {
                let obj = FSRObject::id_to_obj(m);
                let m = obj.as_module();
                if let Some(id) = m.get_bytecode().const_table.table.get(*c_id as usize) {
                    if id != &0 {
                        exp_stack.push(SValue::Global(*id));
                        return;
                    }
                }
            }
            if !vm.has_int_const(i) {
                let obj = FSRInteger::new_inst(*i);
                obj.set_not_delete();
                vm.insert_int_const(i, obj);
            }

            let id = vm.get_int_const(i).unwrap();

            exp_stack.push(SValue::Global(id));
        } else if let ArgType::ConstString(c_id, i) = arg.get_arg() {
            // let string_const = Self::load_string_const(i.clone(), vm);
            // s.insert_const(id, string_const);
            if let Some(m) = module {
                let m = FSRObject::id_to_obj(m).as_module();
                if let Some(id) = m.get_bytecode().const_table.table.get(*c_id as usize) {
                    if id != &0 {
                        exp_stack.push(SValue::Global(*id));
                        return;
                    }
                }
            }
            if !vm.has_str_const(i.as_str()) {
                let obj = FSRString::new_inst(Cow::Owned(i.clone()));
                obj.set_not_delete();
                vm.insert_str_const(i.as_str(), obj);
            }

            let id = vm.get_str_const(i.as_str()).unwrap();
            exp_stack.push(SValue::Global(id));
        } else if let ArgType::Attr(_, name) = arg.get_arg() {
            exp_stack.push(SValue::Attr((0, 0, name)));
        }
        // } else {
        //     panic!("not recongize load var: {:#?}", arg)
        // }
    }

    #[inline(always)]
    fn set_exp_stack_ret(&mut self, exp_stack: &mut Vec<SValue<'a>>) {
        let stack = self.get_cur_mut_stack();
        if let Some(s) = stack.exp.take() {
            *exp_stack = s;
            stack.exp = None;
        }

        if let Some(s) = stack.ret_val.take() {
            exp_stack.push(s);
            stack.ret_val = None;
        }

        // if stack.ret_val.is_some() {
        //     let v = stack.ret_val.take();
        //     exp_stack.push(v.unwrap());
        //     stack.ret_val = None;
        // }
    }

    #[inline(always)]
    fn run_expr(
        &mut self,
        expr: &'a Vec<BytecodeArg>,
        context: &mut ThreadContext<'a>,
        bc: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let mut v;

        while let Some(arg) = expr.get(context.ip.1) {
            context.ip.1 += 1;

            self.set_exp_stack_ret(&mut context.exp);
            // let arg = &expr[context.ip.1];
            #[cfg(feature = "bytecode_trace")]
            {
                let t = format!("{:?} => {:?}", context.ip, arg);
                println!("{:?}", context.exp);
                println!("{}", t);
            }

            if arg.get_operator() == &BytecodeOperator::Load {
                #[cfg(feature = "perf")]
                self.bytecode_map.start_time(&BytecodeOperator::Load);
                let state = self.get_cur_mut_stack();
                Self::load_var(&mut context.exp, arg, context.vm, state, context.module);

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

    pub fn load(
        &mut self,
        module: Box<FSRObject<'a>>,
    ) -> Result<ObjId, FSRError> {
        let mut bytecode_count = 0;
        let module_id = FSRVM::leak_object(module);
        let mut context = ThreadContext {
            exp: Vec::with_capacity(10),
            ip: (0, 0),
            vm: self.get_mut_vm(),
            is_attr: false,
            last_if_test: vec![],
            break_line: vec![],
            continue_line: vec![],
            for_iter_obj: vec![],
            module_stack: vec![],
            module: Some(module_id),
        };

        let module = FSRObject::id_to_obj(module_id).as_module();
        while let Some(expr) = module.get_expr(&context.ip) {
            self.run_expr(expr, &mut context, module.get_bytecode())?;
            bytecode_count += expr.len();
        }

        Ok(module_id)
    }

    pub fn start(
        &'a mut self,
        module: Box<FSRObject<'a>>,
        vm: &'a mut FSRVM<'a>,
    ) -> Result<(), FSRError> {
        let mut bytecode_count = 0;
        self.set_vm(vm);
        let module = FSRVM::leak_object(module);
        let mut context = ThreadContext {
            exp: Vec::with_capacity(10),
            ip: (0, 0),
            vm,
            is_attr: false,
            last_if_test: vec![],
            break_line: vec![],
            continue_line: vec![],
            for_iter_obj: vec![],
            module_stack: vec![],
            module: Some(module),
        };

        let module = FSRObject::id_to_obj(module).as_module();
        while let Some(expr) = module.get_expr(&context.ip) {
            self.run_expr(expr, &mut context, module.get_bytecode())?;
            bytecode_count += expr.len();
        }

        #[cfg(feature = "perf")]
        println!("{:#?}", self.bytecode_map.acumulate);

        #[cfg(feature = "perf")]
        println!("{:#?}", self.bytecode_map.count);

        println!("count: {}", bytecode_count);

        #[cfg(feature = "alloc_trace")]
        println!(
            "reused count: {}",
            crate::backend::types::base::HEAP_TRACE.object_count()
        );
        Ok(())
    }

    pub fn call_fn(
        &mut self,
        fn_def: &'a FSRFnInner,
        args: &[ObjId],
        module: Option<ObjId>,
    ) -> Result<SValue, FSRError> {
        let mut context = ThreadContext {
            exp: Vec::with_capacity(10),
            ip: fn_def.get_ip(),
            is_attr: false,
            vm: self.get_mut_vm(),
            last_if_test: vec![],
            break_line: vec![],
            continue_line: vec![],
            for_iter_obj: vec![],
            module_stack: vec![],
            module,
        };
        {
            //self.save_ip_to_callstate(args.len(), &mut context.exp, &mut args, &mut context.ip);
            self.call_stack.push(CallFrame::new(fn_def.get_name()));
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
        let ret_val = cur.ret_val.take();
        // let v = FSRObject::id_to_obj(s);
        // println!("{:#?}", v);
        match ret_val {
            Some(s) => Ok(s),
            None => Ok(SValue::Global(0)),
        }
    }
}

#[allow(unused_imports)]
mod test {
    use crate::backend::{
        types::{base::FSRObject, module::FSRModule},
        vm::runtime::FSRVM,
    };

    use super::FSRThreadRuntime;

    #[test]
    fn test_export() {
        let source_code = r#"
        i = 0
        export("i", i)


        fn abc() {
            return 'abc'
        }

        export('abc', abc)
        "#;
        let v = FSRModule::from_code("main", source_code).unwrap();
        let mut runtime = FSRThreadRuntime::new();
        let mut vm = FSRVM::new();
        runtime.set_vm(&mut vm);
        runtime.start(Box::new(v), &mut vm).unwrap();
        
        // println!("{:?}", FSRObject::id_to_obj(v.get_object("abc").unwrap()));
    }
}
