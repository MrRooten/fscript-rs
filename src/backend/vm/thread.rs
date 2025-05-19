#![allow(clippy::ptr_arg)]

use std::{
    collections::HashSet,
    ops::Range,
    path::PathBuf,
    str::FromStr,
    sync::{atomic::Ordering, Arc, Condvar, Mutex},
};

use smallvec::SmallVec;

use crate::{
    backend::{
        compiler::bytecode::{
            ArgType, BinaryOffset, Bytecode, BytecodeArg, BytecodeOperator, CompareOperator,
        },
        memory::{
            gc::mark_sweep::MarkSweepGarbageCollector, size_alloc::FSRObjectAllocator,
            GarbageCollector,
        },
        types::{
            base::{Area, AtomicObjId, FSRGlobalObjId, FSRObject, FSRRetValue, FSRValue, ObjId},
            class::FSRClass,
            class_inst::FSRClassInst,
            code::FSRCode,
            float::FSRFloat,
            fn_def::{FSRFn, FSRFnInner},
            integer::FSRInteger,
            list::FSRList,
            module::FSRModule,
            range::FSRRange,
            string::FSRString,
        },
    },
    utils::error::{FSRErrCode, FSRError},
};

use super::{
    free_list::FrameFreeList,
    quick_op::Ops,
    virtual_machine::{get_object_by_global_id, FSRVM, VM},
};

macro_rules! obj_cls {
    ($a:expr) => {
        FSRObject::id_to_obj(FSRObject::id_to_obj($a).cls).as_class()
    };
}

#[derive(Debug)]
pub struct IndexMap {
    vs: Vec<Option<AtomicObjId>>,
}

pub struct IndexIterator<'a> {
    vs: core::slice::Iter<'a, Option<AtomicObjId>>,
}

