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
    virtual_machine::{FSRVM, VM},
};

macro_rules! obj_cls {
    ($a:expr) => {
        FSRObject::id_to_obj($a).cls
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
    #[inline(always)]
    pub fn get(&self, i: &u64) -> Option<&AtomicObjId> {
        match self.vs.get(*i as usize) {
            Some(Some(s)) => Some(s),
            Some(None) => None,
            None => None,
        }
    }

    #[inline(always)]
    pub fn insert(&mut self, i: u64, v: ObjId) {
        if i as usize >= self.vs.len() {
            let new_capacity = (i + 1) + (4 - (i + 1) % 4);
            self.vs.resize_with(new_capacity as usize, || None);
        }

        if let Some(s) = self.vs.get(i as usize) {
            if let Some(s) = s {
                s.store(v, Ordering::Relaxed);
                return;
            }
        }
        self.vs[i as usize] = Some(AtomicObjId::new(v));
    }

    #[inline(always)]
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

    #[inline(always)]
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

    #[inline(always)]
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

    #[inline(always)]
    pub fn clear(&mut self) {
        self.attr_map.clear();
    }

    #[inline(always)]
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
    pub(crate) exp: Vec<SValue<'a>>,
    /// in case of garbage collection collecting this object, this object is for middle value for expression
    pub(crate) middle_value: Vec<ObjId>,
    pub(crate) code: ObjId,
    catch_ends: Vec<(usize, usize)>,
    pub(crate) handling_exception: ObjId,
    // Record current call fn_obj
    pub(crate) fn_obj: ObjId,
}

impl<'a> CallFrame<'a> {
    #[inline(always)]
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

    #[inline(always)]
    pub fn get_var(&self, id: &u64) -> Option<&AtomicObjId> {
        self.var_map.get(id)
    }

    pub fn get_attr(&self, i: usize, j: usize) -> Option<&'a AtomicObjId> {
        self.attr_map.get_attr(i, j)
    }

    #[inline(always)]
    pub fn insert_var(&mut self, id: u64, obj_id: ObjId) {
        self.var_map.insert(id, obj_id);
        self.attr_map.clear_var(id as usize);
    }

    #[inline(always)]
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
pub struct AttrArgs<'a> {
    pub(crate) attr_id: u64,
    pub(crate) father: ObjId,
    pub(crate) attr_object_id: Option<&'a AtomicObjId>,
    pub(crate) name: &'a str,
    pub(crate) call_method: bool,
}

