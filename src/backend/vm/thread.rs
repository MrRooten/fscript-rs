#![allow(clippy::ptr_arg)]

use std::{
    cell::RefCell,
    ops::Range,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, Mutex},
    thread,
    time::Instant,
    vec,
};

use std::borrow::Cow;

use crate::{
    backend::{
        compiler::bytecode::{ArgType, BinaryOffset, Bytecode, BytecodeArg, BytecodeOperator},
        memory::size_alloc::FSRObjectAllocator,
        types::{
            base::{self, FSRGlobalObjId, FSRObject, FSRRetValue, FSRValue, ObjId},
            class::FSRClass,
            class_inst::FSRClassInst,
            error::FSRException,
            float::FSRFloat,
            fn_def::{FSRFn, FSRFnInner},
            integer::FSRInteger,
            list::FSRList,
            module::FSRModule,
            range::FSRRange,
            string::FSRString,
        },
    },
    frontend::ast::token::call,
    utils::error::{FSRErrCode, FSRError},
};

use super::{free_list::FrameFreeList, virtual_machine::FSRVM};

#[derive(Debug)]
pub struct IndexMap {
    vs: Vec<ObjId>,
}

pub struct IndexIterator<'a> {
    vs: core::slice::Iter<'a, ObjId>,
}

#[allow(clippy::new_without_default)]
#[allow(unused)]
impl IndexMap {
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
        Self { vs: vec![0; 4] }
    }

    pub fn iter(&self) -> IndexIterator {
        IndexIterator { vs: self.vs.iter() }
    }

    pub fn clear(&mut self) {
        self.vs.fill(0);
    }
}

impl Iterator for IndexIterator<'_> {
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
    var_map: IndexMap,
    reverse_ip: (usize, usize),
    args: Vec<ObjId>,
    cur_cls: Option<Box<FSRClass<'a>>>,
    ret_val: Option<SValue<'a>>,
    pub(crate) exp: Option<Vec<SValue<'a>>>,
    pub(crate) module: ObjId,
    catch_ends: Vec<(usize, usize)>,
    pub(crate) handling_exception: Option<ObjId>,
}