#[allow(clippy::new_without_default)]
#[allow(unused)]
impl IndexMap {
    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn get(&self, i: &u64) -> Option<&AtomicObjId> {
        match self.vs.get(*i as usize) {
            Some(Some(s)) => Some(s),
            Some(None) => None,
            None => None,
        }
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn insert(&mut self, i: u64, v: ObjId) {
        if i as usize >= self.vs.len() {
            let new_capacity = (i + 1) + (4 - (i + 1) % 4);
            self.vs.resize_with(new_capacity as usize, || None);
        }

        if let Some(Some(s)) = self.vs.get(i as usize) {
            s.store(v, Ordering::Relaxed);
            return;
        }
        self.vs[i as usize] = Some(AtomicObjId::new(v));
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn contains_key(&self, i: &u64) -> bool {
        if self.vs.get(*i as usize).is_none() {
            return false;
        }

        self.vs[*i as usize].is_some()
    }

    pub fn new() -> Self {
        Self { vs: vec![] }
    }

    pub fn iter(&self) -> IndexIterator {
        IndexIterator { vs: self.vs.iter() }
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn clear(&mut self) {
        self.vs.fill_with(|| None);
    }
}

impl<'a> Iterator for IndexIterator<'a> {
    type Item = &'a AtomicObjId;

    fn next(&mut self) -> Option<Self::Item> {
        for s in self.vs.by_ref() {
            if s.is_some() {
                return Some(s.as_ref().unwrap());
            }
        }

        None
    }
}

struct AttrMap<'a> {
    attr_map: Vec<Vec<Option<&'a AtomicObjId>>>,
}

impl<'a> AttrMap<'a> {
    pub fn new() -> Self {
        Self {
            attr_map: vec![vec![None; 4]; 4],
        }
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn insert(&mut self, i: usize, j: usize, v: Option<&'a AtomicObjId>) {
        if i >= self.attr_map.len() {
            let new_capacity = (i + 1) + (4 - (i + 1) % 4);
            self.attr_map.resize(new_capacity, vec![None; 4]);
        }
        if j >= self.attr_map[i].len() {
            let new_capacity = (j + 1) + (4 - (j + 1) % 4);
            self.attr_map[i].resize(new_capacity, None);
        }
        self.attr_map[i][j] = v;
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn clear(&mut self) {
        self.attr_map.clear();
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn clear_var(&mut self, i: usize) {
        if i < self.attr_map.len() {
            self.attr_map[i].clear();
        }
    }

    pub fn get_attr(&self, i: usize, j: usize) -> Option<&'a AtomicObjId> {
        if i < self.attr_map.len() && j < self.attr_map[i].len() {
            return self.attr_map[i][j];
        }
        None
    }
}

pub struct CallFrame<'a> {
    pub(crate) var_map: IndexMap,
    attr_map: AttrMap<'a>,
    reverse_ip: (usize, usize),
    args: SmallVec<[ObjId; 4]>,
    cur_cls: Option<Box<FSRClass<'a>>>,
    pub(crate) ret_val: Option<ObjId>,
    pub(crate) exp: Vec<ObjId>,
    /// in case of garbage collection collecting this object, this object is for middle value for expression
    pub(crate) middle_value: Vec<ObjId>,
    pub(crate) code: ObjId,
    catch_ends: Vec<(usize, usize)>,
    pub(crate) handling_exception: ObjId,
    // Record current call fn_obj
    pub(crate) fn_obj: ObjId,
}

impl<'a> CallFrame<'a> {
    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn clear(&mut self) {
        self.var_map.clear();
        self.args.clear();
        self.cur_cls = None;
        self.ret_val = None;
        self.exp.clear();
        self.attr_map.clear();
        self.handling_exception = FSRObject::none_id();
        self.middle_value.clear();
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn get_var(&self, id: &u64) -> Option<&AtomicObjId> {
        self.var_map.get(id)
    }

    pub fn get_attr(&self, i: usize, j: usize) -> Option<&'a AtomicObjId> {
        self.attr_map.get_attr(i, j)
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn insert_var(&mut self, id: u64, obj_id: ObjId) {
        self.var_map.insert(id, obj_id);
        self.attr_map.clear_var(id as usize);
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn has_var(&self, id: &u64) -> bool {
        self.var_map.contains_key(id)
    }

    pub fn set_reverse_ip(&mut self, ip: (usize, usize)) {
        self.reverse_ip = ip;
    }

    pub fn new(code: ObjId, fn_obj: ObjId) -> Self {
        Self {
            var_map: IndexMap::new(),
            reverse_ip: (0, 0),
            args: SmallVec::new(),
            cur_cls: None,
            ret_val: None,
            exp: vec![],
            code,
            catch_ends: vec![],
            handling_exception: FSRObject::none_id(),
            fn_obj,
            attr_map: AttrMap::new(),
            middle_value: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReferenceArgs<'a> {
    pub(crate) father: ObjId,
    pub(crate) atomic_usize: &'a AtomicObjId,
    pub(crate) call_method: bool,
}

/// Context for bytecode, if fs code call from rust fn will create new context
pub struct FSCodeContext {
    // tracing call stack, is call stack is empty means end of this call except start of this call
    pub(crate) call_end: u32,
    ip: (usize, usize),
    pub(crate) code: ObjId,
}

impl FSCodeContext {
    pub fn new_context(code: ObjId) -> Self {
        FSCodeContext {
            // exp: Vec::with_capacity(8),
            ip: (0, 0),
            code,
            call_end: 1,
        }
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn clear_exp(&mut self) {
        // if self.exp.is_empty() {
        //     return;
        // }

        // self.exp.clear();
    }

    pub fn clear(&mut self) {
        //self.exp.clear();
        self.ip = (0, 0);
    }
}

#[derive(Debug, Default)]
pub struct FlowTracker {
    pub last_if_test: Vec<bool>,
    /// jump out of loop
    pub break_line: Vec<usize>,
    /// jump to next loop
    pub continue_line: Vec<usize>,

    pub ref_for_obj: Vec<ObjId>,

    pub is_break: bool,

    pub for_iter_obj: Vec<ObjId>,
}

impl FlowTracker {
    pub fn new() -> Self {
        Self {
            last_if_test: Vec::new(),
            break_line: Vec::new(),
            continue_line: Vec::new(),
            ref_for_obj: Vec::new(),
            for_iter_obj: Vec::new(),
            is_break: false,
        }
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn false_last_if_test(&mut self) {
        let l = self.last_if_test.len() - 1;
        self.last_if_test[l] = false;
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn true_last_if_test(&mut self) {
        let l = self.last_if_test.len() - 1;
        self.last_if_test[l] = true;
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn peek_last_if_test(&self) -> bool {
        if self.last_if_test.is_empty() {
            return false;
        }

        self.last_if_test[self.last_if_test.len() - 1]
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn push_last_if_test(&mut self, test: bool) {
        self.last_if_test.push(test)
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn pop_last_if_test(&mut self) {
        self.last_if_test.pop();
    }
}

pub struct ThreadLockerState {
    pub(crate) in_rt_cxt: Arc<(Mutex<bool>, Condvar)>,
    pub(crate) is_stop: Arc<(Mutex<bool>, Condvar)>,
}

impl ThreadLockerState {
    pub fn new_state() -> Self {
        Self {
            in_rt_cxt: Arc::new((Mutex::new(true), Condvar::new())),
            is_stop: Arc::new((Mutex::new(true), Condvar::new())),
        }
    }
}

#[derive(PartialEq)]
pub enum GcState {
    Running,
    Stop,
}

pub struct GcContext {
    worklist: Vec<ObjId>,
    gc_state: GcState,
}

impl GcContext {
    pub fn new_context() -> Self {
        Self {
            worklist: Vec::new(),
            gc_state: GcState::Stop,
        }
    }
}

#[allow(clippy::vec_box)]
pub struct FSRThreadRuntime<'a> {
    pub(crate) thread_id: usize,
    /// cur call frame, save for quick access
    pub(crate) cur_frame: Box<CallFrame<'a>>,
    pub(crate) call_frames: Vec<Box<CallFrame<'a>>>,
    pub(crate) frame_free_list: FrameFreeList<'a>,
    pub(crate) thread_allocator: FSRObjectAllocator<'a>,
    pub(crate) flow_tracker: FlowTracker,
    pub(crate) exception: ObjId,
    pub(crate) exception_flag: bool,
    pub(crate) garbage_collect: MarkSweepGarbageCollector<'a>,
    // pub(crate) op_quick: Box<Ops>,
    pub(crate) counter: usize,
    pub(crate) last_aquire_counter: usize,
    pub(crate) til: ThreadLockerState,
    pub(crate) thread_context_stack: Vec<Box<FSCodeContext>>,
    pub(crate) thread_context: Option<Box<FSCodeContext>>,
    pub(crate) gc_context: GcContext,
    #[cfg(feature = "count_bytecode")]
    pub(crate) bytecode_counter: Vec<usize>,
}

impl<'a> FSRThreadRuntime<'a> {
    #[cfg(feature = "count_bytecode")]
    fn dump_bytecode_counter(&self) {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        for (i, count) in self.bytecode_counter.iter().enumerate() {
            if *count > 0 {
                let op = match BytecodeOperator::from_u8(i as u8) {
                    Some(op) => op,
                    None => {
                        continue;
                    }
                };
                map.insert(op, *count);
            }
        }

        println!("bytecode counter: {:?}", map);
    }

    pub fn get_vm(&self) -> Arc<FSRVM<'static>> {
        unsafe { VM.as_ref().unwrap().clone() }
    }

    pub fn new_runtime() -> FSRThreadRuntime<'a> {
        let frame = Box::new(CallFrame::new(0, 0));
        Self {
            cur_frame: frame,
            call_frames: vec![],
            frame_free_list: FrameFreeList::new_list(),
            thread_allocator: FSRObjectAllocator::new(),
            flow_tracker: FlowTracker::new(),
            exception: FSRObject::none_id(),
            exception_flag: false,
            garbage_collect: MarkSweepGarbageCollector::new_gc(),
            thread_id: 0,
            // op_quick: Box::new(Ops::new_init()),
            counter: 0,
            til: ThreadLockerState::new_state(),
            last_aquire_counter: 0,
            thread_context_stack: Vec::with_capacity(8),
            thread_context: None,
            gc_context: GcContext::new_context(),
            #[cfg(feature = "count_bytecode")]
            bytecode_counter: vec![0; 256],
        }
    }

    pub fn clear_marks(&mut self) {
        self.garbage_collect.clear_marks();
    }

    pub fn get_thread_id(&self) -> usize {
        self.thread_id
    }

    pub fn call_stack(&self) -> Vec<ObjId> {
        let mut fns = self
            .call_frames
            .iter()
            .map(|x| x.fn_obj)
            .collect::<Vec<_>>();
        fns.push(self.get_cur_frame().fn_obj);
        fns
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn get_cur_mut_frame(&mut self) -> &mut CallFrame<'a> {
        &mut self.cur_frame
    }

    /// Push new call frame to call stack, and replace current call frame with new one
    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn push_frame(&mut self, frame: Box<CallFrame<'a>>) {
        let old_frame = std::mem::replace(&mut self.cur_frame, frame);
        self.call_frames.push(old_frame);
    }

    /// Pop current call frame and replace with the last one
    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn pop_frame(&mut self) -> Box<CallFrame<'a>> {
        let v = self.call_frames.pop().unwrap();
        std::mem::replace(&mut self.cur_frame, v)
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn get_cur_mut_context(&mut self) -> &mut FSCodeContext {
        self.thread_context.as_mut().unwrap()
    }

    pub fn push_context(&mut self, context: Box<FSCodeContext>) {
        if self.thread_context.is_none() {
            self.thread_context = Some(context);
        } else if let Some(s) = self.thread_context.take() {
            self.thread_context = Some(context);
            self.thread_context_stack.push(s);
        }
    }

    pub fn pop_context(&mut self) -> Box<FSCodeContext> {
        if let Some(s) = self.thread_context.take() {
            // self.thread_context_stack.push(s);
            self.thread_context = self.thread_context_stack.pop();
            return s;
        }
        panic!("pop empty context");
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn get_context(&self) -> &FSCodeContext {
        self.thread_context.as_ref().unwrap()
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn get_cur_frame(&self) -> &CallFrame<'a> {
        &self.cur_frame
    }

    fn process_callframe(&self, work_list: &mut Vec<ObjId>, it: &CallFrame<'a>) {
        for obj in it.var_map.iter() {
            work_list.push(obj.load(Ordering::Relaxed));
        }

        for id in &it.exp {
            work_list.push(*id);
        }

        if let Some(ret_val) = it.ret_val {
            work_list.push(ret_val);
        }

        if it.handling_exception != 0 {
            work_list.push(it.handling_exception);
        }

        for value in &it.middle_value {
            work_list.push(*value);
        }
    }

    /// Add all objects in current call frame to worklist, wait to gc to reference
    fn add_worklist(&self) -> Vec<ObjId> {
        let mut others = self.flow_tracker.for_iter_obj.clone();
        others.extend(self.flow_tracker.ref_for_obj.clone());
        let frames = &self.call_frames;
        let cur_frame = self.get_cur_frame();
        let mut work_list = Vec::with_capacity(16);
        for it in frames {
            self.process_callframe(&mut work_list, it);
        }

        let it = cur_frame;
        self.process_callframe(&mut work_list, it);

        for obj in others {
            work_list.push(obj);
        }

        work_list
    }

    fn process_refs(&mut self, id: ObjId, obj: &FSRObject, full: bool) {
        let work_list = &mut self.gc_context.worklist;
        let mut is_add = false;
        let refs = obj.get_references(full, work_list, &mut is_add);

        for ref_id in refs {
            let obj = FSRObject::id_to_obj(ref_id);
            if obj.area == Area::Minjor {
                is_add = true;
            } else if !full {
                continue;
            }

            if !obj.is_marked() {
                work_list.push(ref_id);
            }
        }

        if !is_add && obj.get_write_barrier() {
            obj.set_write_barrier(false);
        }
    }

    pub fn set_ref_objects_mark(&mut self, full: bool) {
        if self.gc_context.gc_state == GcState::Stop {
            self.gc_context.worklist = self.add_worklist();
        }
        self.gc_context.gc_state = GcState::Running;

        while let Some(id) = self.gc_context.worklist.pop() {
            if FSRObject::is_sp_object(id) {
                continue;
            }

            let obj = FSRObject::id_to_obj(id);
            if obj.is_marked() {
                continue;
            }

            obj.mark();

            if !full && obj.area.is_long() && !obj.get_write_barrier() {
                continue;
            }

            self.process_refs(id, obj, full);
        }
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn compare(
        left: ObjId,
        right: ObjId,
        op: CompareOperator,
        thread: &mut Self,
    ) -> Result<bool, FSRError> {
        let res = match op {
            CompareOperator::Equal => {

                if let Some(rust_fn) = obj_cls!(left)
                    .get_rust_fn(BinaryOffset::Equal)
                {
                    rust_fn(
                        &[left, right],
                        thread,
                        thread.get_context().code,
                    )?
                } else {
                    FSRObject::invoke_offset_method(
                        BinaryOffset::Equal,
                        &[left, right],
                        thread,
                        thread.get_context().code,
                    )?
                }
            }
            CompareOperator::Greater => {
                if let Some(rust_fn) = obj_cls!(left)
                    .get_rust_fn(BinaryOffset::Greater)
                {
                    rust_fn(
                        &[left, right],
                        thread,
                        thread.get_context().code,
                    )?
                } else {
                    FSRObject::invoke_offset_method(
                        BinaryOffset::Greater,
                        &[left, right],
                        thread,
                        thread.get_context().code,
                    )?
                }
            }
            CompareOperator::Less => {
                if let Some(rust_fn) = obj_cls!(left)
                    .get_rust_fn(BinaryOffset::Less)
                {
                    rust_fn(
                        &[left, right],
                        thread,
                        thread.get_context().code,
                    )?
                } else {
                    FSRObject::invoke_offset_method(
                        BinaryOffset::Less,
                        &[left, right],
                        thread,
                        thread.get_context().code,
                    )?
                }
            }
            CompareOperator::GreaterEqual => FSRObject::invoke_offset_method(
                BinaryOffset::GreatEqual,
                &[left, right],
                thread,
                thread.get_context().code,
            )?,
            CompareOperator::LessEqual => FSRObject::invoke_offset_method(
                BinaryOffset::LessEqual,
                &[left, right],
                thread,
                thread.get_context().code,
            )?,

            CompareOperator::NotEqual => FSRObject::invoke_offset_method(
                BinaryOffset::NotEqual,
                &[left, right],
                thread,
                thread.get_context().code,
            )?,
            _ => {
                return Err(FSRError::new(
                    format!("not support op: `{:?}`", op),
                    FSRErrCode::NotSupportOperator,
                ));
            }
        };
        // if let FSRRetValue::GlobalId(id) = &res {
        //     return Ok(id == &1);
        // }

        let id = res.get_id();
        Ok(id == FSRObject::true_id())
    }

    fn pop_stack(&mut self) {
        let v = self.pop_frame();
        self.frame_free_list.free(v);
    }

    fn getter_process(
        self: &mut FSRThreadRuntime<'a>,
        _bytecode: &BytecodeArg,
    ) -> Result<bool, FSRError> {
        let index = self.get_cur_mut_frame().exp.pop().unwrap();

        let container = self.get_cur_mut_frame().exp.pop().unwrap();

        let index_obj_v = FSRObject::id_to_obj(index);
        let list_obj_v = FSRObject::id_to_obj(container);

        // let res = if let Some(get_item) = self
        //     .op_quick
        //     .get_getter(list_obj_v.cls as ObjId, index_obj_v.cls as ObjId)
        // {
        //     get_item(&[container, index], self, self.get_context().code)?
        // } else {
        let res = FSRObject::invoke_offset_method(
            BinaryOffset::GetItem,
            &[container, index],
            self,
            self.get_context().code,
        )?;
        //};

        // pop after finish invoke

        self.get_cur_mut_frame().middle_value.push(container);
        self.get_cur_mut_frame().middle_value.push(index);

        match res {
            FSRRetValue::GlobalId(res_id) => {
                self.get_cur_mut_frame().exp.push(res_id);
            }
        };

        Ok(false)
    }

    // like a[0] = 1
    #[cfg_attr(feature = "more_inline", inline(always))]
    fn getter_assign_process(
        self: &mut FSRThreadRuntime<'a>,
        _bytecode: &'a BytecodeArg,
    ) -> Result<bool, FSRError> {
        let len = self.get_cur_frame().exp.len();
        let index_obj = *self.get_cur_frame().exp.last().unwrap();
        let container_obj = *self.get_cur_frame().exp.get(len - 2).unwrap();
        let value_obj = *self.get_cur_frame().exp.get(len - 3).unwrap();

        let containter_obj_v = FSRObject::id_to_obj(container_obj);
        let index_obj_v = FSRObject::id_to_obj(index_obj);

        if containter_obj_v.area.is_long() && index_obj_v.area == Area::Minjor {
            containter_obj_v.set_write_barrier(true);
        }

        // if let Some(set_item) = self
        //     .op_quick
        //     .get_set_item(containter_obj_v.cls, index_obj_v.cls)
        // {
        //     let res = set_item(
        //         &[container_obj, index_obj, value_obj],
        //         self,
        //         self.get_context().code,
        //     )?;
        //     return Ok(false);
        // }
        let set_item = FSRObject::id_to_obj(container_obj)
            .get_cls_offset_attr(BinaryOffset::SetItem)
            .unwrap()
            .load(Ordering::Relaxed);

        let set_item_fn = FSRObject::id_to_obj(set_item);
        let _res = set_item_fn.call(
            &[container_obj, index_obj, value_obj],
            self,
            self.get_context().code,
            set_item,
        );

        Ok(false)
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn attr_assign_process(
        self: &mut FSRThreadRuntime<'a>,
        bytecode: &'a BytecodeArg,
    ) -> Result<bool, FSRError> {
        if let ArgType::Attr(attr_id, name) = bytecode.get_arg() {
            let len = self.get_cur_frame().exp.len();
            let father = *self.get_cur_frame().exp.last().unwrap();
            let assign_value = *self.get_cur_frame().exp.get(len - 2).unwrap();

            let father_obj = FSRObject::id_to_mut_obj(father).unwrap();
            if father_obj.area.is_long() && FSRObject::id_to_obj(assign_value).area == Area::Minjor
            {
                father_obj.set_write_barrier(true);
            }

            father_obj.set_attr(name, assign_value);

            return Ok(false);
        }
        unimplemented!()
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn assign_process(
        self: &mut FSRThreadRuntime<'a>,
        bytecode: &'a BytecodeArg,
    ) -> Result<bool, FSRError> {
        if let ArgType::Variable(v) = bytecode.get_arg() {
            let var_id = v.0;
            let obj_id = match self.get_cur_mut_frame().exp.pop() {
                Some(s) => s,
                None => {
                    return Err(FSRError::new("", FSRErrCode::EmptyExpStack));
                }
            };

            let state = &mut self.cur_frame;
            state.insert_var(var_id, obj_id);

            state.middle_value.push(obj_id);
            state.attr_map.clear_var(var_id as usize);
            return Ok(false);
        }

        if let ArgType::ClosureVar(v) = bytecode.get_arg() {
            self.load_closure(v)?;
            return Ok(false);
        }

        //Assign variable name
        let assign_id = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new("", FSRErrCode::EmptyExpStack));
            }
        };

        let to_assign_obj_id = self.get_cur_mut_frame().exp.pop().unwrap();

        self.get_cur_mut_frame().middle_value.push(to_assign_obj_id);
        // self.get_cur_mut_frame().middle_value.push(assign_id);

        Ok(false)
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn binary_add_process(self: &mut FSRThreadRuntime<'a>) -> Result<bool, FSRError> {
        let right = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary add 1",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let left = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary add 2",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        if let Some(rust_fn) = obj_cls!(left).get_rust_fn(BinaryOffset::Add) {
            let res = rust_fn(&[left, right], self, self.get_context().code)?;

            match res {
                FSRRetValue::GlobalId(res_id) => {
                    self.get_cur_mut_frame().exp.push(res_id);
                }
            };

            self.get_cur_mut_frame().middle_value.push(left);
            self.get_cur_mut_frame().middle_value.push(right);

            return Ok(false);
        }

        let res = FSRObject::invoke_offset_method(
            BinaryOffset::Add,
            &[left, right],
            self,
            self.get_context().code,
        )?;

        // self.get_cur_mut_frame().middle_value.push(v1_id);
        // self.get_cur_mut_frame().middle_value.push(v2_id);

        match res {
            FSRRetValue::GlobalId(res_id) => {
                self.get_cur_mut_frame().exp.push(res_id);
            } // FSRRetValue::Reference(_) => {
              //     panic!("not support reference return, in add process")
              // }
        };

        Ok(false)
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn binary_sub_process(
        self: &mut FSRThreadRuntime<'a>,
        _bytecode: &BytecodeArg,
    ) -> Result<bool, FSRError> {
        let right = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary sub 1",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let left = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary sub 2",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let res = FSRObject::invoke_offset_method(
            BinaryOffset::Sub,
            &[left, right],
            self,
            self.get_context().code,
        )?;

        self.get_cur_mut_frame().middle_value.push(right);
        self.get_cur_mut_frame().middle_value.push(left);
        match res {
            FSRRetValue::GlobalId(res_id) => {
                self.get_cur_mut_frame().exp.push(res_id);
            } // FSRRetValue::Reference(_) => {
              //     panic!("not support reference return, in sub process")
              // }
        };

        Ok(false)
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn binary_mul_process(
        self: &mut FSRThreadRuntime<'a>,
        _bytecode: &BytecodeArg,
    ) -> Result<bool, FSRError> {
        let right_id = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary mul 1",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let left_id = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary mul 2",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let res = FSRObject::invoke_offset_method(
            BinaryOffset::Mul,
            &[left_id, right_id],
            self,
            self.get_context().code,
        )?;

        self.get_cur_mut_frame().middle_value.push(right_id);
        self.get_cur_mut_frame().middle_value.push(left_id);
        match res {
            FSRRetValue::GlobalId(res_id) => {
                self.get_cur_mut_frame().exp.push(res_id);
            }
        };
        Ok(false)
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn binary_div_process(
        self: &mut FSRThreadRuntime<'a>,
        _bytecode: &BytecodeArg,
    ) -> Result<bool, FSRError> {
        let right_id = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary mul 1",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let left_id = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary mul 2",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let res = FSRObject::invoke_offset_method(
            BinaryOffset::Div,
            &[left_id, right_id],
            self,
            self.get_context().code,
        )?;
        match res {
            FSRRetValue::GlobalId(res_id) => {
                self.get_cur_mut_frame().exp.push(res_id);
            }
        };

        self.get_cur_mut_frame().middle_value.push(right_id);
        self.get_cur_mut_frame().middle_value.push(left_id);
        Ok(false)
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn binary_reminder_process(
        self: &mut FSRThreadRuntime<'a>,
        _bytecode: &BytecodeArg,
    ) -> Result<bool, FSRError> {
        let right_id = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary mul 1",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let left_id = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary mul 2",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        if let Some(rust_fn) = obj_cls!(left_id).get_rust_fn(BinaryOffset::Reminder) {
            let res = rust_fn(&[left_id, right_id], self, self.get_context().code)?;

            match res {
                FSRRetValue::GlobalId(res_id) => {
                    self.get_cur_mut_frame().exp.push(res_id);
                }
            };

            self.get_cur_mut_frame().middle_value.push(right_id);
            self.get_cur_mut_frame().middle_value.push(left_id);

            return Ok(false);
        }

        let res = FSRObject::invoke_offset_method(
            BinaryOffset::Reminder,
            &[left_id, right_id],
            self,
            self.get_context().code,
        )?;
        match res {
            FSRRetValue::GlobalId(res_id) => {
                self.get_cur_mut_frame().exp.push(res_id);
            }
        };

        self.get_cur_mut_frame().middle_value.push(right_id);
        self.get_cur_mut_frame().middle_value.push(left_id);
        Ok(false)
    }

    fn binary_dot_process(
        self: &mut FSRThreadRuntime<'a>,
        bytecode: &BytecodeArg,
    ) -> Result<bool, FSRError> {
        let attr_id = if let ArgType::Attr(attr_id, name) = bytecode.get_arg() {
            (attr_id, name)
        } else {
            unimplemented!()
        };
        let dot_father = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in dot operator",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let dot_father_obj = FSRObject::id_to_obj(dot_father);
        let name = &attr_id.1;
        let id = if dot_father_obj.is_code() {
            //let name = attr_id.1;
            let id = dot_father_obj.get_attr(name);
            let id = match id {
                Some(s) => s,
                None => {
                    return Err(FSRError::new(
                        format!("not have this attr: `{}`", name),
                        FSRErrCode::NoSuchObject,
                    ))
                }
            };

            id
        } else {
            let id = dot_father_obj
                .get_attr(name)
                .expect(&format!("unfound attr: {}", name));

            id
        };

        self.get_cur_mut_frame()
            .exp
            .push(id.load(Ordering::Relaxed));
        self.get_cur_mut_frame().middle_value.push(dot_father);
        self.get_cur_mut_frame()
            .middle_value
            .push(id.load(Ordering::Relaxed));

        Ok(false)
    }

    fn binary_get_cls_attr_process(
        self: &mut FSRThreadRuntime<'a>,
        bytecode: &'a BytecodeArg,
    ) -> Result<bool, FSRError> {
        let attr_id = if let ArgType::Attr(attr_id, name) = bytecode.get_arg() {
            (attr_id, name)
        } else {
            unimplemented!()
        };

        let dot_father = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in dot operator",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let dot_father_obj = FSRObject::id_to_obj(dot_father);
        //println!("father: {:#?}", dot_father_obj);
        let name = &attr_id.1;
        let id = dot_father_obj.get_attr(name);

        if dot_father_obj.is_code() {
            let id = match id {
                Some(s) => s,
                None => {
                    return Err(FSRError::new(
                        format!("not have this attr: `{}`", name),
                        FSRErrCode::NoSuchObject,
                    ))
                }
            };

            self.get_cur_mut_frame()
                .exp
                .push(id.load(Ordering::Relaxed));
            //self.thread_allocator.free_box_attr(attr_id);
            return Ok(false);
        }
        if let Some(id) = id {
            self.get_cur_mut_frame()
                .exp
                .push(id.load(Ordering::Relaxed));
            self.get_cur_mut_frame().middle_value.push(dot_father);
            self.get_cur_mut_frame()
                .middle_value
                .push(id.load(Ordering::Relaxed));
        } else {
            panic!("not found object")
        }

        Ok(false)
    }

    fn binary_range_process(self: &mut FSRThreadRuntime<'a>) -> Result<bool, FSRError> {
        let rhs_id = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary mul 1",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let lhs_id = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary mul 2",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let start = FSRObject::id_to_obj(lhs_id);
        let end = FSRObject::id_to_obj(rhs_id);

        if let FSRValue::Integer(start) = start.value {
            if let FSRValue::Integer(end) = end.value {
                let range = FSRRange {
                    range: Range { start, end },
                };

                let obj = self.thread_allocator.new_object(
                    FSRValue::Range(Box::new(range)),
                    get_object_by_global_id(FSRGlobalObjId::RangeCls) as ObjId,
                );

                let id = FSRVM::leak_object(obj);

                self.get_cur_mut_frame().exp.push(id);
                self.get_cur_mut_frame().middle_value.push(rhs_id);
                self.get_cur_mut_frame().middle_value.push(lhs_id);
                return Ok(false);
            }
        }
        unimplemented!()
    }

    #[inline]
    fn call_process_set_args(
        args_num: usize,
        thread: &mut Self,
        code: ObjId,
        args: &mut SmallVec<[ObjId; 4]>,
    ) -> Result<(), FSRError> {
        let mut i = 0;
        while i < args_num {
            let a_id = thread.get_cur_mut_frame().exp.pop().unwrap();
            thread.get_cur_mut_frame().middle_value.push(a_id);
            args.push(a_id);
            i += 1;
        }

        Ok(())
    }

    // exp will be cleared after call
    #[inline]
    fn save_ip_to_callstate(
        &mut self,
        //ip: &mut (usize, usize),
    ) {
        //Self::call_process_set_args(args_num, self, exp, args);
        let ip = self.get_context().ip;
        let code = self.get_context().code;

        //state.exp = Some(exp.clone());
        // let store_exp = Some(std::mem::take(
        //     &mut self.get_cur_mut_frame().exp,
        //     // Vec::with_capacity(8),
        // ));
        let state = self.get_cur_mut_frame();
        state.set_reverse_ip(ip);
        // state.exp = store_exp;
        state.code = code;
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn process_fsr_cls(
        self: &mut FSRThreadRuntime<'a>,
        cls_id: ObjId,
        args: &mut SmallVec<[ObjId; 4]>,
    ) -> Result<bool, FSRError> {
        //let mut args = vec![];
        // New a object if fn_obj is fsr_cls
        let cls = FSRObject::id_to_obj(cls_id);
        if let FSRValue::Class(c) = &cls.value {
            if c.get_attr("__new__").is_none() {
                // let mut self_obj = FSRObject::new();
                // self_obj.set_cls(cls_id);
                // self_obj.set_value();

                let self_id = self.garbage_collect.new_object(
                    FSRValue::ClassInst(Box::new(FSRClassInst::new(c.get_name()))),
                    cls_id,
                );

                //let self_id = FSRVM::register_object(self_obj);
                self.get_cur_mut_frame().exp.push(self_id);

                return Ok(false);
            }
        }

        let fn_obj = FSRObject::id_to_obj(cls_id);
        let self_id = self.garbage_collect.new_object(
            FSRValue::ClassInst(Box::new(FSRClassInst::new(fn_obj.get_fsr_class_name()))),
            cls_id,
        );

        //args.insert(0, self_id);
        args.push(self_id);
        self.save_ip_to_callstate();
        let self_obj = FSRObject::id_to_obj(self_id);
        let self_new = self_obj.get_cls_attr("__new__");

        if let Some(self_new_obj) = self_new {
            let self_new_obj = self_new_obj.load(Ordering::Relaxed);
            let new_obj = FSRObject::id_to_obj(self_new_obj);

            if let FSRValue::Function(f) = &new_obj.value {
                self.get_cur_mut_context().call_end += 1;
                let frame = self.frame_free_list.new_frame(f.code, self_new_obj);
                self.get_cur_mut_context().code = f.code;
                self.push_frame(frame);
            } else {
                unimplemented!()
            }

            for arg in args.iter() {
                //obj.ref_add();
                self.get_cur_mut_frame().args.push(*arg);
            }

            let offset = new_obj.get_fsr_offset().1;
            self.get_cur_mut_context().ip = (offset.0, 0);
            Ok(true)
        } else {
            unimplemented!()
        }
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn process_fn_is_attr(
        self: &mut FSRThreadRuntime<'a>,
        obj_id: ObjId,
        fn_obj: &'a FSRObject<'a>,
        args: &mut SmallVec<[usize; 4]>,
    ) -> Result<bool, FSRError> {
        // let obj_id = context.exp.pop().unwrap().get_global_id();

        //args.insert(0, obj_id);
        args.push(obj_id);
        if fn_obj.is_fsr_function() {
            let ip = self.get_context().ip;

            // let store_exp = Some(std::mem::replace(
            //     &mut self.get_cur_mut_frame().exp,
            //     Vec::with_capacity(8),
            // ));
            let state = self.get_cur_mut_frame();
            //Save callstate
            state.set_reverse_ip(ip);
            //state.exp = store_exp;

            if let FSRValue::Function(f) = &fn_obj.value {
                self.get_cur_mut_context().call_end += 1;
                let mut frame = self
                    .frame_free_list
                    .new_frame(f.code, FSRObject::obj_to_id(fn_obj));
                frame.code = f.code;
                self.push_frame(frame);
            } else {
                panic!("not a function")
            }

            for arg in args.iter() {
                self.get_cur_mut_frame().args.push(*arg);
            }
            let offset = fn_obj.get_fsr_offset().1;
            if let FSRValue::Function(obj) = &fn_obj.value {
                self.get_cur_mut_context().code = obj.code;
            }
            self.get_cur_mut_context().ip = (offset.0, 0);
            return Ok(true);
        } else {
            args.reverse();
            let v = fn_obj
                .call(
                    args,
                    self,
                    self.get_context().code,
                    FSRObject::obj_to_id(fn_obj),
                )
                .unwrap();

            let id = v.get_id();
            self.get_cur_mut_frame().exp.push(id);
        }
        Ok(false)
    }

    #[inline]
    fn try_get_obj_by_name(&mut self, c_id: u64, name: &str, module: &FSRModule) -> Option<ObjId> {
        {
            let state = self.get_cur_mut_frame();
            if let Some(id) = state.get_var(&c_id) {
                return Some(id.load(Ordering::Relaxed));
            }
        }

        match module.get_object(name) {
            Some(s) => Some(s.load(Ordering::Relaxed)),
            None => {
                // Cache global object in call frame
                let v = self.get_vm().get_global_obj_by_name(name).cloned()?;

                let state = self.get_cur_mut_frame();
                state.insert_var(c_id, v);

                Some(v)
            }
        }
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn process_fsr_fn(
        &mut self,
        fn_id: ObjId,
        fn_obj: &FSRObject<'a>,
        args: &mut SmallVec<[ObjId; 4]>,
    ) -> Result<(), FSRError> {
        self.get_cur_mut_context().call_end += 1;
        self.save_ip_to_callstate();
        let f = fn_obj.as_fn();
        let frame = self.frame_free_list.new_frame(f.code, fn_id);
        self.push_frame(frame);

        for arg in args.iter() {
            self.get_cur_mut_frame().args.push(*arg);
        }
        //let offset = fn_obj.get_fsr_offset();
        //let offset = fn_obj.get_fsr_offset().1;
        //if let FSRValue::Function(obj) = &fn_obj.value {
        self.get_cur_mut_context().code = f.code;
        //}

        self.get_cur_mut_context().ip = (0, 0);
        Ok(())
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn get_fn_args(
        &mut self,
        var: &mut Option<&'a (usize, u64, String, bool)>,
        arg: &'a ArgType,
    ) -> Result<SmallVec<[usize; 4]>, FSRError> {
        let mut args = if let ArgType::CallArgsNumber(n) = *arg {
            // in case of method call like `obj.method()`, reserve the first arg for `self`
            let mut args: SmallVec<[usize; 4]> = SmallVec::<[ObjId; 4]>::new();
            let args_num = n;
            Self::call_process_set_args(args_num, self, self.get_context().code, &mut args)?;
            args.reverse();
            args
        } else if let ArgType::CallArgsNumberWithVar(pack) = arg {
            let mut args: SmallVec<[usize; 4]> = SmallVec::<[ObjId; 4]>::new();
            let args_num = pack.0;
            Self::call_process_set_args(args_num, self, self.get_context().code, &mut args)?;
            *var = Some(pack);
            args.reverse();
            args
        } else {
            SmallVec::new()
        };

        Ok(args)
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn get_call_fn_id(
        &mut self,
        var: &Option<&(usize, u64, String, bool)>,
        module: &FSRModule,
        object_id: Option<ObjId>,
        call_method: bool,
    ) -> Result<(ObjId), FSRError> {
        if let Some(var) = var {
            let var_id = var.1;
            let fn_id = self.try_get_obj_by_name(var.1, &var.2, module).unwrap();
            Ok(fn_id)
        } else {
            let fn_id = self.get_cur_mut_frame().exp.pop().unwrap();
            self.get_cur_mut_frame().middle_value.push(fn_id);
            Ok(fn_id)
        }
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn call_process_ret(
        &mut self,
        fn_id: ObjId,
        args: &mut SmallVec<[ObjId; 4]>,
        object_id: &Option<ObjId>,
        call_method: bool,
    ) -> Result<bool, FSRError> {
        let fn_obj = FSRObject::id_to_obj(fn_id);
        //let call_method = false;
        if fn_obj.is_fsr_function() && !call_method {
            self.process_fsr_fn(fn_id, fn_obj, args)?;
            return Ok(true);
        } else if fn_obj.is_fsr_cls() {
            let v = Self::process_fsr_cls(self, fn_id, args)?;
            if v {
                return Ok(v);
            }
        } else if object_id.is_some() && call_method {
            let v = Self::process_fn_is_attr(self, object_id.unwrap(), fn_obj, args)?;
            if v {
                return Ok(v);
            }
        } else {
            args.reverse();
            let v = match fn_obj.call(args, self, self.get_context().code, fn_id) {
                Ok(o) => o,
                Err(e) => {
                    if e.inner.code == FSRErrCode::RuntimeError {
                        self.exception = e.inner.exception.unwrap();
                        return Ok(false);
                    }

                    panic!("error: in call_process_ret: {}", e);
                }
            };

            let id = v.get_id();
            self.get_cur_mut_frame().exp.push(id);
        }

        Ok(false)
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn call_process(
        self: &mut FSRThreadRuntime<'a>,
        bytecode: &'a BytecodeArg,
    ) -> Result<bool, FSRError> {
        let mut var: Option<&(usize, u64, String, bool)> = None;
        let mut args: SmallVec<[usize; 4]> = SmallVec::<[ObjId; 4]>::new();
        //let mut args = self.get_fn_args(&mut var, bytecode.get_arg())?;
        if let ArgType::CallArgsNumber(args_num) = *bytecode.get_arg() {
            Self::call_process_set_args(args_num, self, self.get_context().code, &mut args)?;
            //args.reverse();
        } else if let ArgType::CallArgsNumberWithVar(pack) = bytecode.get_arg() {
            let args_num = pack.0;
            Self::call_process_set_args(args_num, self, self.get_context().code, &mut args)?;
            var = Some(pack);
            //args.reverse();
        };

        let mut object_id: Option<ObjId> = None;
        // let module = FSRObject::id_to_obj(self.get_context().code).as_code();
        let module = FSRObject::id_to_obj(
            FSRObject::id_to_obj(self.get_context().code)
                .as_code()
                .module,
        )
        .as_module();
        let (fn_id) = self.get_call_fn_id(&var, module, object_id, false)?;

        self.call_process_ret(fn_id, &mut args, &object_id, false)
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn call_method_process(
        self: &mut FSRThreadRuntime<'a>,
        bytecode: &'a BytecodeArg,
    ) -> Result<bool, FSRError> {
        let mut var: Option<&(usize, u64, String, bool)> = None;
        let mut args: SmallVec<[usize; 4]> = SmallVec::<[ObjId; 4]>::new();
        let father;
        let method = if let ArgType::CallArgsNumberWithAttr(pack) = bytecode.get_arg() {
            let args_num = pack.0;
            Self::call_process_set_args(args_num, self, self.get_context().code, &mut args)?;

            father = self.get_cur_mut_frame().exp.pop().unwrap();
            let father_obj = FSRObject::id_to_obj(father);
            let fn_id = father_obj.get_attr(&pack.2).unwrap();
            fn_id.load(Ordering::Relaxed)
        } else {
            unimplemented!()
        };

        //let method = self.get_cur_mut_frame().exp.pop().unwrap().get_global_id();

        let mut object_id: Option<ObjId> = Some(father);
        // let module = FSRObject::id_to_obj(self.get_context().code).as_code();
        let module = FSRObject::id_to_obj(
            FSRObject::id_to_obj(self.get_context().code)
                .as_code()
                .module,
        )
        .as_module();
        //let (fn_id) = self.get_call_fn_id(&var, module, object_id, true)?;

        self.call_process_ret(method, &mut args, &object_id, true)
    }

    fn try_process(
        self: &mut FSRThreadRuntime<'a>,
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
        let ip_0 = self.get_context().ip.0;

        self.get_cur_mut_frame()
            .catch_ends
            .push((ip_0 + catch_line.0 as usize, ip_0 + catch_line.1 as usize));
        Ok(false)
    }

    fn try_end(self: &mut FSRThreadRuntime<'a>) -> Result<bool, FSRError> {
        let end = self.get_cur_mut_frame().catch_ends.pop().unwrap();
        self.get_cur_mut_context().ip = (end.1, 0);
        Ok(true)
    }

    fn catch_end(
        self: &mut FSRThreadRuntime<'a>,

        _bytecode: &BytecodeArg,
    ) -> Result<bool, FSRError> {
        let state = self.get_cur_mut_frame();
        //state.catch_ends.pop().unwrap();
        state.handling_exception = FSRObject::none_id();
        Ok(true)
    }

    fn if_test_process(
        self: &mut FSRThreadRuntime<'a>,
        bytecode: &BytecodeArg,
    ) -> Result<bool, FSRError> {
        let test_val = self.get_cur_mut_frame().exp.pop().unwrap();
        let mut name = "";

        if test_val == FSRObject::false_id() || test_val == FSRObject::none_id() {
            if let ArgType::IfTestNext(n) = bytecode.get_arg() {
                let tmp = self.get_context().ip.0;
                self.get_cur_mut_context().ip = (tmp + n.0 as usize + 1_usize, 0);
                self.flow_tracker.push_last_if_test(false);
                return Ok(true);
            }
        }

        self.get_cur_mut_frame().middle_value.push(test_val);
        self.flow_tracker.push_last_if_test(true);
        Ok(false)
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn if_end(self: &mut FSRThreadRuntime<'a>, _bytecode: &BytecodeArg) -> Result<bool, FSRError> {
        self.flow_tracker.pop_last_if_test();
        Ok(false)
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn else_if_test_process(
        self: &mut FSRThreadRuntime<'a>,
        bytecode: &BytecodeArg,
    ) -> Result<bool, FSRError> {
        let test_val = self.get_cur_mut_frame().exp.pop().unwrap();
        self.get_cur_mut_frame().middle_value.push(test_val);
        if test_val == FSRObject::false_id() || test_val == FSRObject::none_id() {
            if let ArgType::IfTestNext(n) = bytecode.get_arg() {
                let tmp = self.get_context().ip.0;
                self.get_cur_mut_context().ip = (tmp + n.0 as usize + 1_usize, 0);
                self.flow_tracker.false_last_if_test();
                return Ok(true);
            }
        }
        self.flow_tracker.true_last_if_test();
        Ok(false)
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn else_process(
        self: &mut FSRThreadRuntime<'a>,
        bytecode: &BytecodeArg,
    ) -> Result<bool, FSRError> {
        if self.flow_tracker.peek_last_if_test() {
            if let ArgType::IfTestNext(n) = bytecode.get_arg() {
                self.get_cur_mut_context().ip =
                    (self.get_context().ip.0 + n.0 as usize + 1_usize, 0);
                return Ok(true);
            }
        }

        self.flow_tracker.false_last_if_test();
        Ok(false)
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn else_if_match(
        self: &mut FSRThreadRuntime<'a>,
        bytecode: &BytecodeArg,
    ) -> Result<bool, FSRError> {
        if self.flow_tracker.peek_last_if_test() {
            if let ArgType::IfTestNext(n) = bytecode.get_arg() {
                let tmp = self.get_context().ip.0;
                self.get_cur_mut_context().ip = (tmp + n.0 as usize + 1_usize, 0);
                return Ok(true);
            }
        }
        self.flow_tracker.false_last_if_test();
        Ok(false)
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn break_process(
        self: &mut FSRThreadRuntime<'a>,
        _bytecode: &BytecodeArg,
    ) -> Result<bool, FSRError> {
        self.flow_tracker.is_break = true;
        let l = self.flow_tracker.continue_line.len();
        let continue_line = self.flow_tracker.continue_line[l - 1];
        self.get_cur_mut_context().ip = (continue_line, 0);
        Ok(true)
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn continue_process(
        self: &mut FSRThreadRuntime<'a>,
        _bytecode: &BytecodeArg,
    ) -> Result<bool, FSRError> {
        let l = self.flow_tracker.continue_line.len();
        let continue_line = self.flow_tracker.continue_line[l - 1];
        self.get_cur_mut_context().ip = (continue_line, 0);
        Ok(true)
    }

    // save will fix
    fn for_block_ref(self: &mut FSRThreadRuntime<'a>) -> Result<bool, FSRError> {
        let obj_id = {
            let obj_id = *self.get_cur_frame().exp.last().unwrap();

            obj_id
        };

        self.flow_tracker.ref_for_obj.push(obj_id);
        Ok(false)
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn load_for_iter(
        self: &mut FSRThreadRuntime<'a>,
        bytecode: &BytecodeArg,
    ) -> Result<bool, FSRError> {
        let iter_id = self.get_cur_mut_frame().exp.pop().unwrap();

        let iter_obj = FSRObject::id_to_obj(iter_id);
        let read_iter_id = match iter_obj.get_attr("__iter__") {
            Some(s) => {
                let iter_fn = s.load(Ordering::Relaxed);
                let iter_fn_obj = FSRObject::id_to_obj(iter_fn);
                let ret = iter_fn_obj.call(&[iter_id], self, 0, iter_fn)?;
                ret.get_id()
            }
            None => iter_id,
        };

        self.get_cur_mut_frame().middle_value.push(iter_id);
        if let ArgType::ForLine(n) = bytecode.get_arg() {
            self.flow_tracker
                .break_line
                .push(self.get_context().ip.0 + *n as usize);
            self.flow_tracker
                .continue_line
                .push(self.get_context().ip.0 + 1);
        }
        self.flow_tracker.for_iter_obj.push(read_iter_id);
        Ok(false)
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn while_test_process(
        self: &mut FSRThreadRuntime<'a>,
        bytecode: &BytecodeArg,
    ) -> Result<bool, FSRError> {
        let test_val = self.get_cur_mut_frame().exp.pop().unwrap();
        self.get_cur_mut_frame().middle_value.push(test_val);

        if let ArgType::WhileTest(n) = bytecode.get_arg() {
            // Avoid repeat add break ip and continue ip
            if let Some(s) = self.flow_tracker.break_line.last() {
                if self.get_context().ip.0 + *n as usize + 1 != *s {
                    self.flow_tracker
                        .break_line
                        .push(self.get_context().ip.0 + *n as usize + 1);
                }
            } else {
                self.flow_tracker
                    .break_line
                    .push(self.get_context().ip.0 + *n as usize + 1);
            }

            if let Some(s) = self.flow_tracker.continue_line.last() {
                if self.get_context().ip.0 != *s {
                    self.flow_tracker
                        .continue_line
                        .push(self.get_context().ip.0);
                }
            } else {
                self.flow_tracker
                    .continue_line
                    .push(self.get_context().ip.0);
            }
        }

        if (test_val == FSRObject::false_id() || test_val == FSRObject::none_id())
            || self.flow_tracker.is_break
        {
            self.flow_tracker.is_break = false;
            if let ArgType::WhileTest(n) = bytecode.get_arg() {
                self.get_cur_mut_context().ip = (self.get_context().ip.0 + *n as usize + 1, 0);
                self.flow_tracker.break_line.pop();
                self.flow_tracker.continue_line.pop();
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn define_fn(
        self: &mut FSRThreadRuntime<'a>,
        bytecode: &'a BytecodeArg,
        //bc: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        if let ArgType::DefineFnArgs(name_id, name, fn_identify_name, args, store_to_cell) =
            bytecode.get_arg()
        {
            let module_id = FSRObject::id_to_obj(self.get_context().code)
                .as_code()
                .module;
            let module = FSRObject::id_to_obj(module_id).as_module();
            let fn_code = module.get_fn(fn_identify_name).unwrap();
            let fn_code_id = FSRObject::obj_to_id(fn_code);
            let fn_obj = FSRFn::from_fsr_fn(
                name,
                (0, 0),
                args.clone(),
                //bc,
                fn_code_id,
                self.get_cur_frame().fn_obj,
            );

            let fn_obj = self
                .thread_allocator
                .new_object(fn_obj, get_object_by_global_id(FSRGlobalObjId::FnCls));
            let fn_id = FSRVM::leak_object(fn_obj);
            let state = &mut self.cur_frame;
            if let Some(cur_cls) = &mut state.cur_cls {
                let offset = BinaryOffset::from_alias_name(name.as_str());
                if let Some(offset) = offset {
                    cur_cls.insert_offset_attr_obj_id(offset, fn_id);
                    self.get_cur_mut_context().ip = (self.get_context().ip.0 + 1, 0);
                    return Ok(true);
                }
                cur_cls.insert_attr_id(name, fn_id);
                self.get_cur_mut_context().ip = (self.get_context().ip.0 + 1, 0);
                return Ok(true);
            }

            state.insert_var(*name_id, fn_id);
            let define_fn_obj = self.get_cur_frame().fn_obj;
            if define_fn_obj == FSRObject::none_id() {
                let module = FSRObject::id_to_mut_obj(
                    FSRObject::id_to_obj(self.get_context().code)
                        .as_code()
                        .module,
                )
                .unwrap()
                .as_mut_module();
                module.register_object(name, fn_id);
            }
            if *store_to_cell {
                let define_fn_obj = self.get_cur_frame().fn_obj;
                if define_fn_obj == FSRObject::none_id() {
                    panic!("closure var must in closure");
                }
                let define_fn_obj = FSRObject::id_to_mut_obj(define_fn_obj)
                    .expect("not a fn obj")
                    .as_mut_fn();
                if let Some(s) = define_fn_obj.store_cells.get(name.as_str()) {
                    s.store(fn_id, Ordering::Relaxed);
                } else {
                    define_fn_obj
                        .store_cells
                        .insert(name.as_str(), AtomicObjId::new(fn_id));
                }
            }

            let ip_0 = self.get_context().ip.0;
            self.get_cur_mut_context().ip = (ip_0 + 1, 0);
            return Ok(true);
        }
        Ok(false)
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn compare_test(
        self: &mut FSRThreadRuntime<'a>,
        bytecode: &BytecodeArg,
    ) -> Result<bool, FSRError> {
        if let ArgType::Compare(op) = bytecode.get_arg() {
            //let len = self.get_cur_mut_frame().exp.len();
            let right_id = self.get_cur_mut_frame().exp.pop().ok_or_else(|| {
                FSRError::new(
                    "Failed to pop right operand from stack in compare_test",
                    FSRErrCode::EmptyExpStack,
                )
            })?;
            let left_id = *self.get_cur_mut_frame().exp.last().ok_or_else(|| {
                FSRError::new(
                    "Failed to pop left operand from stack in compare_test",
                    FSRErrCode::EmptyExpStack,
                )
            })?;

            let v = Self::compare(left_id, right_id, *op, self)?;

            // self.get_cur_mut_frame().middle_value.push(right_id);
            // self.get_cur_mut_frame().middle_value.push(left_id);
            if v {
                self.get_cur_mut_frame().exp.push(FSRObject::true_id())
            } else {
                self.get_cur_mut_frame().exp.push(FSRObject::false_id())
            }
        } else {
            return Err(FSRError::new(
                "not a compare test",
                FSRErrCode::NotValidArgs,
            ));
        }

        Ok(false)
    }

    #[inline(always)]
    pub fn compare_equal_process(
        self: &mut FSRThreadRuntime<'a>,
        bytecode: &BytecodeArg,
    ) -> Result<bool, FSRError> {
        let right = self.get_cur_mut_frame().exp.pop().unwrap();
        let left = *self.get_cur_mut_frame().exp.last().unwrap();


        if let Some(rust_fn) = obj_cls!(left).get_rust_fn(BinaryOffset::Equal) {
            let res = rust_fn(&[left, right], self, self.get_context().code)?;

            if res.get_id() == FSRObject::true_id() {
                self.get_cur_mut_frame().exp.push(FSRObject::true_id())
            } else {
                self.get_cur_mut_frame().exp.push(FSRObject::false_id())
            }

            return Ok(false);
        }

        let v = FSRObject::invoke_offset_method(
            BinaryOffset::Equal,
            &[left, right],
            self,
            self.get_context().code,
        )?
        .get_id()
            == FSRObject::true_id();
        //};

        if v {
            self.get_cur_mut_frame().exp.push(FSRObject::true_id())
        } else {
            self.get_cur_mut_frame().exp.push(FSRObject::false_id())
        }

        Ok(false)
    }

    fn ret_value(
        self: &mut FSRThreadRuntime<'a>,
        _bytecode: &BytecodeArg,
    ) -> Result<bool, FSRError> {
        let v = if self.get_cur_mut_frame().exp.is_empty() {
            FSRObject::none_id()
        } else {
            self.get_cur_mut_frame().exp.pop().unwrap()
        };

        self.get_cur_mut_frame().middle_value.push(v);
        self.pop_stack();
        let cur = self.get_cur_mut_frame();
        cur.ret_val = Some(v);
        let ip_0 = cur.reverse_ip.0;
        let ip_1 = cur.reverse_ip.1;
        let code = cur.code;
        self.get_cur_mut_context().ip = (ip_0, ip_1);
        self.get_cur_mut_context().code = code;
        self.get_cur_mut_context().call_end -= 1;
        // self.garbage_collect.add_root(v);
        Ok(true)
    }

    fn end_fn(self: &mut FSRThreadRuntime<'a>, _bytecode: &BytecodeArg) -> Result<bool, FSRError> {
        self.pop_stack();
        let cur = self.get_cur_mut_frame();
        let ip_0 = cur.reverse_ip.0;
        let ip_1 = cur.reverse_ip.1;
        let code = cur.code;
        cur.ret_val = Some(FSRObject::none_id());
        self.get_cur_mut_context().ip = (ip_0, ip_1 + 1);
        self.get_cur_mut_context().code = code;
        self.get_cur_mut_context().call_end -= 1;
        Ok(true)
    }

    fn for_block_end(
        self: &mut FSRThreadRuntime<'a>,
        bytecode: &BytecodeArg,
    ) -> Result<bool, FSRError> {
        if let ArgType::ForEnd(n) = bytecode.get_arg() {
            let tmp = self.get_context().ip.0;
            self.get_cur_mut_context().ip = (tmp - *n as usize, 0);
            return Ok(true);
        }

        Ok(false)
    }

    fn while_block_end(
        self: &mut FSRThreadRuntime<'a>,
        bytecode: &BytecodeArg,
    ) -> Result<bool, FSRError> {
        if let ArgType::WhileEnd(n) = bytecode.get_arg() {
            let tmp = self.get_context().ip.0;
            self.get_cur_mut_context().ip = (tmp - *n as usize, 0);
            return Ok(true);
        }

        Ok(false)
    }

    fn load_closure(&mut self, closure: &(u64, String)) -> Result<(), FSRError> {
        let obj_id = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new("", FSRErrCode::EmptyExpStack));
            }
        };

        self.get_cur_mut_frame().middle_value.push(obj_id);
        // FSRObject::id_to_obj(obj_id).ref_add();
        let fn_obj = self.get_cur_frame().fn_obj;
        if fn_obj == FSRObject::none_id() {
            panic!("closure var must in closure");
        }
        let fn_obj = FSRObject::id_to_mut_obj(fn_obj)
            .expect("not a fn object")
            .as_mut_fn();
        if let Some(s) = fn_obj.store_cells.get(closure.1.as_str()) {
            s.store(obj_id, Ordering::Relaxed);
        } else {
            fn_obj
                .store_cells
                .insert(closure.1.as_str(), AtomicObjId::new(obj_id));
        }

        Ok(())
    }

    fn assign_args(
        self: &mut FSRThreadRuntime<'a>,
        bytecode: &BytecodeArg,
    ) -> Result<bool, FSRError> {
        let state = &mut self.cur_frame;
        let v = state.args.pop().ok_or_else(|| {
            FSRError::new(
                "Failed to pop argument from stack in assign_args",
                FSRErrCode::EmptyExpStack,
            )
        })?;
        if let ArgType::Variable(s_id) = bytecode.get_arg() {
            state.insert_var(s_id.0, v);
        } else if let ArgType::ClosureVar(s_id) = bytecode.get_arg() {
            self.load_closure(s_id)?;
        }
        Ok(false)
    }

    // this is a special function for load list
    // will load the list to the stack
    fn load_list(
        self: &mut FSRThreadRuntime<'a>,
        bytecode: &BytecodeArg,
    ) -> Result<bool, FSRError> {
        if let ArgType::LoadListNumber(n) = bytecode.get_arg() {
            let mut list = Vec::with_capacity(*n);
            let n = *n;
            for _ in 0..n {
                let v_id = self.get_cur_mut_frame().exp.pop().ok_or_else(|| {
                    FSRError::new(
                        "Failed to pop value from stack in load_list",
                        FSRErrCode::EmptyExpStack,
                    )
                })?;

                list.push(v_id);
                self.get_cur_mut_frame().middle_value.push(v_id);
            }

            let list = self.garbage_collect.new_object(
                FSRList::new_value(list),
                get_object_by_global_id(FSRGlobalObjId::ListCls) as ObjId,
            );
            self.get_cur_mut_frame().exp.push(list);
        }

        Ok(false)
    }

    fn class_def(
        self: &mut FSRThreadRuntime<'a>,
        bytecode: &'a BytecodeArg,
    ) -> Result<bool, FSRError> {
        if let ArgType::Variable((id, name, store_to_cell)) = bytecode.get_arg() {
            let new_cls = FSRClass::new(name);
            let state = self.get_cur_mut_frame();
            state.cur_cls = Some(Box::new(new_cls));

            return Ok(false);
        }
        unimplemented!()
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
        bc: &BytecodeArg,
        context: ObjId,
    ) -> Result<bool, FSRError> {
        if let ArgType::ImportModule(v, module_name) = bc.get_arg() {
            let code = Self::read_code_from_module(module_name)?;
            let mut module = FSRModule::new_module(&module_name.join("."));
            let module_id = FSRVM::leak_object(Box::new(module));
            let fn_map = FSRCode::from_code(&module_name.join("."), &code, module_id)?;
            let module = FSRObject::id_to_mut_obj(module_id).unwrap();
            module.as_mut_module().init_fn_map(fn_map);

            //self.rt_unlock();
            let obj_id = { self.load(module_id)? };
            //self.rt_lock();
            let state = self.get_cur_mut_frame();
            state.insert_var(*v, obj_id);
            // FSRObject::id_to_mut_obj(context)
            //     .expect("not a code object")
            //     .as_mut_code()
            //     .register_object(module_name.last().unwrap(), obj_id);
            // Self::get_mut_module(context).register_object(module_name.last().unwrap(), obj_id);
            let module = FSRObject::id_to_mut_obj(FSRObject::id_to_obj(context).as_code().module)
                .unwrap()
                .as_mut_module();
            module.register_object(module_name.last().unwrap(), obj_id);
            return Ok(false);
        }
        unimplemented!()
    }

    fn end_class_def(self: &mut FSRThreadRuntime<'a>, bc: &BytecodeArg) -> Result<bool, FSRError> {
        if let ArgType::Variable(var) = bc.get_arg() {
            let id = var.0;
            let state = self.get_cur_mut_frame();
            let mut cls_obj = FSRObject::new();
            // cls_obj.set_cls(FSRGlobalObjId::ClassCls as ObjId);
            cls_obj.set_cls(get_object_by_global_id(FSRGlobalObjId::ClassCls));
            let obj = state.cur_cls.take().unwrap();

            let name = obj.get_name().to_string();
            cls_obj.set_value(FSRValue::Class(obj));
            let obj_id = FSRVM::register_object(cls_obj);
            state.insert_var(id, obj_id);
            // FSRObject::id_to_mut_obj(self.get_context().code)
            //     .expect("not a code object")
            //     .as_mut_code()
            //     .register_object(&name, obj_id);
            let module = FSRObject::id_to_mut_obj(
                FSRObject::id_to_obj(self.get_context().code)
                    .as_code()
                    .module,
            )
            .unwrap()
            .as_mut_module();
            module.register_object(&name, obj_id);
            // self.garbage_collect.add_root(obj_id);
        } else {
            unimplemented!()
        }

        Ok(false)
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn special_load_for(
        self: &mut FSRThreadRuntime<'a>,
        arg: &BytecodeArg,
    ) -> Result<bool, FSRError> {
        let obj = self.flow_tracker.for_iter_obj.last().cloned().unwrap();
        let obj_value = FSRObject::id_to_obj(obj);
        let res = if obj_value.cls == get_object_by_global_id(FSRGlobalObjId::InnerIterator) {
            // next_obj(&[obj], self, self.get_context().code)?
            crate::backend::types::iterator::next_obj(&[obj], self, self.get_context().code)?
        } else {
            FSRObject::invoke_offset_method(
                BinaryOffset::NextObject,
                &[obj],
                self,
                self.get_context().code,
            )?
        };

        let res_id = res.get_id();
        if res_id == FSRObject::none_id() || self.flow_tracker.is_break {
            self.flow_tracker.is_break = false;
            let break_line = self.flow_tracker.break_line.pop().unwrap();
            self.flow_tracker.continue_line.pop();
            let _ = self.flow_tracker.for_iter_obj.pop().unwrap();
            let _ = self.flow_tracker.ref_for_obj.pop().unwrap();

            self.get_cur_mut_context().ip = (break_line, 0);
            return Ok(true);
        }

        if let ArgType::Variable(v) = arg.get_arg() {
            let state = self.get_cur_mut_frame();
            state.insert_var(v.0, res_id);
            return Ok(false);
        }
        self.get_cur_mut_frame().exp.push(res_id);
        Ok(false)
    }

    fn process_logic_and(
        self: &mut FSRThreadRuntime<'a>,
        bc: &BytecodeArg,
    ) -> Result<bool, FSRError> {
        let first = self.get_cur_mut_frame().exp.pop().unwrap();
        if first == FSRObject::none_id() || first == FSRObject::false_id() {
            if let ArgType::AddOffset(offset) = bc.get_arg() {
                self.get_cur_mut_context().ip.1 += *offset;
                self.get_cur_mut_frame().exp.push(FSRObject::false_id());
            }
        }

        Ok(false)
    }

    // process logic or operator in bytecode
    fn process_logic_or(
        self: &mut FSRThreadRuntime<'a>,
        bc: &BytecodeArg,
    ) -> Result<bool, FSRError> {
        let first = self.get_cur_mut_frame().exp.pop().unwrap();
        if first != FSRObject::none_id() && first != FSRObject::false_id() {
            if let ArgType::AddOffset(offset) = bc.get_arg() {
                self.get_cur_mut_context().ip.1 += *offset;
                self.get_cur_mut_frame().exp.push(FSRObject::true_id());
            }
        }

        Ok(false)
    }

    fn not_process(self: &mut FSRThreadRuntime<'a>) -> Result<bool, FSRError> {
        let v1_id = match self.get_cur_frame().exp.last() {
            Some(s) => *s,
            None => {
                return Err(FSRError::new(
                    "error in binary add 1",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        // let mut target = false;
        let target = FSRObject::none_id() == v1_id || FSRObject::false_id() == v1_id;

        if let Some(x) = self.get_cur_mut_frame().exp.pop() {}

        self.get_cur_mut_frame().middle_value.push(v1_id);

        if target {
            self.get_cur_mut_frame().exp.push(FSRObject::true_id());
        } else {
            self.get_cur_mut_frame().exp.push(FSRObject::false_id());
        }

        Ok(false)
    }

    fn empty_process(self: &mut FSRThreadRuntime<'a>, _bc: &BytecodeArg) -> Result<bool, FSRError> {
        Ok(false)
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn process(&mut self, bytecode: &'a BytecodeArg) -> Result<bool, FSRError> {
        let op = bytecode.get_operator();

        let v = match op {
            BytecodeOperator::Assign => Self::assign_process(self, bytecode),
            BytecodeOperator::BinaryAdd => Self::binary_add_process(self),
            BytecodeOperator::BinaryDot => Self::binary_dot_process(self, bytecode),
            BytecodeOperator::BinaryMul => Self::binary_mul_process(self, bytecode),
            BytecodeOperator::Call => Self::call_process(self, bytecode),
            BytecodeOperator::IfTest => Self::if_test_process(self, bytecode),
            BytecodeOperator::WhileTest => Self::while_test_process(self, bytecode),
            BytecodeOperator::DefineFn => Self::define_fn(self, bytecode),
            BytecodeOperator::EndFn => Self::end_fn(self, bytecode),
            BytecodeOperator::CompareTest => Self::compare_test(self, bytecode),
            BytecodeOperator::ReturnValue => Self::ret_value(self, bytecode),
            BytecodeOperator::WhileBlockEnd => Self::while_block_end(self, bytecode),
            BytecodeOperator::AssignArgs => Self::assign_args(self, bytecode),
            BytecodeOperator::ClassDef => Self::class_def(self, bytecode),
            BytecodeOperator::EndDefineClass => Self::end_class_def(self, bytecode),
            BytecodeOperator::LoadList => Self::load_list(self, bytecode),
            BytecodeOperator::Else => Self::else_process(self, bytecode),
            BytecodeOperator::ElseIf => Self::else_if_match(self, bytecode),
            BytecodeOperator::ElseIfTest => Self::else_if_test_process(self, bytecode),
            BytecodeOperator::IfBlockEnd => Self::if_end(self, bytecode),
            BytecodeOperator::Break => Self::break_process(self, bytecode),
            BytecodeOperator::Continue => Self::continue_process(self, bytecode),
            BytecodeOperator::LoadForIter => Self::load_for_iter(self, bytecode),
            BytecodeOperator::ForBlockEnd => Self::for_block_end(self, bytecode),
            BytecodeOperator::SpecialLoadFor => Self::special_load_for(self, bytecode),
            BytecodeOperator::AndJump => Self::process_logic_and(self, bytecode),
            BytecodeOperator::OrJump => Self::process_logic_or(self, bytecode),
            BytecodeOperator::Empty => Self::empty_process(self, bytecode),
            BytecodeOperator::BinarySub => Self::binary_sub_process(self, bytecode),
            BytecodeOperator::Import => {
                Self::process_import(self, bytecode, self.get_context().code)
            }
            BytecodeOperator::BinaryDiv => Self::binary_div_process(self, bytecode),
            BytecodeOperator::NotOperator => Self::not_process(self),
            BytecodeOperator::BinaryClassGetter => {
                Self::binary_get_cls_attr_process(self, bytecode)
            }
            BytecodeOperator::Getter => Self::getter_process(self, bytecode),
            BytecodeOperator::Try => Self::try_process(self, bytecode),
            BytecodeOperator::EndTry => Self::try_end(self),
            BytecodeOperator::EndCatch => Self::catch_end(self, bytecode),
            BytecodeOperator::BinaryRange => Self::binary_range_process(self),
            BytecodeOperator::ForBlockRefAdd => Self::for_block_ref(self),
            BytecodeOperator::LoadConst => Self::load_const(self, bytecode),
            BytecodeOperator::BinaryReminder => Self::binary_reminder_process(self, bytecode),
            BytecodeOperator::AssignContainer => Self::getter_assign_process(self, bytecode),
            BytecodeOperator::AssignAttr => Self::attr_assign_process(self, bytecode),
            BytecodeOperator::CallMethod => Self::call_method_process(self, bytecode),
            BytecodeOperator::CompareEqual => Self::compare_equal_process(self, bytecode),
            BytecodeOperator::Load => Self::load_var(self, bytecode),
            _ => {
                panic!("not implement for {:#?}", op);
            }
        };

        let v = match v {
            Ok(o) => o,
            Err(e) => {
                if e.inner.code == FSRErrCode::RuntimeError {
                    self.exception = e.inner.exception.unwrap();
                    return Ok(false);
                }

                return Err(e);
            }
        };

        Ok(v)
    }

    fn load_const(&mut self, arg: &'a BytecodeArg) -> Result<bool, FSRError> {
        let code = FSRObject::id_to_obj(self.get_context().code)
            .as_code()
            .module;
        let module = FSRObject::id_to_mut_obj(code).unwrap().as_mut_module();
        match arg.get_arg() {
            ArgType::ConstInteger(index, obj, single_op) => {
                let i = obj.parse::<i64>().unwrap();
                let i = if single_op.is_some() && single_op.as_ref().unwrap().eq("-") {
                    -i
                } else {
                    i
                };
                let ptr = {
                    let mut obj = FSRInteger::new_inst(i);
                    // obj.ref_add();
                    obj.area = Area::Global;
                    FSRVM::leak_object(Box::new(obj))
                };

                module.insert_const(*index as usize, ptr);
            }
            ArgType::ConstFloat(index, obj, single_op) => {
                let i = obj.parse::<f64>().unwrap();
                let i = if single_op.is_some() && single_op.as_ref().unwrap().eq("-") {
                    -1.0 * i
                } else {
                    i
                };
                let ptr = {
                    let mut obj = FSRFloat::new_inst(i);
                    // obj.ref_add();
                    obj.area = Area::Global;
                    FSRVM::leak_object(Box::new(obj))
                };

                module.insert_const(*index as usize, ptr);
            }
            ArgType::ConstString(index, s) => {
                let obj = FSRString::new_value(s);
                // obj.ref_add();
                let obj =
                    FSRObject::new_inst(obj, get_object_by_global_id(FSRGlobalObjId::StringCls));
                let ptr = FSRVM::leak_object(Box::new(obj));

                module.insert_const(*index as usize, ptr);
            }
            _ => unimplemented!(),
        }

        Ok(false)
    }

    fn get_chains(
        thread: &FSRThreadRuntime,
        state: &CallFrame<'_>,
        var: &(u64, String, bool),
    ) -> Option<ObjId> {
        let fn_id = state.fn_obj;
        // if in __main__ the module base code
        if fn_id != 0 {
            let obj = FSRObject::id_to_obj(fn_id).as_fn();
            if let Some(s) = obj.get_closure_var(var.1.as_str()) {
                return Some(s);
            }
        }
        // let module = FSRObject::id_to_obj(state.code).as_code();
        let module =
            FSRObject::id_to_obj(FSRObject::id_to_obj(state.code).as_code().module).as_module();
        let vm = thread.get_vm();
        let v = match module.get_object(&var.1) {
            Some(s) => s.load(Ordering::Relaxed),
            None => match vm.get_global_obj_by_name(&var.1) {
                Some(s) => *s,
                None => {
                    unimplemented!("not found var: {}", var.1);
                }
            },
        };

        Some(v)
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn load_var(&mut self, arg: &'a BytecodeArg) -> Result<bool, FSRError> {
        //let exp = &mut self.get_cur_mut_frame().exp;
        match arg.get_arg() {
            ArgType::Variable(var) => {
                let state = self.get_cur_frame();
                let id = if let Some(s) = state.get_var(&var.0) {
                    s.load(Ordering::Relaxed)
                } else {
                    Self::get_chains(self, state, var).unwrap()
                };
                self.get_cur_mut_frame().exp.push(id);
            }
            ArgType::Const(index) => {
                let code = FSRObject::id_to_obj(self.get_context().code)
                    .as_code()
                    .module;
                let module = FSRObject::id_to_obj(code).as_module();
                let obj = module.get_const(*index as usize).unwrap();
                self.get_cur_mut_frame().exp.push(obj);
            }

            ArgType::ClosureVar(v) => {
                let fn_id = self.get_cur_frame().fn_obj;
                if fn_id == FSRObject::none_id() {
                    panic!("not found function object");
                }
                let fn_obj = FSRObject::id_to_obj(fn_id).as_fn();
                let var = fn_obj.get_closure_var(&v.1);
                self.get_cur_mut_frame().exp.push(var.unwrap());
            }
            ArgType::CurrentFn => {
                let fn_id = self.get_cur_frame().fn_obj;
                if fn_id == FSRObject::none_id() {
                    panic!("not found function object");
                }
                self.get_cur_mut_frame().exp.push(fn_id);
            }
            ArgType::GlobalId(id) => {
                self.get_cur_mut_frame().exp.push(*id);
            }
            _ => {
                println!("{:?}", self.get_cur_mut_frame().exp);
                unimplemented!()
            }
        }

        Ok(false)
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn set_exp_stack_ret(&mut self) {
        let state = self.get_cur_frame();
        // if state.exp.is_some() {
        //     let v = self.get_cur_mut_frame().exp.take().unwrap();
        //     self.get_cur_mut_context().exp = v;
        // }

        if self.get_cur_mut_frame().ret_val.is_some() {
            let v = self.get_cur_mut_frame().ret_val.take().unwrap();
            self.get_cur_mut_frame().exp.push(v);
        }
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn exception_process(&mut self) -> bool {
        if self.exception_flag {
            if !self.get_cur_mut_frame().catch_ends.is_empty() {
                self.get_cur_mut_frame().handling_exception = self.exception;
                self.exception = FSRObject::none_id();
                self.exception_flag = false;
                self.get_cur_mut_context().ip =
                    (self.get_cur_mut_frame().catch_ends.pop().unwrap().0, 0);
                // self.garbage_collect.add_root(exception_handling);
                return true;
            } else {
                if self.call_frames.is_empty() {
                    panic!("No handle of error")
                }
                self.pop_stack();
                let cur = self.get_cur_mut_frame();
                let ip_0 = cur.reverse_ip.0;
                let ip_1 = cur.reverse_ip.1;
                let code = cur.code;
                self.get_cur_mut_context().ip = (ip_0, ip_1 + 1);
                self.get_cur_mut_context().code = code;
                self.get_cur_mut_context().call_end -= 1;
                // self.garbage_collect.add_root(self.exception);
                return true;
            }
        }

        false
    }

    pub fn release(&mut self) {}

    pub fn acquire(&mut self) {
        //{
        let mut in_rt_cxt = self.til.in_rt_cxt.0.lock().unwrap();

        while !*in_rt_cxt {
            // println!("sim slow down");
            // sleep(Duration::from_secs(1));
            *self.til.is_stop.0.lock().unwrap() = true;
            self.til.is_stop.1.notify_all();
            println!("wait runtime: {}", self.thread_id);
            in_rt_cxt = self.til.in_rt_cxt.1.wait(in_rt_cxt).unwrap();
            println!("receive runtime: {}, {}", self.thread_id, *in_rt_cxt);
        }

        *self.til.is_stop.0.lock().unwrap() = false;
        //}

        self.last_aquire_counter = self.counter;
    }

    fn rt_yield(&mut self) {
        //self.safe_point_to_stop();
        self.acquire();
    }

    pub fn safe_point_to_stop(&self) {
        // let mut in_rt_ctx = self.til.in_rt_cxt.0.lock().unwrap();
        // *in_rt_ctx = false;
        *self.til.is_stop.0.lock().unwrap() = true;
        self.til.is_stop.1.notify_all();
    }

    pub fn rt_stop(&self) {
        {
            let mut locker = self.til.in_rt_cxt.0.lock().unwrap();
            println!("stop runtime: {}", self.thread_id);
            *locker = false;
        }

        // wait
    }

    pub fn rt_continue(&self) {
        println!("continue thread {}", self.thread_id);
        {
            let mut locker = self.til.in_rt_cxt.0.lock().unwrap();
            while !*locker {
                *locker = true;
            }
        }

        println!("send notify thread {}", self.thread_id);

        self.til.in_rt_cxt.1.notify_all();
        println!("continued thread {}", self.thread_id);
    }

    pub fn rt_wait_stop(&self) {
        let mut is_stop = self.til.is_stop.0.lock().unwrap();
        while !*is_stop {
            is_stop = self.til.is_stop.1.wait(is_stop).unwrap();
        }

        println!("thread {}: is stopped", self.thread_id);
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn run_expr_wrapper(&mut self, expr: &'a [BytecodeArg]) -> Result<bool, FSRError> {
        if self.counter - self.last_aquire_counter > 100 {
            self.rt_yield();
        }

        self.run_expr(expr)
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn run_expr(&mut self, expr: &'a [BytecodeArg]) -> Result<bool, FSRError> {
        let mut v;

        self.set_exp_stack_ret();
        while let Some(arg) = expr.get(self.get_context().ip.1) {
            self.get_cur_mut_context().ip.1 += 1;
            // let arg = &expr[context.ip.1];
            #[cfg(feature = "bytecode_trace")]
            {
                let t = format!("{:?} => {:?}", self.get_context().ip, arg);
                println!("{:?}", self.get_cur_frame().exp);
                println!("{}", t);
            }
            self.counter += 1;
            #[cfg(feature = "count_bytecode")]
            {
                self.bytecode_counter[*arg.get_operator() as usize] += 1;
            }

            v = self.process(arg)?;
            if self.get_cur_frame().ret_val.is_some() {
                return Ok(true);
            }

            if v {
                self.get_cur_mut_frame().exp.clear();
                self.get_cur_mut_frame().middle_value.clear();
                return Ok(false);
            }

            if Self::exception_process(self) {
                return Ok(true);
            }
        }
        self.get_cur_mut_context().ip.0 += 1;
        self.get_cur_mut_context().ip.1 = 0;
        self.get_cur_mut_frame().exp.clear();
        self.get_cur_mut_frame().middle_value.clear();

        if self.garbage_collect.will_collect() {
            let st = std::time::Instant::now();
            if self.gc_context.gc_state == GcState::Stop {
                self.clear_marks();
            }
            self.set_ref_objects_mark(false);
            if self.gc_context.worklist.is_empty() {
                self.collect_gc(false);
                self.gc_context.gc_state = GcState::Stop;
            }

            self.garbage_collect.tracker.collect_time += st.elapsed().as_micros() as u64;
        }
        Ok(false)
    }

    pub fn collect_gc(&mut self, full: bool) {
        self.garbage_collect.collect(full);
    }

    pub fn load(&mut self, main_fn: ObjId) -> Result<ObjId, FSRError> {
        let code = FSRObject::id_to_obj(main_fn)
            .as_module()
            .get_fn("__main__")
            .unwrap();
        let code_id = FSRObject::obj_to_id(code);
        let context = Box::new(FSCodeContext {
            //exp: Vec::with_capacity(8),
            ip: (0, 0),
            code: code_id,
            call_end: 1,
        });

        self.push_context(context);

        let frame = self.frame_free_list.new_frame(code_id, 0);
        self.push_frame(frame);
        //self.unlock_and_lock();
        let mut code = FSRObject::id_to_obj(code_id).as_code();
        while let Some(expr) = code.get_expr(self.get_context().ip.0) {
            self.run_expr_wrapper(expr)?;
            code = FSRObject::id_to_obj(self.get_context().code).as_code();
        }
        //self.rt_unlock();
        self.pop_frame();
        self.pop_context();
        // if let Some(s) = self.pop_context() {
        //     self.thread_allocator.free_code_context(s);
        // }

        Ok(code_id)
    }

    pub fn start(&mut self, module: ObjId) -> Result<(), FSRError> {
        let code_id = FSRObject::obj_to_id(
            FSRObject::id_to_obj(module)
                .as_module()
                .get_fn("__main__")
                .unwrap(),
        );
        let context = self.thread_allocator.new_code_context(code_id);
        self.push_context(context);
        let mut main_code = None;
        for code in FSRObject::id_to_obj(module).as_module().iter_fn() {
            if code.0 == "__main__" {
                main_code = Some(code.1);
                continue;
            }
            //let obj = FSRObject::obj_to_id(code.1);
            //self.run_with_context(FSRObject::obj_to_id(code.1), &mut context)?;
        }

        self.cur_frame.code = code_id;
        self.get_cur_mut_context().code = FSRObject::obj_to_id(main_code.unwrap());
        let mut code = FSRObject::id_to_obj(code_id).as_code();
        //self.get_cur_mut_frame().fn_obj = code_id;
        while let Some(expr) = code.get_expr(self.get_context().ip.0) {
            #[cfg(feature = "bytecode_trace")]
            {
                println!(
                    "cur_module: {}",
                    FSRObject::id_to_obj(self.get_context().code)
                        .as_code()
                        .as_string()
                )
            }
            self.run_expr_wrapper(expr)?;
            code = FSRObject::id_to_obj(self.get_context().code).as_code();
        }

        println!("count: {}", self.counter);
        #[cfg(feature = "count_bytecode")]
        {
            self.dump_bytecode_counter();
        }
        Ok(())
    }

    pub fn call_fn(
        &mut self,
        fn_def: &'a FSRFnInner,
        args: &[ObjId],
        code: ObjId,
    ) -> Result<ObjId, FSRError> {
        let mut context = self.thread_allocator.new_code_context(code);
        context.ip = fn_def.get_ip();
        context.code = code;

        self.push_context(context);
        //self.rt_lock();
        {
            self.get_cur_mut_frame().exp.clear();

            for arg in args.iter().rev() {
                self.get_cur_mut_frame().args.push(*arg);
            }
            let offset = fn_def.get_ip();
            self.get_cur_mut_context().ip = (offset.0, 0);
        }
        let mut code = FSRObject::id_to_obj(self.get_context().code).as_code();
        while let Some(expr) = code.get_expr(self.get_context().ip.0) {
            let v = self.run_expr_wrapper(expr)?;
            if self.exception_flag {
                // If this is last function call, in this call_fn
                if self.get_context().call_end == 0 {
                    let context = self.pop_context();
                    self.thread_allocator.free_code_context(context);
                    return Err(FSRError::new_runtime_error(self.exception));
                }
            }

            if self.get_context().call_end == 0 {
                break;
            }

            code = FSRObject::id_to_obj(self.get_context().code).as_code();
        }

        let cur = self.get_cur_mut_frame();
        if cur.ret_val.is_none() {
            let context = self.pop_context();
            self.thread_allocator.free_code_context(context);
            return Ok(FSRObject::none_id());
        }
        let ret_val = cur.ret_val.take();

        let context = self.pop_context();
        self.thread_allocator.free_code_context(context);
        match ret_val {
            Some(s) => Ok(s),
            None => Ok(0),
        }
    }
}

#[allow(unused_imports)]
mod test {
    use std::sync::{Arc, Mutex};

    use crate::{
        backend::{
            types::{
                base::{FSRObject, ObjId},
                code::FSRCode,
                module::FSRModule,
            },
            vm::virtual_machine::FSRVM,
        },
        utils::error::FSRError,
    };

    use super::FSRThreadRuntime;

    #[test]
    fn test_export() {
        FSRVM::single();
        let source_code = r#"
        i = 0
        export("i", i)

        fn abc() {
            return 'abc'
        }

        export('abc', abc)
        "#;
        let mut obj: Box<FSRObject<'_>> = Box::new(FSRModule::new_module("main"));
        let obj_id = FSRVM::leak_object(obj);
        let v = FSRCode::from_code("main", source_code, obj_id).unwrap();
        let obj = FSRObject::id_to_mut_obj(obj_id).unwrap();
        obj.as_mut_module().init_fn_map(v);
        // let v = v.remove("__main__").unwrap();
        // let base_module = FSRVM::leak_object(Box::new(v));
        let mut runtime = FSRThreadRuntime::new_runtime();
        runtime.start(obj_id).unwrap();

        // println!("{:?}", FSRObject::id_to_obj(v.get_object("abc").unwrap()));
    }

    #[test]
    fn test_float() {
        FSRVM::single();
        let source_code = r#"
        i = 1.1
        b = 1.2
        dump(i + b)
        "#;

        let mut obj: Box<FSRObject<'_>> = Box::new(FSRModule::new_module("main"));
        let obj_id = FSRVM::leak_object(obj);
        let v = FSRCode::from_code("main", source_code, obj_id).unwrap();
        let obj = FSRObject::id_to_mut_obj(obj_id).unwrap();
        obj.as_mut_module().init_fn_map(v);

        // let v = v.remove("__main__").unwrap();
        // let base_module = FSRVM::leak_object(Box::new(v));
        let mut runtime = FSRThreadRuntime::new_runtime();
        runtime.start(obj_id).unwrap();
    }

    // #[test]
    // fn test_print_str() {
    //     let source_code = r#"
    //     class Test {
    //         fn __new__(self) {
    //             self.abc = 123
    //             return self
    //         }

    //         fn __str__(self) {
    //             return 'abc'
    //         }
    //     }
    //     t = Test()
    //     println(t)
    //     "#;
    //     let mut v = FSRCode::from_code("main", source_code).unwrap();
    //     let v = v.remove("__main__").unwrap();
    //     let base_module = FSRVM::leak_object(Box::new(v));
    //     let mut vm = Arc::new(Mutex::new(FSRVM::new()));
    //     let mut runtime = FSRThreadRuntime::new(base_module, vm);
    //     runtime.start(base_module).unwrap();
    // }

    // #[test]
    // fn test_binary_div() {
    //     let source_code = r#"
    //     a = 1.0 / 2.0
    //     println(a)
    //     "#;
    //     let mut v = FSRCode::from_code("main", source_code).unwrap();
    //     let v = v.remove("__main__").unwrap();
    //     let base_module = FSRVM::leak_object(Box::new(v));
    //     let mut vm = Arc::new(Mutex::new(FSRVM::new()));
    //     let mut runtime = FSRThreadRuntime::new(base_module, vm);
    //     runtime.start(base_module).unwrap();
    // }

    // #[test]
    // fn test_class_without_new() {
    //     let source_code = r#"
    //     class Test {
    //         fn abc() {
    //             println("abc")
    //         }
    //     }
    //     Test::abc()
    //     "#;
    //     let mut v = FSRCode::from_code("main", source_code).unwrap();
    //     let v = v.remove("__main__").unwrap();
    //     let base_module = FSRVM::leak_object(Box::new(v));
    //     let mut vm = Arc::new(Mutex::new(FSRVM::new()));
    //     let mut runtime = FSRThreadRuntime::new(base_module, vm);
    //     runtime.start(base_module).unwrap();
    // }

    #[test]
    fn get_list_item() {
        let source_code = r#"
        a = [1, 2, 3]
        println(a[0])

        b = [[1,2,3]]
        c = b[0][0]
        println(c)
        "#;
        let mut obj: Box<FSRObject<'_>> = Box::new(FSRModule::new_module("main"));
        let obj_id = FSRVM::leak_object(obj);
        let v = FSRCode::from_code("main", source_code, obj_id).unwrap();
        let obj = FSRObject::id_to_mut_obj(obj_id).unwrap();
        obj.as_mut_module().init_fn_map(v);
        // let v = v.remove("__main__").unwrap();
        // let base_module = FSRVM::leak_object(Box::new(v));
        let mut runtime = FSRThreadRuntime::new_runtime();
        runtime.start(obj_id).unwrap();
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
        let mut obj: Box<FSRObject<'_>> = Box::new(FSRModule::new_module("main"));
        let obj_id = FSRVM::leak_object(obj);
        let v = FSRCode::from_code("main", source_code, obj_id).unwrap();
        let obj = FSRObject::id_to_mut_obj(obj_id).unwrap();
        obj.as_mut_module().init_fn_map(v);
        // let v = v.remove("__main__").unwrap();
        // let base_module = FSRVM::leak_object(Box::new(v));
        let mut runtime = FSRThreadRuntime::new_runtime();
        runtime.start(obj_id).unwrap();
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
        let mut obj: Box<FSRObject<'_>> = Box::new(FSRModule::new_module("main"));
        let obj_id = FSRVM::leak_object(obj);
        let v = FSRCode::from_code("main", source_code, obj_id).unwrap();
        let obj = FSRObject::id_to_mut_obj(obj_id).unwrap();
        obj.as_mut_module().init_fn_map(v);
        // let v = v.remove("__main__").unwrap();
        // let base_module = FSRVM::leak_object(Box::new(v));
        let mut runtime = FSRThreadRuntime::new_runtime();
        runtime.start(obj_id).unwrap();
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
        let mut obj: Box<FSRObject<'_>> = Box::new(FSRModule::new_module("main"));
        let obj_id = FSRVM::leak_object(obj);
        let v = FSRCode::from_code("main", source_code, obj_id).unwrap();
        let obj = FSRObject::id_to_mut_obj(obj_id).unwrap();
        obj.as_mut_module().init_fn_map(v);
        // let v = v.remove("__main__").unwrap();
        // let base_module = FSRVM::leak_object(Box::new(v));
        let mut runtime = FSRThreadRuntime::new_runtime();
        runtime.start(obj_id).unwrap();
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
        let mut obj: Box<FSRObject<'_>> = Box::new(FSRModule::new_module("main"));
        let obj_id = FSRVM::leak_object(obj);
        let v = FSRCode::from_code("main", source_code, obj_id).unwrap();
        let obj = FSRObject::id_to_mut_obj(obj_id).unwrap();
        obj.as_mut_module().init_fn_map(v);
        // let v = v.remove("__main__").unwrap();
        // let base_module = FSRVM::leak_object(Box::new(v));
        let mut runtime = FSRThreadRuntime::new_runtime();
        runtime.start(obj_id).unwrap();
    }

    #[test]
    fn test_lambda() {
        FSRVM::single();
        let source_code = r#"
        a = || {
            println("abc")
            assert(true)
        }

        a()
        "#;
        let mut obj: Box<FSRObject<'_>> = Box::new(FSRModule::new_module("main"));
        let obj_id = FSRVM::leak_object(obj);
        let v = FSRCode::from_code("main", source_code, obj_id).unwrap();
        let obj = FSRObject::id_to_mut_obj(obj_id).unwrap();
        obj.as_mut_module().init_fn_map(v);
        // let v = v.remove("__main__").unwrap();
        // let base_module = FSRVM::leak_object(Box::new(v));
        let mut runtime = FSRThreadRuntime::new_runtime();
        runtime.start(obj_id).unwrap();
    }

    #[test]
    fn test_gc() {
        let source_code = r#"
        a = 1 + 1
        c = a + 2
        
        a = 1 + 3
        a = 1
        gc_info()
        gc_collect()
        gc_info()
        "#;
        let mut obj: Box<FSRObject<'_>> = Box::new(FSRModule::new_module("main"));
        let obj_id = FSRVM::leak_object(obj);
        let v = FSRCode::from_code("main", source_code, obj_id).unwrap();
        let obj = FSRObject::id_to_mut_obj(obj_id).unwrap();
        obj.as_mut_module().init_fn_map(v);
        // let v = v.remove("__main__").unwrap();
        // let base_module = FSRVM::leak_object(Box::new(v));
        let mut runtime = FSRThreadRuntime::new_runtime();
        runtime.start(obj_id).unwrap();
    }
}
