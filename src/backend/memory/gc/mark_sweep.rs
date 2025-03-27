use std::collections::{HashMap, HashSet};
use std::sync::atomic::Ordering;

use ahash::AHashMap;

use crate::backend::vm::thread::CallFrame;
use crate::backend::{
    memory::{size_alloc::FSRObjectAllocator, FSRAllocator, GarbageCollector},
    types::base::{FSRObject, FSRValue, ObjId},
};

#[derive(Debug)]
struct Tracker {
    object_count: u32,
}

#[derive(Debug)]
pub struct MarkSweepGarbageCollector<'a> {
    // Store all objects
    objects: Vec<Option<Box<FSRObject<'a>>>>,
    // Free slots for objects
    free_slots: Vec<usize>,

    roots: Vec<ObjId>,
    // Object allocator
    allocator: FSRObjectAllocator<'a>,
    // mark bitmap
    marks: Vec<bool>,

    tracker: Tracker,

    self_id: u32,
}

const THROLD: usize = 10240;

impl<'a> MarkSweepGarbageCollector<'a> {
    pub fn get_object_count(&self) -> u32 {
        self.tracker.object_count
    }

    pub fn new() -> Self {
        Self {
            objects: Vec::with_capacity(THROLD),
            free_slots: Vec::with_capacity(THROLD),
            roots: vec![],
            allocator: FSRObjectAllocator::new(),
            marks: Vec::with_capacity(THROLD),
            tracker: Tracker { object_count: 0 },
            self_id: 1,
        }
    }

    pub fn add_root(&mut self, id: ObjId) {
        if FSRObject::id_to_obj(id).garbage_collector_id != self.self_id {
            return;
        }

        if FSRObject::is_sp_object(id) {
            return;
        }

        self.roots.push(id);
    }

    #[inline(always)]
    fn get_garbage_id(&self, id: ObjId) -> Option<usize> {
        if FSRObject::is_sp_object(id) {
            return None;
        }

        let obj = FSRObject::id_to_obj(id);
        let garbage_id = obj.garbage_id as usize;

        if garbage_id < self.objects.len() {
            Some(garbage_id)
        } else {
            None
        }
    }

    fn get_object(&self, id: ObjId) -> Option<&Box<FSRObject<'a>>> {
        // self.get_garbage_id(id)
        //     .and_then(|idx| self.objects.get(idx).and_then(|slot| slot.as_ref()))

        let obj = FSRObject::id_to_obj(id);
        if obj.garbage_collector_id != self.self_id {
            return None;
        }

        let idx = obj.garbage_id as usize;
        if idx < self.objects.len() {
            return self.objects.get(idx).and_then(|slot| slot.as_ref());
        }
        None
    }

    fn get_object_mut(&mut self, id: ObjId) -> Option<&mut Box<FSRObject<'a>>> {
        if let Some(idx) = self.get_garbage_id(id) {
            if idx < self.objects.len() {
                return self.objects.get_mut(idx).and_then(|slot| slot.as_mut());
            }
        }
        None
    }

    fn mark(&mut self, id: ObjId) {
        let obj = FSRObject::id_to_obj(id);
        if obj.garbage_collector_id != self.self_id {
            return;
        }
        let idx = obj.garbage_id as usize;
        if idx >= self.marks.len() {
            self.marks.resize(((self.objects.len() + 7) & !7), false);
        }
        self.marks[idx] = true;
    }

    fn is_marked(&self, id: ObjId) -> bool {
        // self.get_garbage_id(id)
        //     .map(|idx| idx < self.marks.len() && self.marks[idx])
        //     .unwrap_or(false)
        let obj = FSRObject::id_to_obj(id);
        if obj.garbage_collector_id != self.self_id {
            return false;
        }

        let idx = obj.garbage_id as usize;
        if idx >= self.marks.len() {
            return false;
        }
        self.marks[idx]
    }

    fn clear_marks(&mut self) {
        self.marks.iter_mut().for_each(|m| *m = false);
    }

    pub fn will_collect(&self) -> bool {
        self.tracker.object_count as usize > THROLD
    }

    fn free_object(&mut self, id: ObjId) {
        if let Some(idx) = self.get_garbage_id(id) {
            if idx < self.objects.len() {
                if let Some(mut obj) = self.objects[idx].take() {
                    obj.garbage_collector_id = 0;
                    self.allocator.free_object(obj);
                    self.free_slots.push(idx);
                }
            }
        }
    }
}