impl<'a> CallFrame<'a> {
    pub fn clear(&mut self) {
        self.var_map.clear();
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
        allocator: Option<&FSRObjectAllocator<'a>>,
        add_ref: bool,
    ) {
        if self.var_map.contains_key(id) {
            let to_be_dec = self.var_map.get(id).unwrap();
            let origin_obj = FSRObject::id_to_obj(*to_be_dec);
            origin_obj.ref_dec();
            if origin_obj.count_ref() == 0 {
                if let Some(rt) = allocator {
                    rt.free(*to_be_dec);
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

    pub fn new(_name: &'a str, module: ObjId) -> Self {
        Self {
            var_map: IndexMap::new(),
            reverse_ip: (0, 0),
            args: Vec::new(),
            cur_cls: None,
            ret_val: None,
            exp: None,
            module,
            catch_ends: vec![],
            handling_exception: None,
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
    pub fn new(father: ObjId, attr: ObjId, name: &'a str, call_method: bool) -> Box<Self> {
        Box::new(Self {
            father,
            attr,
            name,
            call_method,
        })
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
        Ok(match self {
            SValue::Stack(s) => {
                let state = thread.get_cur_frame();
                if let Some(id) = state.get_var(&s.0) {
                    *id
                } else {
                    let module = FSRObject::id_to_obj(state.module).as_module();
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

    #[inline(always)]
    pub fn drop_box(self, allocator: &FSRObjectAllocator<'a>) {
        match self {
            Self::BoxObject(obj) => {
                allocator.free_object(obj);
            }
            Self::Global(id) => {
                if FSRObject::is_sp_object(id) {
                    return;
                }
                let obj = FSRObject::id_to_obj(id);
                if obj.count_ref() == 0 {
                    allocator.free(id);
                }
            }
            _ => {}
        }
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
    call_end: Vec<()>,
    exp: Vec<SValue<'a>>,
    ip: (usize, usize),
    is_attr: bool,
    pub(crate) module: ObjId,
}

impl<'a> ThreadContext<'a> {
    pub fn new_context(vm: Arc<Mutex<FSRVM<'a>>>, module_id: ObjId) -> Self {
        ThreadContext {
            exp: Vec::with_capacity(10),
            ip: (0, 0),
            is_attr: false,
            module: module_id,
            call_end: vec![()],
        }
    }

    pub fn clear_exp(&mut self, allocator: &FSRObjectAllocator<'a>) {
        while let Some(s) = self.exp.pop() {
            s.drop_box(allocator);
        }
    }
}

pub struct FSRThreadRuntime<'a> {
    pub(crate) call_frames: Vec<CallFrame<'a>>,
    frame_index: usize,
    pub(crate) frame_free_list: FrameFreeList<'a>,
    vm: Arc<Mutex<FSRVM<'a>>>,
    pub(crate) thread_allocator: FSRObjectAllocator<'a>,
    last_if_test: Vec<bool>,
    break_line: Vec<usize>,
    continue_line: Vec<usize>,
    for_iter_obj: Vec<ObjId>, // for i in a , the a will call __iter__ Iterator object
    ref_for_obj: Vec<ObjId>,  // use for ref obj, like (for i in a) ref about a variable
    is_break: bool,
    pub(crate) exception: Option<ObjId>,
}

impl<'a> FSRThreadRuntime<'a> {
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

    pub fn get_vm(&self) -> Arc<Mutex<FSRVM<'a>>> {
        self.vm.clone()
    }

    // pub fn get_mut_vm(&mut self) -> &'a mut FSRVM<'a> {
    //     unsafe { &mut *self.vm_ptr.unwrap() }
    // }

    pub fn new(base_module: ObjId, vm: Arc<Mutex<FSRVM<'a>>>) -> FSRThreadRuntime<'a> {
        Self {
            call_frames: vec![CallFrame::new("base", base_module)],
            vm,
            frame_index: 0,
            frame_free_list: FrameFreeList::new_list(),
            thread_allocator: FSRObjectAllocator::new(),
            last_if_test: vec![],
            break_line: vec![],
            continue_line: vec![],
            for_iter_obj: vec![],
            exception: None,
            ref_for_obj: vec![],
            is_break: false,
        }
    }

    pub fn push_frame(&mut self) {
        self.frame_index += 1;

        if let Some(s) = self.call_frames.get_mut(self.frame_index) {
            s.clear();
        }
    }

    pub fn pop_frame(&mut self) -> Option<&mut CallFrame<'a>> {
        self.frame_index -= 1;
        self.call_frames.get_mut(self.frame_index + 1)
    }

    #[inline(always)]
    pub fn get_cur_mut_frame(&mut self) -> &mut CallFrame<'a> {
        self.call_frames.last_mut().unwrap()
    }

    #[inline(always)]
    fn get_cur_frame(&self) -> &CallFrame<'a> {
        self.call_frames.last().unwrap()
    }

    #[inline(always)]
    fn compare(
        left: ObjId,
        right: ObjId,
        op: &str,
        thread: &mut Self,
        context: &ThreadContext,
    ) -> Result<bool, FSRError> {
        let res;

        if op.eq(">") {
            res = FSRObject::invoke_offset_method(
                BinaryOffset::Greater,
                &[left, right],
                thread,
                context.module,
            )?;
        } else if op.eq("<") {
            res = FSRObject::invoke_offset_method(
                BinaryOffset::Less,
                &[left, right],
                thread,
                context.module,
            )?;
        } else if op.eq(">=") {
            res = FSRObject::invoke_offset_method(
                BinaryOffset::GreatEqual,
                &[left, right],
                thread,
                context.module,
            )?;
        } else if op.eq("<=") {
            res = FSRObject::invoke_offset_method(
                BinaryOffset::LessEqual,
                &[left, right],
                thread,
                context.module,
            )?;
        } else if op.eq("==") {
            res = FSRObject::invoke_offset_method(
                BinaryOffset::Equal,
                &[left, right],
                thread,
                context.module,
            )?;
        } else if op.eq("!=") {
            res = FSRObject::invoke_offset_method(
                BinaryOffset::NotEqual,
                &[left, right],
                thread,
                context.module,
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
        let v = self.call_frames.pop().unwrap();
        for kv in v.var_map.iter() {
            let obj = FSRObject::id_to_obj(kv);

            obj.ref_dec();
            if escape.contains(&kv) {
                continue;
            }

            if obj.count_ref() == 0 {
                self.thread_allocator.free(kv);
            }
            //vm.check_delete(kv.1);
        }
        self.frame_free_list.free(v);
    }

    fn getter_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let obj_id = match context.exp.last().unwrap() {
            SValue::Stack(s) => {
                let state = self.get_cur_mut_frame();
                state.get_var(&s.0).cloned().unwrap()
            }
            SValue::Global(id) => *id,
            SValue::Attr(args) => args.attr,
            SValue::BoxObject(obj) => FSRObject::obj_to_id(obj),
        };

        let list_obj = match context.exp.get(context.exp.len() - 2).unwrap() {
            SValue::Stack(s) => {
                let state = self.get_cur_mut_frame();
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
            context.module,
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
                //let state = self.get_cur_mut_frame();
                let state = self.call_frames.last_mut().unwrap();
                let id = FSRVM::leak_object(obj);

                // println!("{:#?}", obj);
                state.insert_var(var_id, id, Some(&self.thread_allocator), true);
                return Ok(false);
            }
            let obj_id = svalue.get_global_id(self)?;
            let state = self.call_frames.last_mut().unwrap();

            state.insert_var(var_id, obj_id, Some(&self.thread_allocator), true);

            return Ok(false);
        }

        //Assign variable name
        let assign_id = match context.exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new("", FSRErrCode::EmptyExpStack));
            }
        };

        let to_assign_obj_id = match context.exp.pop().unwrap() {
            SValue::Stack(s) => {
                let state = self.get_cur_frame();
                if let Some(id) = state.get_var(&s.0) {
                    *id
                } else {
                    let module = FSRObject::id_to_obj(state.module).as_module();
                    let vm = self.get_vm();
                    let v = match module.get_object(s.1) {
                        Some(s) => s,
                        None => *vm.lock().unwrap().get_global_obj_by_name(s.1).unwrap(),
                    };

                    v
                }
            }
            SValue::Global(id) => id,
            SValue::Attr(args) => args.attr,
            SValue::BoxObject(fsrobject) => FSRVM::leak_object(fsrobject),
        };

        match assign_id {
            SValue::Stack((var_id, _)) => {
                let state = self.call_frames.last_mut().unwrap();
                state.insert_var(
                    &var_id,
                    to_assign_obj_id,
                    Some(&self.thread_allocator),
                    true,
                );
                //FSRObject::id_to_obj(context.module.unwrap()).as_module().register_object(name, fnto_a_id);
            }
            SValue::Attr(attr) => {
                let father_obj = FSRObject::id_to_mut_obj(attr.father);
                father_obj.set_attr(attr.name, to_assign_obj_id);
            }
            SValue::Global(_) => todo!(),
            SValue::BoxObject(b) => todo!(),
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
        let v1 = match context.exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary add 1",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        // if let SValue::Attr(_) = &v1 {
        //     context.exp.pop();
        //     context.is_attr = false;
        // }

        let v2 = match context.exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary add 2",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        // if let SValue::Attr(_) = &v2 {
        //     context.exp.pop();
        //     context.is_attr = false;
        // }

        let v1_id = v1.get_global_id(self)?;
        let v2_id = v2.get_global_id(self)?;
        let res = FSRObject::invoke_offset_method(
            BinaryOffset::Add,
            &[v2_id, v1_id],
            self,
            context.module,
        )?;

        v1.drop_box(&self.thread_allocator);
        v2.drop_box(&self.thread_allocator);

        match res {
            FSRRetValue::Value(object) => {
                context.exp.push(SValue::BoxObject(object));
            }
            FSRRetValue::GlobalId(res_id) => {
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
        let v1 = match context.exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary sub 1",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };
        // if let SValue::Attr(_) = &v1 {
        //     context.exp.pop();
        //     context.is_attr = false;
        // }

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
        // if let SValue::Attr(_) = &v2 {
        //     context.exp.pop();
        //     context.is_attr = false;
        // }

        let v1_id = v1.get_global_id(self)?;
        let v2_id = v2.get_global_id(self)?;
        let res = FSRObject::invoke_offset_method(
            BinaryOffset::Sub,
            &[v2_id, v1_id],
            self,
            context.module,
        )?;
        v1.drop_box(&self.thread_allocator);
        v2.drop_box(&self.thread_allocator);
        match res {
            FSRRetValue::Value(object) => {
                context.exp.push(SValue::BoxObject(object));
            }
            FSRRetValue::GlobalId(res_id) => {
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
        let v1 = match context.exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary mul 1",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        // if let SValue::Attr(_) = &v1 {
        //     context.exp.pop();
        //     context.is_attr = false;
        // }

        let v2 = match context.exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary mul 2",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        // if let SValue::Attr(_) = &v2 {
        //     context.exp.pop();
        //     context.is_attr = false;
        // }

        let v1_id = v1.get_global_id(self)?;
        let v2_id = v2.get_global_id(self)?;

        let res = FSRObject::invoke_offset_method(
            BinaryOffset::Mul,
            &[v2_id, v1_id],
            self,
            context.module,
        )?;

        v1.drop_box(&self.thread_allocator);
        v2.drop_box(&self.thread_allocator);

        match res {
            FSRRetValue::Value(object) => {
                context.exp.push(SValue::BoxObject(object));
            }
            FSRRetValue::GlobalId(res_id) => {
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
        let v1 = match context.exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary mul 1",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        // if let SValue::Attr(_) = &v1 {
        //     context.exp.pop();
        //     context.is_attr = false;
        // }

        let v2 = match context.exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary mul 2",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        // if let SValue::Attr(_) = &v2 {
        //     context.exp.pop();
        //     context.is_attr = false;
        // }

        let v1_id = v1.get_global_id(self)?;
        let v2_id = v2.get_global_id(self)?;

        let res = FSRObject::invoke_offset_method(
            BinaryOffset::Div,
            &[v2_id, v1_id],
            self,
            context.module,
        )?;
        match res {
            FSRRetValue::Value(object) => {
                context.exp.push(SValue::BoxObject(object));
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
            context
                .exp
                .push(SValue::Attr(AttrArgs::new(dot_father, id, name, true)));
        } else {
            //context.exp.push(SValue::Global(dot_father));
            context.exp.push(SValue::Attr(AttrArgs::new(
                dot_father,
                attr_id.attr,
                name,
                true,
            )));
        }

        Ok(false)
    }

    fn binary_get_cls_attr_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
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
            context
                .exp
                .push(SValue::Attr(AttrArgs::new(dot_father, id, name, false)));
        } else {
            //context.exp.push(SValue::Global(dot_father));
            context.exp.push(SValue::Attr(AttrArgs::new(
                dot_father,
                attr_id.attr,
                name,
                false,
            )));
        }

        Ok(false)
    }

    fn binary_range_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
    ) -> Result<bool, FSRError> {
        let rhs = match context.exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary mul 1",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        // if let SValue::Attr(_) = &v1 {
        //     context.exp.pop();
        //     context.is_attr = false;
        // }

        let lhs = match context.exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary mul 2",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let lhs_id = lhs.get_global_id(self)?;
        let rhs_id = rhs.get_global_id(self)?;

        let start = FSRObject::id_to_obj(lhs_id);
        let end = FSRObject::id_to_obj(rhs_id);

        if let FSRValue::Integer(start) = start.value {
            if let FSRValue::Integer(end) = end.value {
                let range = FSRRange {
                    range: Range { start, end },
                };

                let obj = self
                    .thread_allocator
                    .new_object(FSRValue::Range(range), FSRGlobalObjId::RangeCls as ObjId);

                obj.ref_add();
                let id = FSRVM::leak_object(obj);

                context.exp.push(SValue::Global(id));

                return Ok(false);
            }
        }
        unimplemented!()
    }

    #[inline(always)]
    fn chain_get_variable(var: (u64, &String), thread: &Self, module: ObjId) -> Option<ObjId> {
        // 尝试从当前栈获取变量
        if let Some(value) = thread.get_cur_frame().get_var(&var.0) {
            Some(*value)
        } else if let Some(value) = FSRObject::id_to_obj(module).as_module().get_object(var.1) {
            Some(value)
        }
        // 尝试从全局对象中获取变量
        else {
            thread
                .get_vm()
                .lock()
                .unwrap()
                .get_global_obj_by_name(var.1)
                .copied()
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

    // exp will be cleared after call
    #[inline]
    fn save_ip_to_callstate(
        &mut self,
        exp: &mut Vec<SValue<'a>>,
        ip: &mut (usize, usize),
        module: ObjId,
    ) {
        //Self::call_process_set_args(args_num, self, exp, args);
        let state = self.get_cur_mut_frame();
        state.set_reverse_ip(*ip);
        //state.exp = Some(exp.clone());

        state.exp = Some(std::mem::take(exp));
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
        // let state = self.get_cur_mut_stack();
        // state.set_reverse_ip(context.ip);

        // state.exp = Some(context.exp.clone());
        self.save_ip_to_callstate(&mut context.exp, &mut context.ip, context.module);
        let self_obj = FSRObject::id_to_obj(self_id);
        let self_new = self_obj.get_cls_attr("__new__");

        if let Some(id) = self_new {
            let new_obj = FSRObject::id_to_obj(id);
            if let FSRValue::Function(f) = &new_obj.value {
                let mut frame = self.frame_free_list.new_frame("__new__", f.module);
                context.module = f.module;
                self.call_frames.push(frame);
            } else {
                unimplemented!()
            }

            for arg in args.iter().rev() {
                //obj.ref_add();
                self.get_cur_mut_frame().args.push(*arg);
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
            let state = self.get_cur_mut_frame();
            //Save callstate
            state.set_reverse_ip(context.ip);
            state.exp = Some(std::mem::take(&mut context.exp));
            if let FSRValue::Function(f) = &fn_obj.value {
                let mut frame = self.frame_free_list.new_frame(f.get_name(), f.module);
                frame.module = f.module;
                self.call_frames.push(frame);
            }

            for arg in args.iter().rev() {
                self.get_cur_mut_frame().args.push(*arg);
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

    fn try_get_obj_by_name(
        &mut self,
        c_id: u64,
        name: &str,
        module: &FSRModule,
        context: &mut ThreadContext<'a>,
    ) -> Option<ObjId> {
        {
            let state = self.get_cur_mut_frame();
            if let Some(id) = state.get_var(&c_id) {
                return Some(*id);
            }
        }

        match module.get_object(name) {
            Some(s) => Some(s),
            None => {
                // Cache global object in call frame
                let v = self
                    .get_vm()
                    .lock()
                    .unwrap()
                    .get_global_obj_by_name(name)
                    .cloned()?;
                let state = self.get_cur_mut_frame();
                state.insert_var(&c_id, v, None, true);
                Some(v)
            }
        }
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
                context.module,
                &mut context.exp,
                &mut args,
            )?;
            args.reverse();
        }

        //let ptr = vm as *mut FSRVM;
        let mut object_id: Option<ObjId> = None;
        let module = FSRObject::id_to_obj(context.module).as_module();
        let mut call_method = false;
        let fn_id = match context.exp.pop().unwrap() {
            SValue::Stack(s) => self.try_get_obj_by_name(s.0, s.1, module, context).unwrap(),
            SValue::Global(id) => id,
            SValue::Attr(attr) => {
                call_method = attr.call_method;

                if !call_method {
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
        } else if fn_obj.is_fsr_function() {
            context.call_end.push(());
            self.save_ip_to_callstate(&mut context.exp, &mut context.ip, context.module);
            if let FSRValue::Function(f) = &fn_obj.value {
                self.call_frames
                    .push(self.frame_free_list.new_frame("tmp2", f.module));
            }

            for arg in args.iter().rev() {
                self.get_cur_mut_frame().args.push(*arg);
            }
            //let offset = fn_obj.get_fsr_offset();
            let offset = fn_obj.get_fsr_offset().1;
            if let FSRValue::Function(obj) = &fn_obj.value {
                //println!("{:#?}", FSRObject::id_to_obj(obj.module).as_module().as_string());
                context.module = obj.module;
            }

            context.ip = (offset.0, 0);
            return Ok(true);
        } else {
            let v = match fn_obj.call(&args, self, context.module) {
                Ok(o) => o,
                Err(e) => {
                    if e.code == FSRErrCode::RuntimeError {
                        self.exception = e.exception;
                        return Ok(false);
                    }

                    panic!()
                }
            };

            if let FSRRetValue::Value(v) = v {
                let id = FSRVM::leak_object(v);

                context.exp.push(SValue::Global(id));
            } else if let FSRRetValue::GlobalId(id) = v {
                context.exp.push(SValue::Global(id));
            }
        }

        Ok(false)
    }

    fn try_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
    ) -> Result<bool, FSRError> {
        let catch_line = match bytecode.get_arg() {
            ArgType::TryCatch(start_catch, end_catch) => (*start_catch, *end_catch),
            _ => {
                return Err(FSRError::new(
                    "not a try catch line",
                    FSRErrCode::NotValidArgs,
                ))
            }
        };

        self.get_cur_mut_frame().catch_ends.push((
            context.ip.0 + catch_line.0 as usize,
            context.ip.0 + catch_line.1 as usize,
        ));
        Ok(false)
    }

    fn try_end(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
    ) -> Result<bool, FSRError> {
        let end = self.get_cur_mut_frame().catch_ends.pop().unwrap();
        context.ip = (end.1 as usize, 0);
        Ok(true)
    }

    fn catch_end(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
    ) -> Result<bool, FSRError> {
        let state = self.get_cur_mut_frame();
        //state.catch_ends.pop().unwrap();
        state.handling_exception.take();
        Ok(true)
    }

    fn if_test_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let state = self.get_cur_mut_frame();
        let v = context.exp.pop().unwrap();
        let mut name = "";
        let test_val = match &v {
            SValue::Stack(s) => {
                name = s.1;
                if let Some(id) = state.get_var(&s.0) {
                    Some(*id)
                } else {
                    let v = self
                        .get_vm()
                        .lock()
                        .unwrap()
                        .get_global_obj_by_name(name)
                        .cloned()
                        .unwrap();
                    let state = self.get_cur_mut_frame();
                    state.insert_var(&s.0, v, None, true);
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
                self.push_last_if_test(false);
                return Ok(true);
            }
        }
        self.push_last_if_test(true);
        Ok(false)
    }

    #[inline(always)]
    fn if_end(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        self.pop_last_if_test();
        Ok(false)
    }

    #[inline(always)]
    fn else_if_test_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let state = self.get_cur_mut_frame();
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
                self.false_last_if_test();
                return Ok(true);
            }
        }
        self.true_last_if_test();
        Ok(false)
    }

    #[inline(always)]
    fn else_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        if self.peek_last_if_test() {
            if let ArgType::IfTestNext(n) = bytecode.get_arg() {
                context.ip = (context.ip.0 + n.0 as usize + 1_usize, 0);
                return Ok(true);
            }
        }

        self.false_last_if_test();
        Ok(false)
    }

    #[inline(always)]
    fn else_if_match(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        if self.peek_last_if_test() {
            if let ArgType::IfTestNext(n) = bytecode.get_arg() {
                context.ip = (context.ip.0 + n.0 as usize + 1_usize, 0);
                return Ok(true);
            }
        }
        self.false_last_if_test();
        Ok(false)
    }

    #[inline(always)]
    fn break_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        self.is_break = true;
        let l = self.continue_line.len();
        let continue_line = self.continue_line[l - 1];
        context.ip = (continue_line, 0);
        Ok(true)
    }

    #[inline(always)]
    fn continue_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let l = self.continue_line.len();
        let continue_line = self.continue_line[l - 1];
        context.ip = (continue_line, 0);
        Ok(true)
    }

    fn for_block_ref(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
    ) -> Result<bool, FSRError> {
        let obj_id = context.exp.last().unwrap().get_global_id(self)?;
        FSRObject::id_to_obj(obj_id).ref_add();
        self.ref_for_obj.push(obj_id);
        Ok(false)
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
            obj.ref_add();
            FSRVM::leak_object(obj)
        } else {
            let id = iter_obj.get_global_id(self)?;
            FSRObject::id_to_obj(id).ref_add();
            id
        };
        // let v = FSRObject::id_to_obj(iter_id);
        // println!("{:#?}", v);
        if let ArgType::ForLine(n) = bytecode.get_arg() {
            self.break_line.push(context.ip.0 + *n as usize);
            self.continue_line.push(context.ip.0 + 1);
        }
        self.for_iter_obj.push(iter_id);
        Ok(false)
    }

    // #[inline(always)]
    // fn push_for_next(
    //     self: &mut FSRThreadRuntime<'a>,
    //     context: &mut ThreadContext<'a>,
    //     _bytecode: &BytecodeArg,
    //     _: &'a Bytecode,
    // ) -> Result<bool, FSRError> {
    //     let id = context.exp.last().unwrap().get_global_id(self)?;
    //     if id == 0 {
    //         let break_line = self.break_line.pop().unwrap();
    //         self.continue_line.pop();
    //         context.ip = (break_line, 0);
    //         return Ok(true);
    //     }
    //     Ok(false)
    // }

    #[inline(always)]
    fn while_test_process(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let state = self.get_cur_mut_frame();
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

        if let ArgType::WhileTest(n) = bytecode.get_arg() {
            // Avoid repeat add break ip and continue ip
            if let Some(s) = self.break_line.last() {
                if context.ip.0 + *n as usize + 1 != *s {
                    self.break_line.push(context.ip.0 + *n as usize + 1);
                }
            } else {
                self.break_line.push(context.ip.0 + *n as usize + 1);
            }

            if let Some(s) = self.continue_line.last() {
                if context.ip.0 != *s {
                    self.continue_line.push(context.ip.0);
                }
            } else {
                self.continue_line.push(context.ip.0);
            }
        }

        if (test_val == FSRObject::false_id() || test_val == FSRObject::none_id()) || self.is_break
        {
            self.is_break = false;
            if let ArgType::WhileTest(n) = bytecode.get_arg() {
                context.ip = (context.ip.0 + *n as usize + 1, 0);
                self.break_line.pop();
                self.continue_line.pop();
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
                    _ => panic!("not support args value"),
                };
                args.push(v.1.to_string());
            }

            //println!("define_fn: {}", FSRObject::id_to_obj(context.module.unwrap()).as_module().as_string());
            let fn_obj =
                FSRFn::from_fsr_fn(name.1, (context.ip.0 + 1, 0), args, bc, context.module);

            let fn_obj = self
                .thread_allocator
                .new_object(fn_obj, FSRGlobalObjId::FnCls as ObjId);
            fn_obj.ref_add();
            let fn_id = FSRVM::leak_object(fn_obj);
            let state = self.call_frames.last_mut().unwrap();
            if let Some(cur_cls) = &mut state.cur_cls {
                cur_cls.insert_attr_id(name.1, fn_id);
                context.ip = (context.ip.0 + *n as usize + 2, 0);
                return Ok(true);
            }

            state.insert_var(&name.0, fn_id, Some(&self.thread_allocator), true);
            FSRObject::id_to_obj(context.module)
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
        let cur = self.get_cur_mut_frame();
        context.ip = (cur.reverse_ip.0, cur.reverse_ip.1 + 1);
        context.module = cur.module;
        context.call_end.pop();
        Ok(true)
    }

    #[inline(always)]
    fn compare_test(
        self: &mut FSRThreadRuntime<'a>,
        context: &mut ThreadContext<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        if let ArgType::Compare(op) = bytecode.get_arg() {
            let right = context.exp.last().unwrap().get_global_id(self)?;
            let left = context
                .exp
                .get(context.exp.len() - 2)
                .unwrap()
                .get_global_id(self)?;
            let v = Self::compare(left, right, op, self, context)?;

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
        let v = match context.exp.pop().unwrap() {
            SValue::Stack(s) => {
                let state = self.get_cur_frame();
                if let Some(id) = state.get_var(&s.0) {
                    *id
                } else {
                    let module = FSRObject::id_to_obj(state.module).as_module();
                    let vm = self.get_vm();
                    let v = match module.get_object(s.1) {
                        Some(s) => s,
                        None => *vm.lock().unwrap().get_global_obj_by_name(s.1).unwrap(),
                    };
                    v
                }
            }
            SValue::Global(id) => id,
            SValue::Attr(args) => args.attr,
            SValue::BoxObject(obj) => FSRVM::leak_object(obj),
        };

        self.pop_stack(&[v]);
        let cur = self.get_cur_mut_frame();
        cur.ret_val = Some(SValue::Global(v));
        context.ip = (cur.reverse_ip.0, cur.reverse_ip.1);
        context.module = cur.module;
        context.call_end.pop();
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
        let state = self.call_frames.last_mut().unwrap();
        let v = state.args.pop().unwrap();
        if let ArgType::Variable(s_id, _) = bytecode.get_arg() {
            state.insert_var(s_id, v, Some(&self.thread_allocator), true);
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
                    obj.ref_add();
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
        let state = self.get_cur_mut_frame();
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
        let mut module_path = PathBuf::from_str("modules").unwrap();

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

            let frame = self.get_cur_mut_frame();
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
            let state = self.get_cur_mut_frame();
            let mut cls_obj = FSRObject::new();
            cls_obj.set_cls(FSRGlobalObjId::ClassCls as ObjId);
            let obj = state.cur_cls.take().unwrap();

            let name = obj.get_name().to_string();
            cls_obj.set_value(FSRValue::Class(obj));
            let obj_id = FSRVM::register_object(cls_obj);
            state.insert_var(id, obj_id, None, true);
            FSRObject::id_to_obj(context.module)
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
        // context
        //     .exp
        //     .push(SValue::Global(*self.for_iter_obj.last().unwrap()));
        let obj = self.for_iter_obj.last().cloned().unwrap();

        let res = FSRObject::invoke_offset_method(
            BinaryOffset::NextObject,
            &[obj],
            self,
            context.module,
        )?;

        match res {
            FSRRetValue::Value(object) => {
                context.exp.push(SValue::BoxObject(object));
            }
            FSRRetValue::GlobalId(res_id) => {
                if res_id == 0 || self.is_break {
                    self.is_break = false;
                    let break_line = self.break_line.pop().unwrap();
                    self.continue_line.pop();
                    let obj = self.for_iter_obj.pop().unwrap();
                    let iter_obj = FSRObject::id_to_obj(obj);
                    iter_obj.ref_dec();
                    if iter_obj.count_ref() == 0 {
                        self.thread_allocator.free(obj);
                    }
                    let obj_id = self.ref_for_obj.pop().unwrap();
                    let for_obj = FSRObject::id_to_obj(obj_id);
                    for_obj.ref_dec();
                    if for_obj.count_ref() == 0 {
                        self.thread_allocator.free(obj_id);
                    }
                    context.ip = (break_line, 0);
                    return Ok(true);
                }
                context.exp.push(SValue::Global(res_id));
            }
        }

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
        target = FSRObject::none_id() == v1_id || FSRObject::false_id() == v1_id;

        context.exp.pop();

        if target {
            context.exp.push(SValue::Global(FSRObject::true_id()));
        } else {
            context.exp.push(SValue::Global(FSRObject::false_id()));
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
            // BytecodeOperator::PushForNext => Self::push_for_next(self, context, bytecode, bc),
            BytecodeOperator::ForBlockEnd => Self::for_block_end(self, context, bytecode, bc),
            BytecodeOperator::SpecialLoadFor => Self::special_load_for(self, context, bytecode, bc),
            BytecodeOperator::AndJump => Self::process_logic_and(self, context, bytecode, bc),
            BytecodeOperator::OrJump => Self::process_logic_or(self, context, bytecode, bc),
            BytecodeOperator::Empty => Self::empty_process(self, context, bytecode, bc),
            BytecodeOperator::BinarySub => Self::binary_sub_process(self, context, bytecode, bc),
            BytecodeOperator::Import => {
                Self::process_import(self, &mut context.exp, bytecode, context.module)
            }
            BytecodeOperator::BinaryDiv => Self::binary_div_process(self, context, bytecode, bc),
            BytecodeOperator::NotOperator => Self::not_process(self, context, bytecode, bc),
            BytecodeOperator::BinaryClassGetter => {
                Self::binary_get_cls_attr_process(self, context, bytecode, bc)
            }
            BytecodeOperator::Getter => Self::getter_process(self, context, bytecode, bc),
            BytecodeOperator::Try => Self::try_process(self, context, bytecode),
            BytecodeOperator::EndTry => Self::try_end(self, context, bytecode),
            BytecodeOperator::EndCatch => Self::catch_end(self, context, bytecode),
            BytecodeOperator::BinaryRange => Self::binary_range_process(self, context),
            BytecodeOperator::ForBlockRefAdd => Self::for_block_ref(self, context),
            _ => {
                panic!("not implement for {:#?}", op);
            }
        };

        let v = match v {
            Ok(o) => o,
            Err(e) => {
                if e.code == FSRErrCode::RuntimeError {
                    self.exception = e.exception;
                    return Ok(false);
                }

                return Err(e);
            }
        };

        if v {
            return Ok(v);
        }

        Ok(false)
    }

    #[inline(always)]
    fn load_var(exp_stack: &mut Vec<SValue<'a>>, arg: &'a BytecodeArg, module: ObjId) {
        if let ArgType::Variable(id, name) = arg.get_arg() {
            exp_stack.push(SValue::Stack((*id, name)));
        } else if let ArgType::ConstInteger(c_id, i) = arg.get_arg() {
            //let int_const = Self::load_integer_const(i, vm);
            let m = module;
            let obj = FSRObject::id_to_obj(m);
            let m = obj.as_module();
            if let Some(id) = m.get_bytecode().const_table.table.get(*c_id as usize) {
                if id != &0 {
                    exp_stack.push(SValue::Global(*id));
                    return;
                }
            } else {
                panic!("not found integer const")
            }
        } else if let ArgType::ConstFloat(c_id, f) = arg.get_arg() {
            //let float_const = Self::load_float_const(f, vm);
            let m = module;
            let m = FSRObject::id_to_obj(m).as_module();
            if let Some(id) = m.get_bytecode().const_table.table.get(*c_id as usize) {
                if id != &0 {
                    exp_stack.push(SValue::Global(*id));
                    return;
                }
            } else {
                panic!("not found float const")
            }
        } else if let ArgType::ConstString(c_id, i) = arg.get_arg() {
            // let string_const = Self::load_string_const(i.clone(), vm);
            // s.insert_const(id, string_const);
            let m = module;
            let m = FSRObject::id_to_obj(m).as_module();
            if let Some(id) = m.get_bytecode().const_table.table.get(*c_id as usize) {
                if id != &0 {
                    exp_stack.push(SValue::Global(*id));
                    return;
                }
            } else {
                panic!("not found str const")
            }
        } else if let ArgType::Attr(_, name) = arg.get_arg() {
            exp_stack.push(SValue::Attr(AttrArgs::new(0, 0, name, true)));
        }
    }

    fn restore_exp_stack(&mut self, exp_stack: &mut Vec<SValue<'a>>) {}

    #[inline(always)]
    fn set_exp_stack_ret(&mut self, exp_stack: &mut Vec<SValue<'a>>) {
        let state = self.get_cur_mut_frame();
        if state.exp.is_some() {
            if let Some(mut s) = state.exp.take() {
                // std::mem::replace(exp_stack, s);
                *exp_stack = s;
            }
        }

        // if take a none value, it seems a little slow, so check it first
        if state.ret_val.is_none() {
            return;
        }

        if let Some(s) = state.ret_val.take() {
            exp_stack.push(s);
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

            match arg.get_operator() {
                BytecodeOperator::Load => {
                    Self::load_var(&mut context.exp, arg, context.module);
                }
                _ => {
                    v = self.process(context, arg, bc)?;
                    if self.get_cur_frame().ret_val.is_some() {
                        return Ok(true);
                    }

                    if v {
                        context.clear_exp(&self.thread_allocator);
                        return Ok(false);
                    }
                }
            }

            if self.exception.is_some() {
                if !self.get_cur_mut_frame().catch_ends.is_empty() {
                    self.get_cur_mut_frame().handling_exception = self.exception.take();
                    context.ip = (self.get_cur_mut_frame().catch_ends.pop().unwrap().0, 0);
                    return Ok(true);
                } else {
                    if self.call_frames.len() == 1 {
                        panic!("No handle of error")
                    }
                    self.pop_stack(&[]);
                    let cur = self.get_cur_mut_frame();
                    context.ip = (cur.reverse_ip.0, cur.reverse_ip.1 + 1);
                    context.module = cur.module;
                    context.call_end.pop();
                    return Ok(true);
                }
            }
        }

        context.ip.0 += 1;
        context.ip.1 = 0;
        context.clear_exp(&self.thread_allocator);
        context.is_attr = false;
        Ok(false)
    }

    pub fn load(&mut self, module: Box<FSRObject<'a>>) -> Result<ObjId, FSRError> {
        let mut bytecode_count = 0;
        let module_id = FSRVM::leak_object(module);
        let mut context = ThreadContext {
            exp: Vec::with_capacity(10),
            ip: (0, 0),
            is_attr: false,
            module: module_id,
            call_end: vec![()],
        };

        self.call_frames
            .push(self.frame_free_list.new_frame("load_module", module_id));

        let module = FSRObject::id_to_obj(module_id).as_module();
        while let Some(expr) = module.get_expr(&context.ip) {
            self.run_expr(expr, &mut context, module.get_bytecode())?;
            bytecode_count += expr.len();
        }

        self.call_frames.pop();

        Ok(module_id)
    }

    pub fn start(&mut self, module_id: ObjId) -> Result<(), FSRError> {
        let mut bytecode_count = 0;

        let mut context = ThreadContext {
            exp: Vec::with_capacity(10),
            ip: (0, 0),
            is_attr: false,
            module: module_id,
            call_end: vec![()],
        };

        let mut module = FSRObject::id_to_obj(module_id).as_module();
        while let Some(expr) = FSRObject::id_to_obj(context.module)
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
            module = FSRObject::id_to_obj(context.module).as_module();
        }

        println!("count: {}", bytecode_count);

        #[cfg(feature = "alloc_trace")]
        println!(
            "reused count: {}",
            crate::backend::types::base::HEAP_TRACE.object_count()
        );
        Ok(())
    }

    pub fn run_with_context(
        &mut self,
        module_id: ObjId,
        context: &mut ThreadContext<'a>,
    ) -> Result<(), FSRError> {
        let mut module = FSRObject::id_to_obj(module_id).as_module();
        while let Some(expr) = FSRObject::id_to_obj(context.module)
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
            self.run_expr(expr, context, module.get_bytecode())?;
            module = FSRObject::id_to_obj(context.module).as_module();
        }

        #[cfg(feature = "alloc_trace")]
        println!(
            "reused count: {}",
            crate::backend::types::base::HEAP_TRACE.object_count()
        );
        context.ip = (0, 0);
        Ok(())
    }

    pub fn call_fn(
        &mut self,
        fn_def: &'a FSRFnInner,
        args: &[ObjId],
        module: ObjId,
    ) -> Result<SValue, FSRError> {
        let mut context = ThreadContext {
            exp: Vec::with_capacity(10),
            ip: fn_def.get_ip(),
            is_attr: false,
            module,
            call_end: vec![()],
        };
        {
            //self.save_ip_to_callstate(args.len(), &mut context.exp, &mut args, &mut context.ip);
            // self.call_frames
            //     .push(self.frame_free_list.new_frame(fn_def.get_name(), module));
            context.exp.clear();

            for arg in args.iter().rev() {
                self.get_cur_mut_frame().args.push(*arg);
            }
            //let offset = fn_obj.get_fsr_offset();
            let offset = fn_def.get_ip();
            context.ip = (offset.0, 0);
        }

        while let Some(expr) = fn_def.get_bytecode().get(&context.ip) {
            let v = self.run_expr(expr, &mut context, fn_def.get_bytecode())?;

            if self.exception.is_some() {
                // If this is last function call, in this call_fn
                if context.call_end.is_empty() {
                    return Err(FSRError::new_runtime_error(self.exception.unwrap()));
                }
            }

            if context.call_end.is_empty() {
                break;
            }

            if v {
                break;
            }
        }

        let cur = self.get_cur_mut_frame();
        if cur.ret_val.is_none() {
            return Ok(SValue::Global(0));
        }
        let ret_val = cur.ret_val.take();
        // self.call_frames.pop();
        // let v = FSRObject::id_to_obj(s);
        // println!("{:#?}", v);
        match ret_val {
            Some(s) => Ok(s),
            None => Ok(SValue::Global(0)),
        }
    }

    pub fn call_fn_with_context(
        &mut self,
        fn_def: &'a FSRFnInner,
        args: &[ObjId],
        module: ObjId,
        context: &mut ThreadContext<'a>,
    ) -> Result<SValue, FSRError> {
        {
            //self.save_ip_to_callstate(args.len(), &mut context.exp, &mut args, &mut context.ip);
            // self.call_frames
            //     .push(self.frame_free_list.new_frame(fn_def.get_name(), module));
            context.exp.clear();

            for arg in args.iter().rev() {
                self.get_cur_mut_frame().args.push(*arg);
            }
            //let offset = fn_obj.get_fsr_offset();
            let offset = fn_def.get_ip();
            context.ip = (offset.0, 0);
        }

        while let Some(expr) = fn_def.get_bytecode().get(&context.ip) {
            let v = self.run_expr(expr, context, fn_def.get_bytecode())?;

            if self.exception.is_some() {
                // If this is last function call, in this call_fn
                if context.call_end.is_empty() {
                    return Err(FSRError::new_runtime_error(self.exception.unwrap()));
                }
            }

            if context.call_end.is_empty() {
                break;
            }

            if v {
                break;
            }
        }

        let cur = self.get_cur_mut_frame();
        if cur.ret_val.is_none() {
            return Ok(SValue::Global(0));
        }
        let ret_val = cur.ret_val.take();
        // self.call_frames.pop();
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
        vm::virtual_machine::FSRVM,
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

        b = [[1,2,3]]
        c = b[0][0]
        println(c)
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

    #[test]
    fn test_try_catch_success() {
        let source_code = r#"
        try {
            a = 1 == 1
        } catch {
            println("catch")
        }

        println('ok')
        "#;
        let v = FSRModule::from_code("main", source_code).unwrap();
        let base_module = FSRVM::leak_object(Box::new(v));
        let mut vm = Arc::new(Mutex::new(FSRVM::new()));
        let mut runtime = FSRThreadRuntime::new(base_module, vm);
        runtime.start(base_module).unwrap();
    }

    #[test]
    fn test_try_catch_failed() {
        let source_code = r#"
        try {
            a = 1 == 1
            throw_error(1)
            println('if not error will print this text')
        } catch {
            e = get_error()
            println(e)
            println("catch")
        }

        println('ok')
        "#;
        let v = FSRModule::from_code("main", source_code).unwrap();
        let base_module = FSRVM::leak_object(Box::new(v));
        let mut vm = Arc::new(Mutex::new(FSRVM::new()));
        let mut runtime = FSRThreadRuntime::new(base_module, vm);
        runtime.start(base_module).unwrap();
    }

    #[test]
    fn test_try_catch_failed2() {
        let source_code = r#"
        fn abc() {
            throw_error(1)
            println('in abc')
        }
        try {
            a = 1 == 1
            abc()
            println('if not error will print')
        } catch {
            e = get_error()
            println(e)
            println("catch")
        }

        println('ok')
        "#;
        let v = FSRModule::from_code("main", source_code).unwrap();
        let base_module = FSRVM::leak_object(Box::new(v));
        let mut vm = Arc::new(Mutex::new(FSRVM::new()));
        let mut runtime = FSRThreadRuntime::new(base_module, vm);
        runtime.start(base_module).unwrap();
    }

    #[test]
    fn test_try_catch_failed3() {
        let source_code = r#"
        fn abc() {
            try {
                throw_error(1)
            } catch {
            }
            
            println('in abc')
        }
        try {
            a = 1 == 1
            abc()
            println('if not error will print')
        } catch {
            e = get_error()
            println(e)
            println("catch")
        }

        println('ok')
        "#;
        let v = FSRModule::from_code("main", source_code).unwrap();
        let base_module = FSRVM::leak_object(Box::new(v));
        let mut vm = Arc::new(Mutex::new(FSRVM::new()));
        let mut runtime = FSRThreadRuntime::new(base_module, vm);
        runtime.start(base_module).unwrap();
    }

    #[test]
    fn test_try() {
        let code = "try { a = 1 + 1 }";
    }

    #[test]
    fn test_range() {
        let range = r#"
        for i in 0..4 {
            println(i)
        }
        "#;
        let v = FSRModule::from_code("main", range).unwrap();
        let base_module = FSRVM::leak_object(Box::new(v));
        let mut vm = Arc::new(Mutex::new(FSRVM::new()));
        let mut runtime = FSRThreadRuntime::new(base_module, vm);
        runtime.start(base_module).unwrap();
    }

    #[test]
    fn size_of_object() {
        println!("size of object: {}", std::mem::size_of::<FSRObject>());
        println!("size of svalue: {}", std::mem::size_of::<super::SValue>());
    }
}
