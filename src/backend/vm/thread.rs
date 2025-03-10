#![allow(clippy::ptr_arg)]

use std::{
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, Mutex},
    thread, vec,
};

use std::borrow::Cow;

use crate::{
    backend::{
        compiler::bytecode::{ArgType, BinaryOffset, Bytecode, BytecodeArg, BytecodeOperator},
        types::{
            base::{self, FSRGlobalObjId, FSRObject, FSRRetValue, FSRValue, ObjId},
            class::FSRClass,
            class_inst::FSRClassInst,
            float::FSRFloat,
            fn_def::{FSRFn, FSRFnInner},
            integer::FSRInteger,
            list::FSRList,
            module::FSRModule,
            string::FSRString,
        },
    },
    frontend::ast::token::call,
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
        Self { vs: vec![0; 5] }
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
    module: Option<ObjId>,
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
    pub fn insert_var(
        &mut self,
        id: &u64,
        obj_id: ObjId,
        vm: Option<&Arc<Mutex<FSRVM<'a>>>>,
        add_ref: bool,
    ) {
        if self.var_map.contains_key(id) {
            let to_be_dec = self.var_map.get(id).unwrap();
            let origin_obj = FSRObject::id_to_obj(*to_be_dec);
            origin_obj.ref_dec();
            if origin_obj.count_ref() == 0 {
                if let Some(vm) = vm {
                    vm.lock().unwrap().allocator.free(*to_be_dec);
                } else {
                    FSRObject::drop_object(*to_be_dec);
                }
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

    pub fn new(_name: &'a Cow<str>, module: Option<ObjId>) -> Self {
        Self {
            var_map: TempHashMap::new(),
            reverse_ip: (0, 0),
            args: Vec::new(),
            cur_cls: None,
            ret_val: None,
            exp: None,
            module,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AttrArgs<'a> {
    father: ObjId,
    attr: ObjId,
    name: &'a str,
    call_method: bool,
}

impl<'a> AttrArgs<'a> {
    pub fn new(father: ObjId, attr: ObjId, name: &'a str, call_method: bool) -> Self {
        Self {
            father,
            attr,
            name,
            call_method,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SValue<'a> {
    Stack((u64, &'a String)),
    Attr(Box<AttrArgs<'a>>), // father, attr, name, call_method
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
        
        Ok(match self {
            SValue::Stack(s) => {
                if let Some(id) = state.get_var(&s.0) {
                    *id
                } else {
                    let module = FSRObject::id_to_obj(state.module.unwrap()).as_module();
                    let vm = thread.get_vm();
                    let v = match module.get_object(s.1) {
                        Some(s) => s,
                        None => *vm.lock().unwrap().get_global_obj_by_name(s.1).unwrap(),
                    };

                    v
                }
            }
            SValue::Global(id) => *id,
            SValue::Attr(args) => args.attr,
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
    vm: Arc<Mutex<FSRVM<'a>>>,
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
}

impl<'a> VecMap<'a> {
    pub fn insert(&mut self, _k: BytecodeOperator, f: BytecodeFn<'a>) {
        self.inner.push(f);
    }

    #[inline(always)]
    pub fn get(&self, k: &BytecodeOperator) -> Option<&BytecodeFn<'a>> {
        self.inner.get(*k as usize)
    }
}

#[derive(Default)]
pub struct FSRThreadRuntime<'a> {
    call_stack: Vec<CallFrame<'a>>,
    stack_index: usize,

    vm: Arc<Mutex<FSRVM<'a>>>,
}

impl<'a> FSRThreadRuntime<'a> {
    pub fn get_vm(&self) -> Arc<Mutex<FSRVM<'a>>> {
        self.vm.clone()
    }

    // pub fn get_mut_vm(&mut self) -> &'a mut FSRVM<'a> {
    //     unsafe { &mut *self.vm_ptr.unwrap() }
    // }

    pub fn new(base_module: ObjId, vm: Arc<Mutex<FSRVM<'a>>>) -> FSRThreadRuntime<'a> {
        Self {
            call_stack: vec![CallFrame::new(&Cow::Borrowed("base"), Some(base_module))],
            vm: vm,
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
            res =
                FSRObject::invoke_offset_method(BinaryOffset::Less, &[left, right], thread, None)?;
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
            res =
                FSRObject::invoke_offset_method(BinaryOffset::Equal, &[left, right], thread, None)?;
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

    fn pop_stack(&mut self, escape: &[ObjId]) {
        let v = self.call_stack.pop().unwrap();
        for kv in v.var_map.iter() {
            let obj = FSRObject::id_to_obj(kv);

            obj.ref_dec();
            if escape.contains(&kv) {
                continue;
            }

            if obj.count_ref() == 0 {
                self.get_vm().lock().unwrap().allocator.free(kv);
            }
            //vm.check_delete(kv.1);
        }
    }

    fn getter_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let obj_id = match context.exp.last().unwrap() {
            SValue::Stack(s) => {
                let state = self.get_cur_mut_stack();
                state.get_var(&s.0).cloned().unwrap()
            }
            SValue::Global(id) => *id,
            SValue::Attr(args) => args.attr,
            SValue::BoxObject(obj) => FSRObject::obj_to_id(obj),
        };

        let list_obj = match context.exp.get(context.exp.len() - 2).unwrap() {
            SValue::Stack(s) => {
                let state = self.get_cur_mut_stack();
                state.get_var(&s.0).cloned().unwrap()
            }
            SValue::Global(id) => *id,
            SValue::Attr(args) => args.attr,
            SValue::BoxObject(obj) => FSRObject::obj_to_id(obj),
        };

        let res = FSRObject::invoke_offset_method(
            BinaryOffset::GetItem,
            &[list_obj, obj_id],
            self,
            None,
        )?;

        // pop after finish invoke
        context.exp.pop();

        context.exp.pop();

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
                state.insert_var(var_id, id, Some(&context.vm), true);
                return Ok(false);
            }
            let obj_id = svalue.get_global_id(self)?;
            let state = self.get_cur_mut_stack();

            state.insert_var(var_id, obj_id, Some(&context.vm), true);

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
            SValue::Stack((var_id, name)) => {
                let state = self.get_cur_mut_stack();
                state.insert_var(&var_id, to_assign_obj_id, Some(&context.vm), true);
                //FSRObject::id_to_obj(context.module.unwrap()).as_module().register_object(name, fnto_a_id);
            }
            SValue::Attr(attr) => {
                let father_obj = FSRObject::id_to_mut_obj(attr.father);
                father_obj.set_attr(attr.name, to_assign_obj_id);
            }
            SValue::Global(_) => todo!(),
            SValue::BoxObject(_) => todo!(),
        }

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
        let res = FSRObject::invoke_offset_method(BinaryOffset::Add, &[v2_id, v1_id], self, None)?;

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
        // if is binary dot operator, pop last father
        if let SValue::Attr(_) = &v2 {
            context.exp.pop();
            context.is_attr = false;
        }

        let v1_id = v1.get_global_id(self)?;
        let v2_id = v2.get_global_id(self)?;
        let res = FSRObject::invoke_offset_method(BinaryOffset::Sub, &[v2_id, v1_id], self, None)?;

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

        let res = FSRObject::invoke_offset_method(BinaryOffset::Mul, &[v2_id, v1_id], self, None)?;
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
    fn binary_div_process(
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

        let res = FSRObject::invoke_offset_method(BinaryOffset::Div, &[v2_id, v1_id], self, None)?;
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

        let name = attr_id.name;
        let id = dot_father_obj.get_attr(name);

        if dot_father_obj.is_module() {
            let id = match id {
                Some(s) => s,
                None => {
                    return Err(FSRError::new(
                        format!("not have this attr: `{}`", name),
                        FSRErrCode::NoSuchObject,
                    ))
                }
            };
            context.exp.push(SValue::Global(id));
            return Ok(false);
        }
        if let Some(id) = id {
            // let obj = FSRObject::id_to_obj(id);
            // println!("{:#?}", obj);
            //context.exp.push(SValue::Global(dot_father));
            context.exp.push(SValue::Attr(Box::new(AttrArgs::new(
                dot_father, id, name, true,
            ))));
        } else {
            //context.exp.push(SValue::Global(dot_father));
            context.exp.push(SValue::Attr(Box::new(AttrArgs::new(
                dot_father,
                attr_id.attr,
                name,
                true,
            ))));
        }

        Ok(false)
    }

    fn binary_get_cls_attr_process(
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

        let name = attr_id.name;
        let id = dot_father_obj.get_attr(name);

        if dot_father_obj.is_module() {
            let id = match id {
                Some(s) => s,
                None => {
                    return Err(FSRError::new(
                        format!("not have this attr: `{}`", name),
                        FSRErrCode::NoSuchObject,
                    ))
                }
            };
            context.exp.push(SValue::Global(id));
            return Ok(false);
        }
        if let Some(id) = id {
            // let obj = FSRObject::id_to_obj(id);
            // println!("{:#?}", obj);
            //context.exp.push(SValue::Global(dot_father));
            context.exp.push(SValue::Attr(Box::new(AttrArgs::new(
                dot_father, id, name, false,
            ))));
        } else {
            //context.exp.push(SValue::Global(dot_father));
            context.exp.push(SValue::Attr(Box::new(AttrArgs::new(
                dot_father,
                attr_id.attr,
                name,
                false,
            ))));
        }

        Ok(false)
    }

    #[inline(always)]
    fn chain_get_variable(var: (u64, &String), thread: &Self, module: ObjId) -> Option<ObjId> {
        // 尝试从当前栈获取变量
        if let Some(value) = thread.get_cur_stack().get_var(&var.0) {
            Some(*value)
        } else if let Some(value) = FSRObject::id_to_obj(module).as_module().get_object(var.1) {
            Some(value)
        }
        // 尝试从全局对象中获取变量
        else if let Some(value) = thread
            .get_vm()
            .lock()
            .unwrap()
            .get_global_obj_by_name(var.1)
        {
            Some(*value)
        } else {
            None
        }
    }

    #[inline]
    fn call_process_set_args(
        args_num: usize,
        thread: &Self,
        module: ObjId,
        exp: &mut Vec<SValue<'a>>,
        args: &mut Vec<ObjId>,
    ) -> Result<(), FSRError> {
        let mut i = 0;
        while i < args_num {
            let arg = exp.pop().unwrap();
            let a_id = match arg {
                SValue::Stack(s) => match Self::chain_get_variable(s, thread, module) {
                    Some(s) => s,
                    None => {
                        return Err(FSRError::new(
                            format!("not found variable: `{}`", s.1),
                            FSRErrCode::NoSuchObject,
                        ))
                    }
                },
                SValue::Global(g) => g,
                SValue::BoxObject(obj) => FSRVM::leak_object(obj),
                SValue::Attr(a) => a.attr,
            };
            args.push(a_id);
            i += 1;
        }

        Ok(())
    }

    #[inline]
    fn save_ip_to_callstate(
        &mut self,
        exp: &mut Vec<SValue<'a>>,
        ip: &mut (usize, usize),
        module: Option<ObjId>,
    ) {
        //Self::call_process_set_args(args_num, self, exp, args);
        let state = self.get_cur_mut_stack();
        state.set_reverse_ip(*ip);
        state.exp = Some(exp.clone());
        state.module = module;
    }

    #[inline(always)]
    fn process_fsr_cls(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        cls_id: ObjId,
        args: &mut Vec<usize>,
    ) -> Result<bool, FSRError> {
        //let mut args = vec![];
        // New a object if fn_obj is fsr_cls
        let cls = FSRObject::id_to_obj(cls_id);
        if let FSRValue::Class(c) = &cls.value {
            if c.get_attr("__new__").is_none() {
                let mut self_obj = FSRObject::new();
                self_obj.set_cls(cls_id);
                self_obj.set_value(FSRValue::ClassInst(Box::new(FSRClassInst::new(
                    c.get_name(),
                ))));

                let self_id = FSRVM::register_object(self_obj);
                context.exp.push(SValue::Global(self_id));

                return Ok(false);
            }
        }

        let mut self_obj = FSRObject::new();
        self_obj.set_cls(cls_id);
        let fn_obj = FSRObject::id_to_obj(cls_id);
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
        let self_obj = FSRObject::id_to_obj(self_id);
        let self_new = self_obj.get_cls_attr("__new__");

        if let Some(id) = self_new {
            let new_obj = FSRObject::id_to_obj(id);
            if let FSRValue::Function(f) = &new_obj.value {
                self.save_ip_to_callstate(&mut context.exp, &mut context.ip, context.module);
                let mut frame = CallFrame::new(&Cow::Borrowed("__new__"), Some(f.module));
                context.module = Some(f.module);
                self.call_stack.push(frame);
            } else {
                unimplemented!()
            }

            context.exp.clear();
            for arg in args.iter().rev() {
                //obj.ref_add();
                self.get_cur_mut_stack().args.push(*arg);
            }

            let offset = new_obj.get_fsr_offset().1;
            context.ip = (offset.0, 0);
            Ok(true)
        } else {
            unimplemented!()
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

        if fn_obj.is_fsr_function() {
            let state = self.get_cur_mut_stack();
            //Save callstate
            state.set_reverse_ip(context.ip);
            state.exp = Some(context.exp.clone());
            if let FSRValue::Function(f) = &fn_obj.value {
                let mut frame = CallFrame::new(f.get_name(), Some(f.module));
                frame.module = Some(f.module);
                self.call_stack.push(frame);
            }

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
            Self::call_process_set_args(
                args_num,
                self,
                context.module.unwrap(),
                &mut context.exp,
                &mut args,
            )?;
            args.reverse();
        }

        //let ptr = vm as *mut FSRVM;
        let mut object_id: Option<ObjId> = None;
        let module = FSRObject::id_to_obj(context.module.unwrap()).as_module();
        let mut call_method = false;
        let fn_id = match context.exp.pop().unwrap() {
            SValue::Stack(s) => {
                let state = self.get_cur_mut_stack();
                if let Some(id) = state.get_var(&s.0) {
                    *id
                } else {
                    match module.get_object(s.1) {
                        Some(s) => s,
                        None => {
                            // Cache global object in call frame
                            let v = *context
                                .vm
                                .lock()
                                .unwrap()
                                .get_global_obj_by_name(s.1)
                                .unwrap();
                            state.insert_var(&s.0, v, None, false);
                            v
                        }
                    }
                }
            }
            SValue::Global(id) => id,
            SValue::Attr(attr) => {
                call_method = attr.call_method;

                if call_method == false {
                    let cls_obj = FSRObject::id_to_obj(attr.father).as_class();
                    cls_obj.get_attr(attr.name).unwrap()
                } else {
                    object_id = Some(attr.father);
                    attr.attr
                }
            }
            SValue::BoxObject(_) => todo!(),
        };

        let fn_obj = FSRObject::id_to_obj(fn_id);

        if fn_obj.is_fsr_cls() {
            let v = Self::process_fsr_cls(self, context, fn_id, &mut args)?;
            if v {
                return Ok(v);
            }
        } else if object_id.is_some() && call_method {
            let v = Self::process_fn_is_attr(self, context, object_id.unwrap(), fn_obj, &mut args)?;
            context.is_attr = false;
            if v {
                return Ok(v);
            }
        } else {
            if fn_obj.is_fsr_function() {
                self.save_ip_to_callstate(&mut context.exp, &mut context.ip, context.module);
                if let FSRValue::Function(f) = &fn_obj.value {
                    self.call_stack
                        .push(CallFrame::new(&Cow::Borrowed("tmp2"), Some(f.module)));
                }

                context.exp.clear();
                for arg in args.iter().rev() {
                    self.get_cur_mut_stack().args.push(*arg);
                }
                //let offset = fn_obj.get_fsr_offset();
                let offset = fn_obj.get_fsr_offset().1;
                if let FSRValue::Function(obj) = &fn_obj.value {
                    //println!("{:#?}", FSRObject::id_to_obj(obj.module).as_module().as_string());
                    context.module = Some(obj.module);
                }

                context.ip = (offset.0, 0);
                return Ok(true);
            } else {
                let v = fn_obj.call(&args, self, context.module).unwrap();

                if let FSRRetValue::Value(v) = v {
                    let id = FSRVM::leak_object(v);
                    //self.get_cur_mut_stack().ret_val = Some(SValue::Global(id));
                    context.exp.push(SValue::Global(id));

                    //self.pop_stack(vm, &[id]);
                } else if let FSRRetValue::GlobalId(id) = v {
                    context.exp.push(SValue::Global(id));
                    //self.get_cur_mut_stack().ret_val = Some(SValue::Global(id));

                    //self.pop_stack(vm, &[id]);
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
        let v = context.exp.pop().unwrap();
        let mut name = "";
        let test_val = match &v {
            SValue::Stack(s) => {
                name = s.1;
                if let Some(id) = state.get_var(&s.0) {
                    Some(*id)
                } else {
                    let v = match context.vm.lock().unwrap().get_global_obj_by_name(s.1) {
                        Some(o) => *o,
                        None => {
                            return Err(FSRError::new(
                                format!("not found object in test: {}", name),
                                FSRErrCode::NoSuchObject,
                            ))
                        }
                    };
                    state.insert_var(&s.0, v, None, false);
                    Some(v)
                }
            }
            SValue::Global(id) => Some(*id),
            SValue::BoxObject(obj) => Some(obj.is_true_id()),
            _ => {
                unimplemented!()
            }
        };

        let test_val = match test_val {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    format!("not found object in test: {}", name),
                    FSRErrCode::NoSuchObject,
                ))
            }
        };
        if test_val == FSRObject::false_id() || test_val == FSRObject::none_id() {
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
        if test_val == FSRObject::false_id() || test_val == FSRObject::none_id() {
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
        if test_val == FSRObject::false_id() || test_val == FSRObject::none_id() {
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

            //println!("define_fn: {}", FSRObject::id_to_obj(context.module.unwrap()).as_module().as_string());
            let fn_obj = FSRFn::from_fsr_fn(
                &name.1,
                (context.ip.0 + 1, 0),
                args,
                bc,
                context.module.unwrap(),
            );
            fn_obj.ref_add();
            let fn_id = FSRVM::register_object(fn_obj);
            if let Some(cur_cls) = &mut state.cur_cls {
                cur_cls.insert_attr_id(name.1, fn_id);
                context.ip = (context.ip.0 + *n as usize + 2, 0);
                return Ok(true);
            }

            state.insert_var(&name.0, fn_id, Some(&context.vm), true);
            FSRObject::id_to_obj(context.module.unwrap())
                .as_module()
                .register_object(name.1, fn_id);

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
        self.pop_stack(&[]);
        let cur = self.get_cur_mut_stack();
        context.ip = (cur.reverse_ip.0, cur.reverse_ip.1 + 1);
        context.module = cur.module;
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
                context.exp.push(SValue::Global(FSRObject::true_id()))
            } else {
                context.exp.push(SValue::Global(FSRObject::false_id()))
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

        self.pop_stack(&[v]);
        let cur = self.get_cur_mut_stack();
        cur.ret_val = Some(SValue::Global(v));
        context.ip = (cur.reverse_ip.0, cur.reverse_ip.1);
        context.module = cur.module;
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
            state.insert_var(s_id, v, Some(&context.vm), true);
        }
        Ok(false)
    }

    // this is a special function for load list
    // will load the list to the stack
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

    fn read_code_from_module(module_name: &Vec<String>) -> Result<String, FSRError> {
        let mut module_path = PathBuf::from_str("fs_modules").unwrap();

        for m in module_name.iter().enumerate() {
            if m.0 == module_name.len() - 1 {
                module_path = module_path.join(format!("{}.fs", m.1));
            } else {
                module_path = module_path.join(m.1);
            }
        }

        let code = std::fs::read_to_string(module_path).unwrap();

        Ok(code)
    }

    fn process_import(
        self: &mut FSRThreadRuntime<'a>,
        exp: &mut Vec<SValue<'a>>,
        bc: &BytecodeArg,
        context: ObjId,
    ) -> Result<bool, FSRError> {
        if let ArgType::ImportModule(v, module_name) = bc.get_arg() {
            let code = Self::read_code_from_module(module_name)?;

            let module = FSRModule::from_code(&module_name.join("."), &code)?;
            let obj_id = { self.load(Box::new(module))? };

            let frame = self.get_cur_mut_stack();
            frame.insert_var(v, obj_id, None, true);
            FSRObject::id_to_obj(context)
                .as_module()
                .register_object(module_name.last().unwrap(), obj_id);
            return Ok(false);
        }
        unimplemented!()
    }

    fn end_class_def(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bc: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        if let ArgType::Variable(id, name) = bc.get_arg() {
            let state = self.get_cur_mut_stack();
            let mut cls_obj = FSRObject::new();
            cls_obj.set_cls(FSRGlobalObjId::ClassCls as ObjId);
            let obj = state.cur_cls.take().unwrap();

            let name = obj.get_name().to_string();
            cls_obj.set_value(FSRValue::Class(obj));
            let obj_id = FSRVM::register_object(cls_obj);
            state.insert_var(id, obj_id, None, true);
            FSRObject::id_to_obj(context.module.unwrap())
                .as_module()
                .register_object(&name, obj_id);
        } else {
            unimplemented!()
        }

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

    // process logic or operator in bytecode
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

    fn not_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bc: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let _state = self.get_cur_mut_stack();
        let v1 = match context.exp.last() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary add 1",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let v1_id = v1.get_global_id(self)?;
        let mut target = false;
        if (FSRObject::none_id() == v1_id || FSRObject::false_id() == v1_id) {
            target = true;
        } else {
            target = false;
        }

        context.exp.pop();

        if target {
            context.exp.push(SValue::Global(FSRObject::true_id()));
        } else {
            context.exp.push(SValue::Global(FSRObject::false_id()));
        }

        Ok(false)
    }

    fn process_logic_or_load(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bc: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let _state = self.get_cur_mut_stack();
        let v1 = match context.exp.last() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary add 1",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let v2 = match context.exp.get(context.exp.len() - 2) {
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

        if (FSRObject::none_id() != v1_id || FSRObject::false_id() != v1_id)
            || (FSRObject::none_id() != v2_id || FSRObject::false_id() != v2_id)
        {
            context.exp.push(SValue::Global(FSRObject::true_id()));
        } else {
            context.exp.push(SValue::Global(FSRObject::false_id()));
        }

        context.exp.pop();
        context.exp.pop();

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
            BytecodeOperator::BinarySub => Self::binary_sub_process(self, context, bytecode, bc),
            BytecodeOperator::Import => {
                Self::process_import(self, &mut context.exp, bytecode, context.module.unwrap())
            }
            BytecodeOperator::BinaryDiv => Self::binary_div_process(self, context, bytecode, bc),
            BytecodeOperator::NotOperator => Self::not_process(self, context, bytecode, bc),
            BytecodeOperator::BinaryClassGetter => {
                Self::binary_get_cls_attr_process(self, context, bytecode, bc)
            }
            BytecodeOperator::Getter => Self::getter_process(self, context, bytecode, bc),
            _ => {
                panic!("not implement for {:#?}", op);
            }
        }?;

        if v {
            return Ok(v);
        }

        Ok(false)
    }

    #[inline(always)]
    fn load_var(
        exp_stack: &mut Vec<SValue<'a>>,
        arg: &'a BytecodeArg,
        vm: &Arc<Mutex<FSRVM<'a>>>,
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
            if !vm.lock().unwrap().has_int_const(i) {
                let obj = FSRInteger::new_inst(*i);
                obj.set_not_delete();
                vm.lock().unwrap().insert_int_const(i, obj);
            }

            let id = vm.lock().unwrap().get_int_const(i).unwrap();

            exp_stack.push(SValue::Global(id));
        } else if let ArgType::ConstFloat(c_id, f) = arg.get_arg() {
            //let float_const = Self::load_float_const(f, vm);
            if let Some(m) = module {
                let m = FSRObject::id_to_obj(m).as_module();
                if let Some(id) = m.get_bytecode().const_table.table.get(*c_id as usize) {
                    if id != &0 {
                        exp_stack.push(SValue::Global(*id));
                        return;
                    }
                }
            }

            if !vm.lock().unwrap().has_float_const(f) {
                let obj = FSRFloat::new_inst(*f);
                obj.set_not_delete();
                vm.lock().unwrap().insert_float_const(f, obj);
            }

            let id = vm.lock().unwrap().get_float_const(f).unwrap();
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

            let mut str_const = vm.lock().unwrap().get_str_const(i.as_str());
            if str_const.is_none() {
                let obj = FSRString::new_inst(Cow::Owned(i.clone()));
                obj.set_not_delete();
                str_const = Some(vm.lock().unwrap().insert_str_const(i.as_str(), obj));
            }

            let id = str_const.unwrap();
            exp_stack.push(SValue::Global(id));
        } else if let ArgType::Attr(_, name) = arg.get_arg() {
            exp_stack.push(SValue::Attr(Box::new(AttrArgs::new(0, 0, name, true))));
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
            //std::mem::replace(exp_stack, s);
            stack.exp = None;
        }

        if let Some(s) = stack.ret_val.take() {
            exp_stack.push(s);
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
                let state = self.get_cur_mut_stack();
                Self::load_var(&mut context.exp, arg, &context.vm, state, context.module);
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

    pub fn load(&mut self, module: Box<FSRObject<'a>>) -> Result<ObjId, FSRError> {
        let mut bytecode_count = 0;
        let module_id = FSRVM::leak_object(module);
        let mut context = ThreadContext {
            exp: Vec::with_capacity(10),
            ip: (0, 0),
            vm: self.get_vm(),
            is_attr: false,
            last_if_test: vec![],
            break_line: vec![],
            continue_line: vec![],
            for_iter_obj: vec![],
            module_stack: vec![],
            module: Some(module_id),
        };

        self.call_stack.push(CallFrame::new(
            &Cow::Borrowed("load_module"),
            Some(module_id),
        ));

        let module = FSRObject::id_to_obj(module_id).as_module();
        while let Some(expr) = module.get_expr(&context.ip) {
            self.run_expr(expr, &mut context, module.get_bytecode())?;
            bytecode_count += expr.len();
        }

        self.call_stack.pop();

        Ok(module_id)
    }

    pub fn start(&mut self, module_id: ObjId) -> Result<(), FSRError> {
        let mut bytecode_count = 0;

        let mut context = ThreadContext {
            exp: Vec::with_capacity(10),
            ip: (0, 0),
            vm: self.get_vm(),
            is_attr: false,
            last_if_test: vec![],
            break_line: vec![],
            continue_line: vec![],
            for_iter_obj: vec![],
            module_stack: vec![],
            module: Some(module_id),
        };

        let mut module = FSRObject::id_to_obj(module_id).as_module();
        while let Some(expr) = FSRObject::id_to_obj(context.module.unwrap())
            .as_module()
            .get_expr(&context.ip)
        {
            #[cfg(feature = "bytecode_trace")]
            {
                println!(
                    "cur_module: {}",
                    FSRObject::id_to_obj(context.module.unwrap())
                        .as_module()
                        .as_string()
                )
            }
            self.run_expr(expr, &mut context, module.get_bytecode())?;
            bytecode_count += expr.len();
            module = FSRObject::id_to_obj(context.module.unwrap()).as_module();
        }

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
            vm: self.get_vm(),
            last_if_test: vec![],
            break_line: vec![],
            continue_line: vec![],
            for_iter_obj: vec![],
            module_stack: vec![],
            module,
        };
        {
            //self.save_ip_to_callstate(args.len(), &mut context.exp, &mut args, &mut context.ip);
            self.call_stack
                .push(CallFrame::new(fn_def.get_name(), module));
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
    use std::sync::{Arc, Mutex};

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
        let base_module = FSRVM::leak_object(Box::new(v));

        let mut vm = Arc::new(Mutex::new(FSRVM::new()));
        let mut runtime = FSRThreadRuntime::new(base_module, vm);
        runtime.start(base_module).unwrap();

        // println!("{:?}", FSRObject::id_to_obj(v.get_object("abc").unwrap()));
    }

    #[test]
    fn test_float() {
        let source_code = r#"
        i = 1.1
        b = 1.2
        dump(i + b)
        "#;
        let v = FSRModule::from_code("main", source_code).unwrap();
        let base_module = FSRVM::leak_object(Box::new(v));

        let mut vm = Arc::new(Mutex::new(FSRVM::new()));
        let mut runtime = FSRThreadRuntime::new(base_module, vm);
        runtime.start(base_module).unwrap();
    }

    #[test]
    fn test_print_str() {
        let source_code = r#"
        class Test {
            fn __new__(self) {
                self.abc = 123
                return self
            }

            fn __str__(self) {
                return 'abc'
            }
        }
        t = Test()
        println(t)
        "#;
        let v = FSRModule::from_code("main", source_code).unwrap();
        let base_module = FSRVM::leak_object(Box::new(v));
        let mut vm = Arc::new(Mutex::new(FSRVM::new()));
        let mut runtime = FSRThreadRuntime::new(base_module, vm);
        runtime.start(base_module).unwrap();
    }

    #[test]
    fn test_binary_div() {
        let source_code = r#"
        a = 1.0 / 2.0
        println(a)
        "#;
        let v = FSRModule::from_code("main", source_code).unwrap();
        let base_module = FSRVM::leak_object(Box::new(v));
        let mut vm = Arc::new(Mutex::new(FSRVM::new()));
        let mut runtime = FSRThreadRuntime::new(base_module, vm);
        runtime.start(base_module).unwrap();
    }

    #[test]
    fn test_class_without_new() {
        let source_code = r#"
        class Test {
            fn abc() {
                println("abc")
            }
        }
        Test::abc()
        "#;
        let v = FSRModule::from_code("main", source_code).unwrap();
        let base_module = FSRVM::leak_object(Box::new(v));
        let mut vm = Arc::new(Mutex::new(FSRVM::new()));
        let mut runtime = FSRThreadRuntime::new(base_module, vm);
        runtime.start(base_module).unwrap();
    }

    #[test]
    fn get_list_item() {
        let source_code = r#"
        a = [1, 2, 3]
        println(a[0])
        println(a[1])
        println(a[2])
        "#;
        let v = FSRModule::from_code("main", source_code).unwrap();
        let base_module = FSRVM::leak_object(Box::new(v));
        let mut vm = Arc::new(Mutex::new(FSRVM::new()));
        let mut runtime = FSRThreadRuntime::new(base_module, vm);
        runtime.start(base_module).unwrap();
    }

    #[test]
    fn test_svalue_size() {
        println!("svalue size: {}", std::mem::size_of::<super::SValue>());
    }
}