impl<'a> GarbageCollector<'a> for MarkSweepGarbageCollector<'a> {
    fn new_object(&mut self, value: FSRValue<'a>, cls: ObjId) -> ObjId {
        let mut obj = self.allocator.new_object(value, cls);
        obj.garbage_collector_id = self.self_id;
        // Reuse free slot if available
        let slot_idx = if let Some(free_idx) = self.free_slots.pop() {
            free_idx
        } else {
            let idx = self.objects.len();
            self.objects.push(None);
            idx
        };

        obj.garbage_id = slot_idx as u32;

        let obj_id = FSRObject::obj_to_id(&obj);

        self.objects[slot_idx] = Some(obj);
        self.tracker.object_count += 1;

        if self.marks.len() <= slot_idx {
            self.marks.resize(((self.objects.len() + 7) & !7), false);
        }

        obj_id
    }

    

    fn collect(&mut self, frames: &Vec<Box<CallFrame<'a>>>, cur_frame: &Box<CallFrame>, others: &[ObjId]) {
        self.clear_marks();

        let mut work_list = vec![];
        for it in frames {
            for obj in it.var_map.iter() {
                work_list.push(obj);
                
            }

            if let Some(s) = &it.exp {
                for i in s {
                    let v = match i {
                        crate::backend::vm::thread::SValue::Global(i) => *i,
                        _ => continue
                    };

                    work_list.push(v);
                }
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
            work_list.push(obj);
        }

        if let Some(ret_val) = it.ret_val {
            work_list.push(ret_val);
        }

        if it.handling_exception != 0 {
            work_list.push(it.handling_exception);
        }

        for obj in others {
            work_list.push(*obj);
        }

        work_list.extend(self.roots.iter());
        self.roots.clear();


        while let Some(id) = work_list.pop() {
            if self.is_marked(id) {
                continue;
            }

            self.mark(id);

            if let Some(obj) = self.get_object(id) {
                let refs = obj.get_references();

                for ref_id in refs {
                    if !self.is_marked(ref_id) {
                        work_list.push(ref_id);
                    }
                }
            }
        }

        let mut i = 0;
        let mut freed_count = 0;

        while i < self.objects.len() {
            let should_free = if let Some(obj) = &self.objects[i] {
                obj.garbage_collector_id == self.self_id
                    && (i >= self.marks.len() || !self.marks[i])
            } else {
                false
            };

            if should_free {
                // 直接在这里获取对象ID并释放
                if let Some(mut obj) = self.objects[i].take() {
                    // let id = FSRObject::obj_to_id(obj);
                    obj.garbage_collector_id = 0;
                    self.free_slots.push(obj.garbage_id as usize);
                    self.allocator.free_object(obj);

                    freed_count += 1;
                }
            }
            i += 1;
        }

        self.tracker.object_count -= freed_count;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{
        types::{base::FSRGlobalObjId, list::FSRList},
        vm::virtual_machine::FSRVM,
    };

    // #[test]
    // fn test_mark_sweep_gc() {
    //     let _vm = FSRVM::new();
    //     let mut gc = MarkSweepGarbageCollector::new();

    //     let integer_cls = FSRGlobalObjId::IntegerCls as ObjId;
    //     let list_cls = FSRGlobalObjId::ListCls as ObjId;

    //     let int1 = gc.new_object(FSRValue::Integer(10), integer_cls);
    //     let int2 = gc.new_object(FSRValue::Integer(20), integer_cls);
    //     let int3 = gc.new_object(FSRValue::Integer(30), integer_cls);

    //     let mut list_val = vec![];
    //     list_val.push(int1);
    //     list_val.push(int2);
    //     let list = gc.new_object(FSRList::new_value(list_val), list_cls);

    //     assert!(gc.get_object(int1).is_some());
    //     assert!(gc.get_object(int2).is_some());
    //     assert!(gc.get_object(int3).is_some());
    //     assert!(gc.get_object(list).is_some());

    //     gc.collect();

    //     assert!(gc.get_object(int1).is_none());
    //     assert!(gc.get_object(int2).is_none());
    //     assert!(gc.get_object(int3).is_none());
    //     assert!(gc.get_object(list).is_none());

    //     let int1 = gc.new_object(FSRValue::Integer(10), integer_cls);
    //     let int2 = gc.new_object(FSRValue::Integer(20), integer_cls);
    //     let int3 = gc.new_object(FSRValue::Integer(30), integer_cls);

    //     let mut list_val = vec![];
    //     list_val.push(int1);
    //     list_val.push(int2);
    //     let list = gc.new_object(FSRList::new_value(list_val), list_cls);

    //     gc.add_root(list);
    //     gc.collect();

    //     assert!(gc.get_object(int1).is_some());
    //     assert!(gc.get_object(int2).is_some());
    //     assert!(gc.get_object(list).is_some());

    //     assert!(gc.get_object(int3).is_none());

    //     gc.remove_root(list);
    //     gc.collect();

    //     assert!(gc.get_object(int1).is_none());
    //     assert!(gc.get_object(int2).is_none());
    //     assert!(gc.get_object(list).is_none());

    //     let before_alloc = gc.objects.len();
    //     let free_count = gc.free_slots.len();

    //     let new_int = gc.new_object(FSRValue::Integer(100), integer_cls);

    //     assert!(gc.get_object(new_int).is_some());

    //     assert_eq!(gc.objects.len(), before_alloc);
    //     assert_eq!(gc.free_slots.len(), free_count - 1);

    //     println!("{:#?}", gc);
    // }

    // #[test]
    // fn test_mark_sweep_gc_list() {
    //     let _vm = FSRVM::new();
    //     let mut gc = MarkSweepGarbageCollector::new();

    //     let integer_cls = FSRGlobalObjId::IntegerCls as ObjId;
    //     let list_cls = FSRGlobalObjId::ListCls as ObjId;

    //     let int1 = gc.new_object(FSRValue::Integer(10), integer_cls);
    //     let int2 = gc.new_object(FSRValue::Integer(20), integer_cls);
    //     let int3 = gc.new_object(FSRValue::Integer(30), integer_cls);

    //     let mut list_val = vec![];
    //     list_val.push(int1);
    //     list_val.push(int2);
    //     let list = gc.new_object(FSRList::new_value(list_val), list_cls);

    //     assert!(gc.get_object(int1).is_some());
    //     assert!(gc.get_object(int2).is_some());
    //     assert!(gc.get_object(int3).is_some());
    //     assert!(gc.get_object(list).is_some());

    //     gc.collect();

    //     assert!(gc.get_object(int1).is_none());
    //     assert!(gc.get_object(int2).is_none());
    //     assert!(gc.get_object(int3).is_none());
    //     assert!(gc.get_object(list).is_none());

    //     let int1 = gc.new_object(FSRValue::Integer(10), integer_cls);
    //     let int2 = gc.new_object(FSRValue::Integer(20), integer_cls);
    //     let int3 = gc.new_object(FSRValue::Integer(30), integer_cls);

    //     let mut list_val = vec![];
    //     list_val.push(int1);
    //     list_val.push(int2);
    //     let list = gc.new_object(FSRList::new_value(list_val), list_cls);
    // }
}