impl<'a> AttrArgs<'a> {
    pub fn new(
        attr_id: u64,
        father: ObjId,
        attr: Option<&'a AtomicObjId>,
        name: &'a str,
        call_method: bool,
    ) -> Box<Self> {
        Box::new(Self {
            attr_id,
            father,
            attr_object_id: attr,
            name,
            call_method,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ReferenceArgs<'a> {
    pub(crate) father: ObjId,
    pub(crate) atomic_usize: &'a AtomicObjId,
    pub(crate) call_method: bool,
}

#[derive(Debug)]
pub enum SValue<'a> {
    Stack(&'a (u64, String, bool)),
    Attr(Box<AttrArgs<'a>>), // father, attr, name, call_method
    Global(ObjId),
    Reference(Box<ReferenceArgs<'a>>), // Owner, ref, call_method
}

impl<'a> SValue<'a> {
    fn get_chains(
        &self,
        thread: &FSRThreadRuntime,
        state: &CallFrame<'_>,
        var: &(u64, String, bool),
    ) -> Option<ObjId> {
        let fn_id = state.fn_obj;
        if fn_id != 0 {
            let obj = FSRObject::id_to_obj(fn_id).as_fn();
            if let Some(s) = obj.get_closure_var(var.1.as_str()) {
                return Some(s);
            }
        }
        // let code = FSRObject::id_to_obj(state.code).as_code();
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

    #[inline(always)]
    pub fn get_global_id(&self, thread: &FSRThreadRuntime) -> Option<ObjId> {
        Some(match self {
            SValue::Stack(s) => {
                let state = thread.get_cur_frame();
                if let Some(id) = state.get_var(&s.0) {
                    id.load(Ordering::Relaxed)
                } else {
                    Self::get_chains(self, thread, state, s)?
                }
            }
            SValue::Global(id) => *id,
            SValue::Attr(args) => args.attr_object_id?.load(Ordering::Relaxed),
            SValue::Reference(refer) => refer.atomic_usize.load(Ordering::Relaxed),
        })
    }

    #[inline(always)]
    pub fn drop_box(self, allocator: &mut FSRObjectAllocator<'a>) {
        match self {
            Self::Attr(b) => {
                allocator.free_box_attr(b);
            }
            _ => {}
        }
    }
}

/// Context for bytecode, if fs code call from rust fn will create new context
pub struct FSCodeContext {
    // tracing call stack, is call stack is empty means end of this call except start of this call
    pub(crate) call_end: u32,
    // exp: Vec<SValue<'a>>,
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

    #[inline(always)]
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

pub struct ThreadLockerState {
    pub(crate) in_rt_cxt: Arc<(Mutex<bool>, Condvar)>,
    pub(crate) is_stop: Arc<(Mutex<bool>, Condvar)>,
}

impl ThreadLockerState {
    pub fn new() -> Self {
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
    pub fn new() -> Self {
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
    pub(crate) remembered_set: HashSet<ObjId>,
    pub(crate) op_quick: Box<Ops>,
    pub(crate) counter: usize,
    pub(crate) last_aquire_counter: usize,
    pub(crate) til: ThreadLockerState,
    pub(crate) thread_context_stack: Vec<Box<FSCodeContext>>,
    pub(crate) thread_context: Option<Box<FSCodeContext>>,
    pub(crate) gc_context: GcContext,
}

impl<'a> FSRThreadRuntime<'a> {
    pub fn get_vm(&self) -> Arc<FSRVM<'static>> {
        unsafe { VM.as_ref().unwrap().clone() }
    }

    // pub fn get_mut_vm(&mut self) -> &'a mut FSRVM<'a> {
    //     unsafe { &mut *self.vm_ptr.unwrap() }
    // }

    fn get_chains(
        thread: &FSRThreadRuntime,
        state: &CallFrame<'_>,
        var: &(u64, String, bool),
    ) -> Option<ObjId> {
        let fn_id = state.fn_obj;
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

    pub fn get_global_id(&self, value: &SValue<'a>) -> Result<ObjId, FSRError> {
        Ok(match value {
            SValue::Stack(s) => {
                let state = self.get_cur_frame();
                if let Some(id) = state.get_var(&s.0) {
                    id.load(Ordering::Relaxed)
                } else {
                    Self::get_chains(self, state, s).unwrap()
                }
            }
            SValue::Global(id) => *id,
            SValue::Attr(args) => args.attr_object_id.unwrap().load(Ordering::Relaxed),
            SValue::Reference(refer) => refer.atomic_usize.load(Ordering::Relaxed),
        })
    }

    pub fn new() -> FSRThreadRuntime<'a> {
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
            op_quick: Box::new(Ops::new_init()),
            counter: 0,
            til: ThreadLockerState::new(),
            last_aquire_counter: 0,
            thread_context_stack: Vec::with_capacity(8),
            thread_context: None,
            remembered_set: HashSet::new(),
            gc_context: GcContext::new(),
        }
    }

    pub fn add_object_to_remembered_set(&mut self, id: ObjId) {
        self.remembered_set.insert(id);
    }

    pub fn remove_object_from_remembered_set(&mut self, id: ObjId) {
        self.remembered_set.remove(&id);
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

    #[inline(always)]
    pub fn get_cur_mut_frame(&mut self) -> &mut CallFrame<'a> {
        &mut self.cur_frame
    }

    #[inline(always)]
    pub fn push_frame(&mut self, frame: Box<CallFrame<'a>>) {
        let old_frame = std::mem::replace(&mut self.cur_frame, frame);
        self.call_frames.push(old_frame);
    }

    #[inline(always)]
    pub fn pop_frame(&mut self) -> Box<CallFrame<'a>> {
        let v = self.call_frames.pop().unwrap();
        std::mem::replace(&mut self.cur_frame, v)
    }

    #[inline(always)]
    pub fn get_cur_mut_context(&mut self) -> &mut FSCodeContext {
        self.thread_context.as_mut().unwrap()
    }

    pub fn push_context(&mut self, context: Box<FSCodeContext>) {
        if self.thread_context.is_none() {
            self.thread_context = Some(context);
        } else {
            if let Some(s) = self.thread_context.take() {
                self.thread_context = Some(context);
                self.thread_context_stack.push(s);
            }
        }
    }

    pub fn pop_context(&mut self) -> Box<FSCodeContext> {
        if let Some(s) = self.thread_context.take() {
            // self.thread_context_stack.push(s);
            self.thread_context = self.thread_context_stack.pop();
            return s;
        }
        panic!("pop empty context");
        // let v = self.thread_context_stack.pop();
        // std::mem::replace(&mut self.thread_context, v)
    }

    #[inline(always)]
    pub fn get_context(&self) -> &FSCodeContext {
        self.thread_context.as_ref().unwrap()
    }

    #[inline(always)]
    pub fn get_cur_frame(&self) -> &CallFrame<'a> {
        &self.cur_frame
    }

    #[inline(always)]
    fn mark(&self, id: ObjId) -> Option<()> {
        let obj = FSRObject::id_to_mut_obj(id)?;
        obj.mark();
        Some(())
    }

    fn add_worklist(&self) -> Vec<ObjId> {
        let mut others = self.flow_tracker.for_iter_obj.clone();
        others.extend(self.flow_tracker.ref_for_obj.clone());
        let frames = &self.call_frames;
        let cur_frame = self.get_cur_frame();
        let mut work_list = Vec::with_capacity(16);
        for it in frames {
            for obj in it.var_map.iter() {
                work_list.push(obj.load(Ordering::Relaxed));
            }

            //if let Some(s) = &it.exp {
            for i in &it.exp {
                let id = match i.get_global_id(self) {
                    Some(id) => id,
                    None => {
                        continue;
                    }
                };
                work_list.push(id);
            }
            //}

            for value in &it.middle_value {
                work_list.push(*value);
            }

            if let Some(ret_val) = it.ret_val {
                work_list.push(ret_val);
            }

            if it.handling_exception != 0 {
                work_list.push(it.handling_exception);
            }
        }

        let it = cur_frame;
        for obj in it.var_map.iter() {
            work_list.push(obj.load(Ordering::Relaxed));
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

        for obj in it.exp.iter() {
            let id = match obj.get_global_id(self) {
                Some(id) => id,
                None => {
                    continue;
                }
            };
            work_list.push(id);
        }

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

    #[inline(always)]
    pub fn compare(
        left: ObjId,
        right: ObjId,
        op: CompareOperator,
        thread: &mut Self,
    ) -> Result<bool, FSRError> {
        let res = match op {
            CompareOperator::Greater => {
                let left_obj = FSRObject::id_to_obj(left);
                let right_obj = FSRObject::id_to_obj(right);

                if let Some(greater) = thread
                    .op_quick
                    .get_greater(right_obj.cls as ObjId, left_obj.cls as ObjId)
                {
                    greater(&[left, right], thread, thread.get_context().code)?
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
                let left_obj = FSRObject::id_to_obj(left);
                let right_obj = FSRObject::id_to_obj(right);

                if let Some(less) = thread
                    .op_quick
                    .get_less(right_obj.cls as ObjId, left_obj.cls as ObjId)
                {
                    less(&[left, right], thread, thread.get_context().code)?
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
            CompareOperator::Equal => {
                let left_obj = FSRObject::id_to_obj(left);
                let right_obj = FSRObject::id_to_obj(right);

                if let Some(equal) = thread
                    .op_quick
                    .get_equal(right_obj.cls as usize, left_obj.cls as usize)
                {
                    equal(&[left, right], thread, thread.get_context().code)?
                } else {
                    FSRObject::invoke_offset_method(
                        BinaryOffset::Equal,
                        &[left, right],
                        thread,
                        thread.get_context().code,
                    )?
                }
            }
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
        return Ok(id == FSRObject::true_id());
    }

    fn pop_stack(&mut self) {
        let v = self.pop_frame();
        self.frame_free_list.free(v);
    }

    fn getter_process(
        self: &mut FSRThreadRuntime<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let index_obj = self.get_cur_mut_frame().exp.pop().unwrap();

        let index_id = match &index_obj {
            SValue::Stack(s) => {
                let state = self.get_cur_mut_frame();
                state.get_var(&s.0).unwrap().load(Ordering::Relaxed)
            }
            SValue::Global(id) => *id,
            SValue::Attr(args) => args.attr_object_id.unwrap().load(Ordering::Relaxed),
            //SValue::BoxObject(obj) => FSRObject::obj_to_id(obj),
            SValue::Reference(refer) => refer.atomic_usize.load(Ordering::Relaxed),
        };

        let list_obj = self.get_cur_mut_frame().exp.pop().unwrap();
        let list_id = match &list_obj {
            SValue::Stack(s) => {
                let state = self.get_cur_mut_frame();
                state.get_var(&s.0).unwrap().load(Ordering::Relaxed)
            }
            SValue::Global(id) => *id,
            SValue::Attr(args) => args.attr_object_id.unwrap().load(Ordering::Relaxed),
            SValue::Reference(refer) => refer.atomic_usize.load(Ordering::Relaxed),
        };

        let index_obj_v = FSRObject::id_to_obj(index_id);
        let list_obj_v = FSRObject::id_to_obj(list_id);

        let res = if let Some(get_item) = self
            .op_quick
            .get_getter(list_obj_v.cls as ObjId, index_obj_v.cls as ObjId)
        {
            get_item(&[list_id, index_id], self, self.get_context().code)?
        } else {
            FSRObject::invoke_offset_method(
                BinaryOffset::GetItem,
                &[list_id, index_id],
                self,
                self.get_context().code,
            )?
        };

        // pop after finish invoke
        list_obj.drop_box(&mut self.thread_allocator);
        index_obj.drop_box(&mut self.thread_allocator);
        self.get_cur_mut_frame().middle_value.push(list_id);
        self.get_cur_mut_frame().middle_value.push(index_id);

        match res {
            FSRRetValue::GlobalId(res_id) => {
                self.get_cur_mut_frame().exp.push(SValue::Global(res_id));
            }
            FSRRetValue::Reference(atomic_usize) => {
                // self.get_cur_mut_frame()
                //     .exp
                //     .push(SValue::Reference(list_id, atomic_usize, false));
                self.get_cur_mut_frame()
                    .exp
                    .push(SValue::Reference(Box::new(ReferenceArgs {
                        father: list_id,
                        atomic_usize,
                        call_method: false,
                    })));
            }
        };

        Ok(false)
    }

    #[inline(always)]
    fn assign_process(
        self: &mut FSRThreadRuntime<'a>,
        bytecode: &'a BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        if let ArgType::Variable(v) = bytecode.get_arg() {
            let var_id = v.0;
            let svalue = match self.get_cur_mut_frame().exp.pop() {
                Some(s) => s,
                None => {
                    return Err(FSRError::new("", FSRErrCode::EmptyExpStack));
                }
            };

            let obj_id = svalue.get_global_id(self).unwrap();
            let state = &mut self.cur_frame;
            state.insert_var(var_id, obj_id);
            svalue.drop_box(&mut self.thread_allocator);
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

        let to_assign_obj_svalue = self.get_cur_mut_frame().exp.pop().unwrap();

        let to_assign_obj_id = match &to_assign_obj_svalue {
            SValue::Stack(s) => {
                let state = self.get_cur_frame();
                if let Some(id) = state.get_var(&s.0) {
                    id.load(Ordering::Relaxed)
                } else {
                    let module = FSRObject::id_to_obj(
                        FSRObject::id_to_obj(self.get_context().code)
                            .as_code()
                            .module,
                    )
                    .as_module();
                    let vm = self.get_vm();
                    let v = match module.get_object(&s.1) {
                        Some(s) => s.load(Ordering::Relaxed),
                        None => *vm.get_global_obj_by_name(&s.1).unwrap(),
                    };

                    v
                }
            }
            SValue::Global(id) => *id,
            SValue::Attr(args) => {
                let id = args.attr_object_id.unwrap().load(Ordering::Relaxed);
                //self.thread_allocator.free_box_attr(args);
                id
            }
            //SValue::BoxObject(fsrobject) => FSRVM::leak_object(fsrobject),
            SValue::Reference(refer) => refer.atomic_usize.load(Ordering::Relaxed),
        };

        match &assign_id {
            SValue::Stack(v) => {
                let state = &mut self.cur_frame;
                state.insert_var(v.0, to_assign_obj_id);
                state.attr_map.clear_var(v.0 as usize);
                //FSRObject::id_to_obj(context.module.unwrap()).as_module().register_object(name, fnto_a_id);
            }
            SValue::Attr(attr) => {
                let father_obj =
                    FSRObject::id_to_mut_obj(attr.father).expect("not a class instance");
                if father_obj.area.is_long()
                    && FSRObject::id_to_obj(to_assign_obj_id).area == Area::Minjor
                {
                    father_obj.set_write_barrier(true);
                }
                if let Some(s) = attr.attr_object_id {
                    s.store(to_assign_obj_id, Ordering::Relaxed);
                } else {
                    father_obj.set_attr(attr.name, to_assign_obj_id);
                }

                //self.thread_allocator.free_box_attr(attr);
            }
            SValue::Global(_) => todo!(),
            SValue::Reference(ref refer) => {
                let owner = FSRObject::id_to_obj(refer.father);
                if owner.area.is_long()
                    && FSRObject::id_to_obj(to_assign_obj_id).area == Area::Minjor
                {
                    owner.set_write_barrier(true);
                }
                refer
                    .atomic_usize
                    .store(to_assign_obj_id, Ordering::Relaxed);
            }
        }

        self.get_cur_mut_frame().middle_value.push(to_assign_obj_id);
        // self.get_cur_mut_frame().middle_value.push(assign_id);

        Ok(false)
    }

    #[inline(always)]
    fn binary_add_process(
        self: &mut FSRThreadRuntime<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let v1 = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary add 1",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let v2 = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary add 2",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let v1_id = v1.get_global_id(self).unwrap();
        let v2_id = v2.get_global_id(self).unwrap();

        let v1_cls = obj_cls!(v1_id);
        let v2_cls = obj_cls!(v2_id);
        if let Some(op_quick) = self.op_quick.get_add(v1_cls, v2_cls) {
            let res = op_quick(&[v2_id, v1_id], self, self.get_context().code)?;
            v1.drop_box(&mut self.thread_allocator);
            v2.drop_box(&mut self.thread_allocator);

            match res {
                FSRRetValue::GlobalId(res_id) => {
                    self.get_cur_mut_frame().exp.push(SValue::Global(res_id));
                }
                FSRRetValue::Reference(_) => {
                    panic!("not support reference return, in add process")
                }
            };

            return Ok(false);
        }

        let res = FSRObject::invoke_binary_method(
            BinaryOffset::Add,
            v2_id,
            v1_id,
            self,
            self.get_context().code,
        )?;

        v1.drop_box(&mut self.thread_allocator);
        v2.drop_box(&mut self.thread_allocator);
        self.get_cur_mut_frame().middle_value.push(v1_id);
        self.get_cur_mut_frame().middle_value.push(v2_id);

        match res {
            FSRRetValue::GlobalId(res_id) => {
                self.get_cur_mut_frame().exp.push(SValue::Global(res_id));
            }
            FSRRetValue::Reference(_) => {
                panic!("not support reference return, in add process")
            }
        };

        Ok(false)
    }

    #[inline(always)]
    fn binary_sub_process(
        self: &mut FSRThreadRuntime<'a>,

        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let right_value = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary sub 1",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let left_value = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary sub 2",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let right = right_value.get_global_id(self).unwrap();
        let left = left_value.get_global_id(self).unwrap();
        let res = FSRObject::invoke_binary_method(
            BinaryOffset::Sub,
            left,
            right,
            self,
            self.get_context().code,
        )?;
        right_value.drop_box(&mut self.thread_allocator);
        left_value.drop_box(&mut self.thread_allocator);
        self.get_cur_mut_frame().middle_value.push(right);
        self.get_cur_mut_frame().middle_value.push(left);
        match res {
            FSRRetValue::GlobalId(res_id) => {
                self.get_cur_mut_frame().exp.push(SValue::Global(res_id));
            }
            FSRRetValue::Reference(_) => {
                panic!("not support reference return, in sub process")
            }
        };

        Ok(false)
    }

    #[inline(always)]
    fn binary_mul_process(
        self: &mut FSRThreadRuntime<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let right_value = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary mul 1",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let left_value = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary mul 2",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let right_id = right_value.get_global_id(self).unwrap();
        let left_id = left_value.get_global_id(self).unwrap();

        let res = FSRObject::invoke_binary_method(
            BinaryOffset::Mul,
            left_id,
            right_id,
            self,
            self.get_context().code,
        )?;

        right_value.drop_box(&mut self.thread_allocator);
        left_value.drop_box(&mut self.thread_allocator);
        self.get_cur_mut_frame().middle_value.push(right_id);
        self.get_cur_mut_frame().middle_value.push(left_id);
        match res {
            // FSRRetValue::Value(object) => {
            //     context.exp.push(SValue::BoxObject(object));
            // }
            FSRRetValue::GlobalId(res_id) => {
                self.get_cur_mut_frame().exp.push(SValue::Global(res_id));
            }
            FSRRetValue::Reference(_) => {
                panic!("not support reference return, in mul process")
            }
        };
        Ok(false)
    }

    #[inline(always)]
    fn binary_div_process(
        self: &mut FSRThreadRuntime<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let right_value = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary mul 1",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let left_value = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary mul 2",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let right_id = right_value.get_global_id(self).unwrap();
        let left_id = left_value.get_global_id(self).unwrap();

        let res = FSRObject::invoke_binary_method(
            BinaryOffset::Div,
            left_id,
            right_id,
            self,
            self.get_context().code,
        )?;
        match res {
            // FSRRetValue::Value(object) => {
            //     context.exp.push(SValue::BoxObject(object));
            // }
            FSRRetValue::GlobalId(res_id) => {
                self.get_cur_mut_frame().exp.push(SValue::Global(res_id));
            }
            FSRRetValue::Reference(_) => {
                panic!("not support reference return, in div process")
            }
        };

        right_value.drop_box(&mut self.thread_allocator);
        left_value.drop_box(&mut self.thread_allocator);
        self.get_cur_mut_frame().middle_value.push(right_id);
        self.get_cur_mut_frame().middle_value.push(left_id);
        Ok(false)
    }

    #[inline(always)]
    fn binary_reminder_process(
        self: &mut FSRThreadRuntime<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let right_value = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary mul 1",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let left_value = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary mul 2",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let right_id = right_value.get_global_id(self).unwrap();
        let left_id = left_value.get_global_id(self).unwrap();

        let left_cls = obj_cls!(left_id);
        let right_cls = obj_cls!(right_id);
        if let Some(op_quick) = self.op_quick.get_reminder(left_cls, right_cls) {
            let res = op_quick(&[left_id, right_id], self, self.get_context().code)?;
            left_value.drop_box(&mut self.thread_allocator);
            right_value.drop_box(&mut self.thread_allocator);
            self.get_cur_mut_frame().middle_value.push(right_id);
            self.get_cur_mut_frame().middle_value.push(left_id);
            match res {
                FSRRetValue::GlobalId(res_id) => {
                    self.get_cur_mut_frame().exp.push(SValue::Global(res_id));
                }
                FSRRetValue::Reference(_) => {
                    panic!("not support reference return, in reminder process")
                }
            };

            return Ok(false);
        }

        let res = FSRObject::invoke_binary_method(
            BinaryOffset::Reminder,
            left_id,
            right_id,
            self,
            self.get_context().code,
        )?;
        match res {
            // FSRRetValue::Value(object) => {
            //     context.exp.push(SValue::BoxObject(object));
            // }
            FSRRetValue::GlobalId(res_id) => {
                self.get_cur_mut_frame().exp.push(SValue::Global(res_id));
            }
            FSRRetValue::Reference(_) => {
                panic!("not support reference return, in div process")
            }
        };

        right_value.drop_box(&mut self.thread_allocator);
        left_value.drop_box(&mut self.thread_allocator);
        self.get_cur_mut_frame().middle_value.push(right_id);
        self.get_cur_mut_frame().middle_value.push(left_id);
        Ok(false)
    }

    fn binary_dot_process(
        self: &mut FSRThreadRuntime<'a>,

        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let attr_id = match self.get_cur_mut_frame().exp.pop().unwrap() {
            SValue::Stack(_) => unimplemented!(),
            SValue::Global(_) => unimplemented!(),
            SValue::Attr(id) => id,
            SValue::Reference(_) => todo!(),
        };
        let dot_father_svalue = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in dot operator",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let dot_father = dot_father_svalue.get_global_id(self).unwrap();
        let dot_father_obj = FSRObject::id_to_obj(dot_father);
        if dot_father_obj.is_code() {
            let name = attr_id.name;
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
            self.get_cur_mut_frame()
                .exp
                .push(SValue::Reference(Box::new(ReferenceArgs {
                    father: dot_father,
                    atomic_usize: id,
                    call_method: false,
                })));
            self.get_cur_mut_frame().middle_value.push(dot_father);
            self.get_cur_mut_frame()
                .middle_value
                .push(id.load(Ordering::Relaxed));
            self.thread_allocator.free_box_attr(attr_id);
            return Ok(false);
        }

        let name = attr_id.name;

        let id = if let SValue::Stack(s) = dot_father_svalue {
            let state = self.get_cur_mut_frame();
            if let Some(s) = state.get_attr(s.0 as usize, attr_id.attr_id as usize) {
                Some(s)
            } else {
                dot_father_obj.get_attr(name)
            }
        } else {
            dot_father_obj.get_attr(name)
        };
        if let Some(id) = id {
            let new_attr = self.thread_allocator.new_box_attr(
                attr_id.attr_id,
                dot_father,
                Some(id),
                name,
                true,
            );
            self.get_cur_mut_frame().exp.push(SValue::Attr(new_attr));
            if let SValue::Stack(s) = dot_father_svalue {
                let state = self.get_cur_mut_frame();
                state
                    .attr_map
                    .insert(s.0 as usize, attr_id.attr_id as usize, Some(id));
            }
            self.get_cur_mut_frame().middle_value.push(dot_father);
            self.get_cur_mut_frame()
                .middle_value
                .push(id.load(Ordering::Relaxed));
        } else {
            //context.exp.push(SValue::Global(dot_father));
            let new_attr = self.thread_allocator.new_box_attr(
                attr_id.attr_id,
                dot_father,
                attr_id.attr_object_id,
                name,
                true,
            );
            self.get_cur_mut_frame().exp.push(SValue::Attr(new_attr));
            if let SValue::Stack(s) = dot_father_svalue {
                let state = self.get_cur_mut_frame();
                state.attr_map.insert(
                    s.0 as usize,
                    attr_id.attr_id as usize,
                    attr_id.attr_object_id,
                );
            }

            self.get_cur_mut_frame().middle_value.push(dot_father);
        }

        // self.get_cur_mut_frame().middle_value.push(SValue::Attr(attr_id).get_global_id(self).unwrap());
        // self.get_cur_mut_frame().middle_value.push(dot_father_svalue.get_global_id(self).unwrap());

        Ok(false)
    }

    fn binary_get_cls_attr_process(
        self: &mut FSRThreadRuntime<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let attr_id = match self.get_cur_mut_frame().exp.pop().unwrap() {
            SValue::Stack(_) => unimplemented!(),
            SValue::Global(_) => unimplemented!(),
            SValue::Attr(id) => id,
            SValue::Reference(_) => todo!(),
        };

        let dot_father = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s.get_global_id(self).unwrap(),
            None => {
                return Err(FSRError::new(
                    "error in dot operator",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let dot_father_obj = FSRObject::id_to_obj(dot_father);

        let name = attr_id.name;
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
            // self.get_cur_mut_frame()
            //     .exp
            //     .push(SValue::Reference(dot_father, id, false));
            self.get_cur_mut_frame()
                .exp
                .push(SValue::Reference(Box::new(ReferenceArgs {
                    father: dot_father,
                    atomic_usize: id,
                    call_method: false,
                })));
            self.thread_allocator.free_box_attr(attr_id);
            return Ok(false);
        }
        if let Some(id) = id {
            let new_attr = self.thread_allocator.new_box_attr(
                attr_id.attr_id,
                dot_father,
                Some(id),
                name,
                false,
            );
            self.get_cur_mut_frame().exp.push(SValue::Attr(new_attr));
            self.get_cur_mut_frame().middle_value.push(dot_father);
            self.get_cur_mut_frame()
                .middle_value
                .push(id.load(Ordering::Relaxed));
        } else {
            //context.exp.push(SValue::Global(dot_father));
            let new_attr = self.thread_allocator.new_box_attr(
                attr_id.attr_id,
                dot_father,
                attr_id.attr_object_id,
                name,
                false,
            );
            self.get_cur_mut_frame().exp.push(SValue::Attr(new_attr));
            self.get_cur_mut_frame().middle_value.push(dot_father);
        }

        Ok(false)
    }

    fn binary_range_process(self: &mut FSRThreadRuntime<'a>) -> Result<bool, FSRError> {
        let rhs = match self.get_cur_mut_frame().exp.pop() {
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

        let lhs = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary mul 2",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let lhs_id = lhs.get_global_id(self).unwrap();
        let rhs_id = rhs.get_global_id(self).unwrap();

        let start = FSRObject::id_to_obj(lhs_id);
        let end = FSRObject::id_to_obj(rhs_id);

        if let FSRValue::Integer(start) = start.value {
            if let FSRValue::Integer(end) = end.value {
                let range = FSRRange {
                    range: Range { start, end },
                };

                let obj = self.thread_allocator.new_object(
                    FSRValue::Range(Box::new(range)),
                    FSRGlobalObjId::RangeCls as ObjId,
                );

                let id = FSRVM::leak_object(obj);

                self.get_cur_mut_frame().exp.push(SValue::Global(id));
                rhs.drop_box(&mut self.thread_allocator);
                lhs.drop_box(&mut self.thread_allocator);
                self.get_cur_mut_frame().middle_value.push(rhs_id);
                self.get_cur_mut_frame().middle_value.push(lhs_id);
                return Ok(false);
            }
        }
        unimplemented!()
    }

    #[inline(always)]
    fn chain_get_variable(var: &(u64, String, bool), thread: &Self, code: ObjId) -> Option<ObjId> {
        if let Some(value) = thread.get_cur_frame().get_var(&var.0) {
            Some(value.load(Ordering::Relaxed))
        } else if let Some(value) =
            FSRObject::id_to_obj(FSRObject::id_to_obj(code).as_code().module)
                .as_module()
                .get_object(&var.1)
        {
            Some(value.load(Ordering::Relaxed))
        } else {
            thread.get_vm().get_global_obj_by_name(&var.1).copied()
        }
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
            let arg = thread.get_cur_mut_frame().exp.pop().unwrap();
            let a_id = match &arg {
                SValue::Stack(s) => match Self::chain_get_variable(s, thread, code) {
                    Some(s) => s,
                    None => {
                        return Err(FSRError::new(
                            format!("not found variable in set args: `{}`", s.1),
                            FSRErrCode::NoSuchObject,
                        ))
                    }
                },
                SValue::Global(g) => *g,
                SValue::Attr(a) => a.attr_object_id.unwrap().load(Ordering::Relaxed),
                SValue::Reference(ref refer) => refer.atomic_usize.load(Ordering::Relaxed),
            };
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

    #[inline(always)]
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
                self.get_cur_mut_frame().exp.push(SValue::Global(self_id));

                return Ok(false);
            }
        }

        let fn_obj = FSRObject::id_to_obj(cls_id);
        let self_id = self.garbage_collect.new_object(
            FSRValue::ClassInst(Box::new(FSRClassInst::new(fn_obj.get_fsr_class_name()))),
            cls_id,
        );

        args.insert(0, self_id);

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

            for arg in args.iter().rev() {
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

    #[inline(always)]
    fn process_fn_is_attr(
        self: &mut FSRThreadRuntime<'a>,
        obj_id: ObjId,
        fn_obj: &'a FSRObject<'a>,
        args: &mut SmallVec<[usize; 4]>,
    ) -> Result<bool, FSRError> {
        // let obj_id = context.exp.pop().unwrap().get_global_id(self).unwrap();

        args.insert(0, obj_id);

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

            for arg in args.iter().rev() {
                self.get_cur_mut_frame().args.push(*arg);
            }
            let offset = fn_obj.get_fsr_offset().1;
            if let FSRValue::Function(obj) = &fn_obj.value {
                self.get_cur_mut_context().code = obj.code;
            }
            self.get_cur_mut_context().ip = (offset.0, 0);
            return Ok(true);
        } else {
            let v = fn_obj
                .call(
                    args,
                    self,
                    self.get_context().code,
                    FSRObject::obj_to_id(fn_obj),
                )
                .unwrap();

            let id = v.get_id();
            self.get_cur_mut_frame().exp.push(SValue::Global(id));
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

    #[inline(always)]
    fn process_fsr_fn(
        &mut self,
        fn_id: ObjId,
        fn_obj: &FSRObject<'a>,
        call_method: bool,
        args: &mut SmallVec<[ObjId; 4]>,
    ) -> Result<(), FSRError> {
        self.get_cur_mut_context().call_end += 1;
        self.save_ip_to_callstate();
        let f = fn_obj.as_fn();
        let frame = self.frame_free_list.new_frame(f.code, fn_id);
        self.push_frame(frame);

        if call_method {
            let self_obj = match self.get_cur_mut_frame().exp.pop() {
                Some(s) => s.get_global_id(self).unwrap(),
                None => {
                    return Err(FSRError::new(
                        "Failed to retrieve self object in call_process",
                        FSRErrCode::EmptyExpStack,
                    ));
                }
            };
            self.get_cur_mut_frame().args.push(self_obj);
            self.get_cur_mut_frame().middle_value.push(self_obj);
        }

        for arg in args.iter().rev() {
            self.get_cur_mut_frame().args.push(*arg);
        }
        //let offset = fn_obj.get_fsr_offset();
        let offset = fn_obj.get_fsr_offset().1;
        if let FSRValue::Function(obj) = &fn_obj.value {
            self.get_cur_mut_context().code = obj.code;
        }

        self.get_cur_mut_context().ip = (offset.0, 0);
        Ok(())
    }

    #[inline(always)]
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

    #[inline(always)]
    fn get_call_fn_id(
        &mut self,
        var: &Option<&(usize, u64, String, bool)>,
        module: &FSRModule,
        object_id: &mut Option<ObjId>,
    ) -> Result<(ObjId, bool), FSRError> {
        let mut call_method = false;
        if let Some(var) = var {
            let var_id = var.1;
            let fn_id = self.try_get_obj_by_name(var.1, &var.2, module).unwrap();
            Ok((fn_id, call_method))
        } else {
            let fn_svalue = self.get_cur_mut_frame().exp.pop().unwrap();
            let fn_id = match &fn_svalue {
                SValue::Stack(s) => self.try_get_obj_by_name(s.0, &s.1, module).unwrap(),
                SValue::Global(id) => *id,
                SValue::Attr(attr) => {
                    call_method = attr.call_method;

                    let id = if !call_method {
                        let cls_obj = FSRObject::id_to_obj(attr.father).as_class();
                        cls_obj.get_attr(attr.name).unwrap().load(Ordering::Relaxed)
                    } else {
                        *object_id = Some(attr.father);
                        attr.attr_object_id.unwrap().load(Ordering::Relaxed)
                    };

                    //self.thread_allocator.free_box_attr(attr);

                    id
                }
                SValue::Reference(ref refer) => {
                    call_method = refer.call_method;

                    refer.atomic_usize.load(Ordering::Relaxed)
                }
            };
            self.get_cur_mut_frame().middle_value.push(fn_id);
            Ok((fn_id, call_method))
        }
    }

    #[inline(always)]
    fn call_process_ret(
        &mut self,
        fn_id: ObjId,
        args: &mut SmallVec<[ObjId; 4]>,
        object_id: &Option<ObjId>,
        call_method: bool,
    ) -> Result<bool, FSRError> {
        let fn_obj = FSRObject::id_to_obj(fn_id);
        if fn_obj.is_fsr_cls() {
            let v = Self::process_fsr_cls(self, fn_id, args)?;
            if v {
                return Ok(v);
            }
        } else if object_id.is_some() && call_method {
            let v = Self::process_fn_is_attr(self, object_id.unwrap(), fn_obj, args)?;
            if v {
                return Ok(v);
            }
        } else if fn_obj.is_fsr_function() {
            self.process_fsr_fn(fn_id, fn_obj, call_method, args)?;
            return Ok(true);
        } else {
            let v = match fn_obj.call(&args, self, self.get_context().code, fn_id) {
                Ok(o) => o,
                Err(e) => {
                    if e.inner.code == FSRErrCode::RuntimeError {
                        self.exception = e.inner.exception.unwrap();
                        return Ok(false);
                    }

                    panic!()
                }
            };

            let id = v.get_id();
            self.get_cur_mut_frame().exp.push(SValue::Global(id));
        }

        Ok(false)
    }

    #[inline(always)]
    fn call_process(
        self: &mut FSRThreadRuntime<'a>,
        bytecode: &'a BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let mut var: Option<&(usize, u64, String, bool)> = None;
        let mut args = self.get_fn_args(&mut var, bytecode.get_arg())?;

        let mut object_id: Option<ObjId> = None;
        // let module = FSRObject::id_to_obj(self.get_context().code).as_code();
        let module = FSRObject::id_to_obj(
            FSRObject::id_to_obj(self.get_context().code)
                .as_code()
                .module,
        )
        .as_module();
        let (fn_id, call_method) = self.get_call_fn_id(&var, module, &mut object_id)?;

        self.call_process_ret(fn_id, &mut args, &object_id, call_method)
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
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let v = self.get_cur_mut_frame().exp.pop().unwrap();
        let mut name = "";
        let test_val = match &v {
            SValue::Stack(s) => {
                name = &s.1;
                // let module = FSRObject::id_to_obj(self.get_context().code).as_code();
                let module = FSRObject::id_to_obj(
                    FSRObject::id_to_obj(self.get_context().code)
                        .as_code()
                        .module,
                )
                .as_module();
                self.try_get_obj_by_name(s.0, name, module)
            }
            SValue::Global(id) => Some(*id),
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
                let tmp = self.get_context().ip.0;
                self.get_cur_mut_context().ip = (tmp + n.0 as usize + 1_usize, 0);
                self.flow_tracker.push_last_if_test(false);
                v.drop_box(&mut self.thread_allocator);
                return Ok(true);
            }
        }

        v.drop_box(&mut self.thread_allocator);
        self.get_cur_mut_frame().middle_value.push(test_val);
        self.flow_tracker.push_last_if_test(true);
        Ok(false)
    }

    #[inline(always)]
    fn if_end(
        self: &mut FSRThreadRuntime<'a>,

        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        self.flow_tracker.pop_last_if_test();
        Ok(false)
    }

    #[inline(always)]
    fn else_if_test_process(
        self: &mut FSRThreadRuntime<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let test_svalue = self.get_cur_mut_frame().exp.pop().unwrap();
        let test_val = match &test_svalue {
            SValue::Stack(s) => {
                let state = self.get_cur_mut_frame();
                if let Some(id) = state.get_var(&s.0) {
                    id.load(Ordering::Relaxed)
                } else {
                    return Err(FSRError::new(
                        format!("Not found variable in else if test process `{}`", s.1),
                        FSRErrCode::NoSuchObject,
                    ));
                }
            }
            SValue::Global(id) => *id,
            _ => {
                return Err(FSRError::new(
                    "Not a valid test object",
                    FSRErrCode::NotValidArgs,
                ))
            }
        };
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

    #[inline(always)]
    fn else_process(
        self: &mut FSRThreadRuntime<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
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

    #[inline(always)]
    fn else_if_match(
        self: &mut FSRThreadRuntime<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
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

    #[inline(always)]
    fn break_process(
        self: &mut FSRThreadRuntime<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        self.flow_tracker.is_break = true;
        let l = self.flow_tracker.continue_line.len();
        let continue_line = self.flow_tracker.continue_line[l - 1];
        self.get_cur_mut_context().ip = (continue_line, 0);
        Ok(true)
    }

    #[inline(always)]
    fn continue_process(
        self: &mut FSRThreadRuntime<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let l = self.flow_tracker.continue_line.len();
        let continue_line = self.flow_tracker.continue_line[l - 1];
        self.get_cur_mut_context().ip = (continue_line, 0);
        Ok(true)
    }

    // save will fix
    fn for_block_ref(self: &mut FSRThreadRuntime<'a>) -> Result<bool, FSRError> {
        let obj_id = {
            let obj = self.get_cur_frame().exp.last().unwrap();

            let obj_id = self.get_global_id(obj)?;
            obj_id
        };

        self.flow_tracker.ref_for_obj.push(obj_id);
        Ok(false)
    }

    #[inline(always)]
    fn load_for_iter(
        self: &mut FSRThreadRuntime<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let iter_obj = self.get_cur_mut_frame().exp.pop().unwrap();
        let iter_id = {
            let id = iter_obj.get_global_id(self).unwrap();
            //FSRObject::id_to_obj(id).ref_add();
            id
        };

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

    #[inline(always)]
    fn while_test_process(
        self: &mut FSRThreadRuntime<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let test_svalue = self.get_cur_mut_frame().exp.pop().unwrap();
        let test_val = match test_svalue {
            SValue::Stack(s) => {
                let state = self.get_cur_mut_frame();
                if let Some(id) = state.get_var(&s.0) {
                    id.load(Ordering::Relaxed)
                } else {
                    return Err(FSRError::new(
                        format!("Not found variable in while test process `{}`", s.1),
                        FSRErrCode::NoSuchObject,
                    ));
                }
            }
            SValue::Global(id) => id,
            _ => {
                unimplemented!()
            }
        };
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
        bytecode: &BytecodeArg,
        bc: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let name = match self.get_cur_mut_frame().exp.pop().unwrap() {
            SValue::Stack(id) => id,
            SValue::Attr(_) => panic!(),
            SValue::Global(_) => panic!(),
            // SValue::BoxObject(_) => todo!(),
            SValue::Reference(_) => todo!(),
        };

        if let ArgType::DefineFnArgs(n, arg_len, fn_identify_name) = bytecode.get_arg() {
            let mut args = vec![];
            for _ in 0..*arg_len {
                let v = match self.get_cur_mut_frame().exp.pop().unwrap() {
                    SValue::Stack(id) => id,
                    _ => panic!("not support args value"),
                };
                args.push(v.1.to_string());
            }

            //println!("define_fn: {}", FSRObject::id_to_obj(context.module.unwrap()).as_module().as_string());
            let module_id = FSRObject::id_to_obj(self.get_context().code).as_code().module;
            let module = FSRObject::id_to_obj(module_id).as_module();
            let fn_code = module.get_fn(&fn_identify_name).unwrap();
            let fn_code_id = FSRObject::obj_to_id(fn_code);
            let fn_obj = FSRFn::from_fsr_fn(
                &name.1,
                (0, 0),
                args,
                bc,
                fn_code_id,
                self.get_cur_frame().fn_obj,
            );

            let fn_obj = self
                .thread_allocator
                .new_object(fn_obj, FSRGlobalObjId::FnCls as ObjId);
            let fn_id = FSRVM::leak_object(fn_obj);
            let state = &mut self.cur_frame;
            if let Some(cur_cls) = &mut state.cur_cls {
                let offset = BinaryOffset::from_alias_name(name.1.as_str());
                if let Some(offset) = offset {
                    cur_cls.insert_offset_attr_obj_id(offset, fn_id);
                    self.get_cur_mut_context().ip = (self.get_context().ip.0 + 1, 0);
                    return Ok(true);
                }
                cur_cls.insert_attr_id(&name.1, fn_id);
                self.get_cur_mut_context().ip = (self.get_context().ip.0 + 1, 0);
                return Ok(true);
            }

            state.insert_var(name.0, fn_id);
            let define_fn_obj = self.get_cur_frame().fn_obj;
            if define_fn_obj == FSRObject::none_id() {
                // FSRObject::id_to_mut_obj(self.get_context().code)
                //     .expect("not a code object")
                //     .as_mut_code()
                //     .register_object(&name.1, fn_id);
                // self.get_mut_module(self.get_context().code).register_object(&name.1, fn_id);
                let module = FSRObject::id_to_mut_obj(
                    FSRObject::id_to_obj(self.get_context().code)
                        .as_code()
                        .module,
                )
                .unwrap()
                .as_mut_module();
                module.register_object(&name.1, fn_id);
            }
            if name.2 {
                let define_fn_obj = self.get_cur_frame().fn_obj;
                if define_fn_obj == FSRObject::none_id() {
                    panic!("closure var must in closure");
                }
                let define_fn_obj = FSRObject::id_to_mut_obj(define_fn_obj)
                    .expect("not a fn obj")
                    .as_mut_fn();
                if let Some(s) = define_fn_obj.store_cells.get(name.1.as_str()) {
                    s.store(fn_id, Ordering::Relaxed);
                } else {
                    define_fn_obj
                        .store_cells
                        .insert(name.1.as_str(), AtomicObjId::new(fn_id));
                }
            }

            let ip_0 = self.get_context().ip.0;
            self.get_cur_mut_context().ip = (ip_0 + 1, 0);
            return Ok(true);
        }
        Ok(false)
    }

    #[inline(always)]
    fn compare_test(
        self: &mut FSRThreadRuntime<'a>,
        bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        if let ArgType::Compare(op) = bytecode.get_arg() {
            let right = self.get_cur_mut_frame().exp.pop().ok_or_else(|| {
                FSRError::new(
                    "Failed to pop right operand from stack in compare_test",
                    FSRErrCode::EmptyExpStack,
                )
            })?;
            let left = self.get_cur_mut_frame().exp.pop().ok_or_else(|| {
                FSRError::new(
                    "Failed to pop left operand from stack in compare_test",
                    FSRErrCode::EmptyExpStack,
                )
            })?;

            let right_id = right.get_global_id(self).unwrap();
            let left_id = left.get_global_id(self).unwrap();

            let v = Self::compare(left_id, right_id, *op, self)?;

            left.drop_box(&mut self.thread_allocator);
            right.drop_box(&mut self.thread_allocator);
            self.get_cur_mut_frame().middle_value.push(right_id);
            self.get_cur_mut_frame().middle_value.push(left_id);
            if v {
                self.get_cur_mut_frame()
                    .exp
                    .push(SValue::Global(FSRObject::true_id()))
            } else {
                self.get_cur_mut_frame()
                    .exp
                    .push(SValue::Global(FSRObject::false_id()))
            }
        } else {
            return Err(FSRError::new(
                "not a compare test",
                FSRErrCode::NotValidArgs,
            ));
        }

        Ok(false)
    }

    fn ret_value(
        self: &mut FSRThreadRuntime<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let v = if self.get_cur_mut_frame().exp.is_empty() {
            FSRObject::none_id()
        } else {
            match self.get_cur_mut_frame().exp.pop().unwrap() {
                SValue::Stack(s) => {
                    let state = self.get_cur_frame();
                    if let Some(id) = state.get_var(&s.0) {
                        id.load(Ordering::Relaxed)
                    } else {
                        //let code = FSRObject::id_to_obj(state.code).as_code();
                        // let module = self.get_module(state.code);
                        let module = FSRObject::id_to_obj(
                            FSRObject::id_to_obj(self.get_context().code)
                                .as_code()
                                .module,
                        )
                        .as_module();
                        let vm = self.get_vm();
                        let v = match module.get_object(&s.1) {
                            Some(s) => s.load(Ordering::Relaxed),
                            None => *vm.get_global_obj_by_name(&s.1).ok_or_else(|| {
                                FSRError::new(
                                    format!("not found object in test: {}", s.1),
                                    FSRErrCode::NoSuchObject,
                                )
                            })?,
                        };
                        v
                    }
                }
                SValue::Global(id) => id,
                SValue::Attr(args) => {
                    let id = args.attr_object_id.unwrap().load(Ordering::Relaxed);
                    self.thread_allocator.free_box_attr(args);
                    id
                }
                // SValue::BoxObject(obj) => FSRVM::leak_object(obj),
                SValue::Reference(ref refer) => refer.atomic_usize.load(Ordering::Relaxed),
            }
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

    fn end_define_fn(
        self: &mut FSRThreadRuntime<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
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
        _: &'a Bytecode,
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
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        if let ArgType::WhileEnd(n) = bytecode.get_arg() {
            let tmp = self.get_context().ip.0;
            self.get_cur_mut_context().ip = (tmp - *n as usize, 0);
            return Ok(true);
        }

        Ok(false)
    }

    fn load_closure(&mut self, closure: &(u64, String)) -> Result<(), FSRError> {
        let svalue = match self.get_cur_mut_frame().exp.pop() {
            Some(s) => s,
            None => {
                return Err(FSRError::new("", FSRErrCode::EmptyExpStack));
            }
        };

        let obj_id = svalue.get_global_id(self).unwrap();
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
        _: &'a Bytecode,
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
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        if let ArgType::LoadListNumber(n) = bytecode.get_arg() {
            let mut list = Vec::with_capacity(*n);
            let n = *n;
            for _ in 0..n {
                let v = self.get_cur_mut_frame().exp.pop().ok_or_else(|| {
                    FSRError::new(
                        "Failed to pop value from stack in load_list",
                        FSRErrCode::EmptyExpStack,
                    )
                })?;
                let v_id = v.get_global_id(self).unwrap();

                list.push(v_id);
                self.get_cur_mut_frame().middle_value.push(v_id);
            }

            let list = self
                .garbage_collect
                .new_object(FSRList::new_value(list), FSRGlobalObjId::ListCls as ObjId);
            self.get_cur_mut_frame().exp.push(SValue::Global(list));
        }

        Ok(false)
    }

    fn class_def(
        self: &mut FSRThreadRuntime<'a>,
        _bytecode: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let id = match self.get_cur_mut_frame().exp.pop().ok_or_else(|| {
            FSRError::new(
                "Failed to pop class id from stack in class_def",
                FSRErrCode::EmptyExpStack,
            )
        })? {
            SValue::Stack(i) => i,
            SValue::Attr(_) => panic!(),
            SValue::Global(_) => panic!(),
            //SValue::BoxObject(_) => todo!(),
            SValue::Reference(_) => todo!(),
        };

        let new_cls = FSRClass::new(&id.1);
        let state = self.get_cur_mut_frame();
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

    fn end_class_def(
        self: &mut FSRThreadRuntime<'a>,
        bc: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        if let ArgType::Variable(var) = bc.get_arg() {
            let id = var.0;
            let state = self.get_cur_mut_frame();
            let mut cls_obj = FSRObject::new();
            cls_obj.set_cls(FSRGlobalObjId::ClassCls as ObjId);
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

    #[inline(always)]
    fn special_load_for(
        self: &mut FSRThreadRuntime<'a>,
        arg: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let obj = self.flow_tracker.for_iter_obj.last().cloned().unwrap();
        let obj_value = FSRObject::id_to_obj(obj);
        let res = if obj_value.cls == FSRGlobalObjId::InnerIterator as ObjId {
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
        if res_id == 0 || self.flow_tracker.is_break {
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
        self.get_cur_mut_frame().exp.push(SValue::Global(res_id));
        Ok(false)
    }

    fn process_logic_and(
        self: &mut FSRThreadRuntime<'a>,
        bc: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let first = self
            .get_cur_mut_frame()
            .exp
            .pop()
            .unwrap()
            .get_global_id(self)
            .unwrap();
        if first == 0 || first == 2 {
            if let ArgType::AddOffset(offset) = bc.get_arg() {
                self.get_cur_mut_context().ip.1 += *offset;
                self.get_cur_mut_frame().exp.push(SValue::Global(2));
            }
        }

        Ok(false)
    }

    // process logic or operator in bytecode
    fn process_logic_or(
        self: &mut FSRThreadRuntime<'a>,
        bc: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let first = self
            .get_cur_mut_frame()
            .exp
            .pop()
            .unwrap()
            .get_global_id(self)
            .unwrap();
        if first != 0 && first != 2 {
            if let ArgType::AddOffset(offset) = bc.get_arg() {
                self.get_cur_mut_context().ip.1 += *offset;
                self.get_cur_mut_frame().exp.push(SValue::Global(1));
            }
        }

        Ok(false)
    }

    fn not_process(
        self: &mut FSRThreadRuntime<'a>,
        _bc: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        let v1 = match self.get_cur_frame().exp.last() {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    "error in binary add 1",
                    FSRErrCode::EmptyExpStack,
                ));
            }
        };

        let v1_id = self.get_global_id(v1)?;
        // let mut target = false;
        let target = FSRObject::none_id() == v1_id || FSRObject::false_id() == v1_id;

        if let Some(x) = self.get_cur_mut_frame().exp.pop() {
            x.drop_box(&mut self.thread_allocator)
        }

        self.get_cur_mut_frame().middle_value.push(v1_id);

        if target {
            self.get_cur_mut_frame()
                .exp
                .push(SValue::Global(FSRObject::true_id()));
        } else {
            self.get_cur_mut_frame()
                .exp
                .push(SValue::Global(FSRObject::false_id()));
        }

        Ok(false)
    }

    fn empty_process(
        self: &mut FSRThreadRuntime<'a>,
        _bc: &BytecodeArg,
        _: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        Ok(false)
    }

    #[inline(always)]
    fn process(&mut self, bytecode: &'a BytecodeArg, bc: &'a Bytecode) -> Result<bool, FSRError> {
        let op = bytecode.get_operator();

        let v = match op {
            BytecodeOperator::Assign => Self::assign_process(self, bytecode, bc),
            BytecodeOperator::BinaryAdd => Self::binary_add_process(self, bytecode, bc),
            BytecodeOperator::BinaryDot => Self::binary_dot_process(self, bytecode, bc),
            BytecodeOperator::BinaryMul => Self::binary_mul_process(self, bytecode, bc),
            BytecodeOperator::Call => Self::call_process(self, bytecode, bc),
            BytecodeOperator::IfTest => Self::if_test_process(self, bytecode, bc),
            BytecodeOperator::WhileTest => Self::while_test_process(self, bytecode, bc),
            BytecodeOperator::DefineFn => Self::define_fn(self, bytecode, bc),
            BytecodeOperator::EndDefineFn => Self::end_define_fn(self, bytecode, bc),
            BytecodeOperator::CompareTest => Self::compare_test(self, bytecode, bc),
            BytecodeOperator::ReturnValue => Self::ret_value(self, bytecode, bc),
            BytecodeOperator::WhileBlockEnd => Self::while_block_end(self, bytecode, bc),
            BytecodeOperator::AssignArgs => Self::assign_args(self, bytecode, bc),
            BytecodeOperator::ClassDef => Self::class_def(self, bytecode, bc),
            BytecodeOperator::EndDefineClass => Self::end_class_def(self, bytecode, bc),
            BytecodeOperator::LoadList => Self::load_list(self, bytecode, bc),
            BytecodeOperator::Else => Self::else_process(self, bytecode, bc),
            BytecodeOperator::ElseIf => Self::else_if_match(self, bytecode, bc),
            BytecodeOperator::ElseIfTest => Self::else_if_test_process(self, bytecode, bc),
            BytecodeOperator::IfBlockEnd => Self::if_end(self, bytecode, bc),
            BytecodeOperator::Break => Self::break_process(self, bytecode, bc),
            BytecodeOperator::Continue => Self::continue_process(self, bytecode, bc),
            BytecodeOperator::LoadForIter => Self::load_for_iter(self, bytecode, bc),
            BytecodeOperator::ForBlockEnd => Self::for_block_end(self, bytecode, bc),
            BytecodeOperator::SpecialLoadFor => Self::special_load_for(self, bytecode, bc),
            BytecodeOperator::AndJump => Self::process_logic_and(self, bytecode, bc),
            BytecodeOperator::OrJump => Self::process_logic_or(self, bytecode, bc),
            BytecodeOperator::Empty => Self::empty_process(self, bytecode, bc),
            BytecodeOperator::BinarySub => Self::binary_sub_process(self, bytecode, bc),
            BytecodeOperator::Import => {
                Self::process_import(self, bytecode, self.get_context().code)
            }
            BytecodeOperator::BinaryDiv => Self::binary_div_process(self, bytecode, bc),
            BytecodeOperator::NotOperator => Self::not_process(self, bytecode, bc),
            BytecodeOperator::BinaryClassGetter => {
                Self::binary_get_cls_attr_process(self, bytecode, bc)
            }
            BytecodeOperator::Getter => Self::getter_process(self, bytecode, bc),
            BytecodeOperator::Try => Self::try_process(self, bytecode),
            BytecodeOperator::EndTry => Self::try_end(self),
            BytecodeOperator::EndCatch => Self::catch_end(self, bytecode),
            BytecodeOperator::BinaryRange => Self::binary_range_process(self),
            BytecodeOperator::ForBlockRefAdd => Self::for_block_ref(self),
            BytecodeOperator::LoadConst => Self::load_const(self, bytecode),
            BytecodeOperator::BinaryReminder => Self::binary_reminder_process(self, bytecode, bc),
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

        if v {
            return Ok(v);
        }

        Ok(false)
    }

    fn load_const(&mut self, arg: &'a BytecodeArg) -> Result<bool, FSRError> {
        let code = FSRObject::id_to_mut_obj(self.get_context().code)
            .unwrap()
            .as_mut_code();
        match arg.get_arg() {
            ArgType::ConstInteger(index, obj, single_op) => {
                let i = obj.parse::<i64>().unwrap();
                let i = if single_op.is_some() && single_op.as_ref().unwrap().eq("-") {
                    -1 * i
                } else {
                    i
                };
                let ptr = {
                    let mut obj = FSRInteger::new_inst(i);
                    // obj.ref_add();
                    obj.area = Area::Global;
                    let ptr = FSRVM::leak_object(Box::new(obj));
                    ptr
                };

                code.insert_const(*index as usize, ptr);
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
                    let ptr = FSRVM::leak_object(Box::new(obj));
                    ptr
                };

                code.insert_const(*index as usize, ptr);
            }
            ArgType::ConstString(index, s) => {
                let obj = FSRString::new_value(s);
                // obj.ref_add();
                let obj = FSRObject::new_inst(obj, FSRGlobalObjId::StringCls as ObjId);
                let ptr = FSRVM::leak_object(Box::new(obj));

                code.insert_const(*index as usize, ptr);
            }
            _ => unimplemented!(),
        }

        Ok(false)
    }

    #[inline(always)]
    fn load_var(&mut self, arg: &'a BytecodeArg) -> Result<bool, FSRError> {
        let exp = &mut self.get_cur_mut_frame().exp;
        match arg.get_arg() {
            ArgType::Variable(var) => {
                self.get_cur_mut_frame().exp.push(SValue::Stack(var));
            }
            ArgType::Const(index) => {
                let code = FSRObject::id_to_obj(self.get_context().code).as_code();
                let obj = code.get_const(*index as usize).unwrap();
                self.get_cur_mut_frame().exp.push(SValue::Global(obj));
            }
            ArgType::Attr(attr_id, name) => {
                let new_attr = self
                    .thread_allocator
                    .new_box_attr(*attr_id, 0, None, name, true);
                self.get_cur_mut_frame().exp.push(SValue::Attr(new_attr));
            }
            ArgType::ClosureVar(v) => {
                let fn_id = self.get_cur_frame().fn_obj;
                if fn_id == 0 {
                    panic!("not found function object");
                }
                let fn_obj = FSRObject::id_to_obj(fn_id).as_fn();
                let var = fn_obj.get_closure_var(&v.1);
                self.get_cur_mut_frame()
                    .exp
                    .push(SValue::Global(var.unwrap()));
            }
            ArgType::CurrentFn => {
                let fn_id = self.get_cur_frame().fn_obj;
                if fn_id == 0 {
                    panic!("not found function object");
                }
                self.get_cur_mut_frame().exp.push(SValue::Global(fn_id));
            }
            ArgType::GlobalId(id) => {
                self.get_cur_mut_frame().exp.push(SValue::Global(*id));
            }
            _ => {
                println!("{:?}", self.get_cur_mut_frame().exp);
                unimplemented!()
            }
        }

        Ok(false)
    }

    #[inline(always)]
    fn set_exp_stack_ret(&mut self) {
        let state = self.get_cur_frame();
        // if state.exp.is_some() {
        //     let v = self.get_cur_mut_frame().exp.take().unwrap();
        //     self.get_cur_mut_context().exp = v;
        // }

        if self.get_cur_mut_frame().ret_val.is_some() {
            let v = self.get_cur_mut_frame().ret_val.take().unwrap();
            self.get_cur_mut_frame().exp.push(SValue::Global(v));
        }
    }

    #[inline(always)]
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

    #[inline(always)]
    fn run_expr_wrapper(
        &mut self,
        expr: &'a [BytecodeArg],
        bc: &'a Bytecode,
    ) -> Result<bool, FSRError> {
        if self.counter - self.last_aquire_counter > 100 {
            self.rt_yield();
        }

        self.run_expr(expr, bc)
    }

    #[inline(always)]
    fn run_expr(&mut self, expr: &'a [BytecodeArg], bc: &'a Bytecode) -> Result<bool, FSRError> {
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
            match arg.get_operator() {
                BytecodeOperator::Load => {
                    Self::load_var(self, arg)?;
                }
                _ => {
                    v = self.process(arg, bc)?;
                    if self.get_cur_frame().ret_val.is_some() {
                        return Ok(true);
                    }

                    if v {
                        self.get_cur_mut_frame().exp.clear();
                        return Ok(false);
                    }
                }
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
            self.run_expr_wrapper(expr, code.get_bytecode())?;
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
            self.run_expr_wrapper(expr, code.get_bytecode())?;
            code = FSRObject::id_to_obj(self.get_context().code).as_code();
        }

        println!("count: {}", self.counter);

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
            let v = self.run_expr_wrapper(expr, fn_def.get_bytecode())?;
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
        let mut runtime = FSRThreadRuntime::new();
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
        let mut runtime = FSRThreadRuntime::new();
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
        let mut runtime = FSRThreadRuntime::new();
        runtime.start(obj_id).unwrap();
    }

    #[test]
    fn test_svalue_size() {
        println!("svalue size: {}", std::mem::size_of::<super::SValue>());
        println!(
            "result size: {}",
            std::mem::size_of::<Result<ObjId, FSRError>>()
        );
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
        let mut runtime = FSRThreadRuntime::new();
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
        let mut runtime = FSRThreadRuntime::new();
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
        let mut runtime = FSRThreadRuntime::new();
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
        let mut runtime = FSRThreadRuntime::new();
        runtime.start(obj_id).unwrap();
    }

    #[test]
    fn test_lambda() {
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
        let mut runtime = FSRThreadRuntime::new();
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
        let mut runtime = FSRThreadRuntime::new();
        runtime.start(obj_id).unwrap();
    }
}
