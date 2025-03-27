use std::collections::HashSet;
use std::sync::atomic::Ordering;

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

    roots: HashSet<ObjId>,
    // Object allocator
    allocator: FSRObjectAllocator<'a>,
    // mark bitmap
    marks: Vec<bool>,

    tracker: Tracker,
}

impl<'a> MarkSweepGarbageCollector<'a> {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            free_slots: Vec::new(),
            roots: HashSet::new(),
            allocator: FSRObjectAllocator::new(),
            marks: Vec::new(),
            tracker: Tracker { object_count: 0 },
        }
    }


    pub fn add_root(&mut self, id: ObjId) {
        self.roots.insert(id);
    }


    pub fn remove_root(&mut self, id: ObjId) {
        self.roots.remove(&id);
    }


    fn get_garbage_id(&self, id: ObjId) -> Option<usize> {
        if FSRObject::is_sp_object(id) {
            return None;
        }


        let obj = unsafe { FSRObject::id_to_obj(id) };
        let garbage_id = obj.garbage_id.load(Ordering::Relaxed) as usize;

        

        if garbage_id < self.objects.len() {
            Some(garbage_id)
        } else {
            None
        }
    }


    fn get_object(&self, id: ObjId) -> Option<&Box<FSRObject<'a>>> {
        self.get_garbage_id(id)
            .and_then(|idx| self.objects.get(idx).and_then(|slot| slot.as_ref()))
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
        if let Some(idx) = self.get_garbage_id(id) {
            if idx >= self.marks.len() {
                self.marks.resize(self.objects.len(), false);
            }
            self.marks[idx] = true;
        }
    }


    fn is_marked(&self, id: ObjId) -> bool {
        self.get_garbage_id(id)
            .map(|idx| idx < self.marks.len() && self.marks[idx])
            .unwrap_or(false)
    }


    fn clear_marks(&mut self) {
        self.marks.iter_mut().for_each(|m| *m = false);
    }
}

impl<'a> GarbageCollector<'a> for MarkSweepGarbageCollector<'a> {
    fn new_object(&mut self, cls: ObjId, value: FSRValue<'a>) -> ObjId {

        let mut obj = self.allocator.allocate(value, cls);

        // Reuse free slot if available
        let slot_idx = if let Some(free_idx) = self.free_slots.pop() {
            free_idx
        } else {

            let idx = self.objects.len();
            self.objects.push(None);
            idx
        };


        obj.garbage_id.store(slot_idx as u32, Ordering::Relaxed);


        let obj_id = FSRObject::obj_to_id(&obj);


        self.objects[slot_idx] = Some(obj);
        self.tracker.object_count += 1;

        if self.marks.len() <= slot_idx {
            self.marks.resize(self.objects.len(), false);
        }

        obj_id
    }

    fn free_object(&mut self, id: ObjId) {
        if let Some(idx) = self.get_garbage_id(id) {
            if idx < self.objects.len() {
                if let Some(obj) = self.objects[idx].take() {

                    self.allocator.free_object(obj);

                    self.free_slots.push(idx);
                }
            }
        }
    }

    fn collect(&mut self) {
        self.clear_marks();

        let mut work_list: Vec<ObjId> = self.roots.iter().copied().collect();

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

        let mut to_free = Vec::new();

        for (idx, obj_opt) in self.objects.iter().enumerate() {
            if let Some(obj) = obj_opt {
                let id = FSRObject::obj_to_id(obj);

                if idx >= self.marks.len() || !self.marks[idx] {
                    to_free.push(id);
                }
            }
        }

        for id in to_free {
            self.tracker.object_count -= 1;
            self.free_object(id);
        }
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{
        types::{base::FSRGlobalObjId, list::FSRList},
        vm::virtual_machine::FSRVM,
    };

    #[test]
    fn test_mark_sweep_gc() {
        let _vm = FSRVM::new();
        let mut gc = MarkSweepGarbageCollector::new();


        let integer_cls = FSRGlobalObjId::IntegerCls as ObjId;
        let list_cls = FSRGlobalObjId::ListCls as ObjId;

        let int1 = gc.new_object(integer_cls, FSRValue::Integer(10));
        let int2 = gc.new_object(integer_cls, FSRValue::Integer(20));
        let int3 = gc.new_object(integer_cls, FSRValue::Integer(30));

        let mut list_val = vec![];
        list_val.push(int1);
        list_val.push(int2);
        let list = gc
            .new_object(list_cls, FSRList::new_value(list_val));

        assert!(gc.get_object(int1).is_some());
        assert!(gc.get_object(int2).is_some());
        assert!(gc.get_object(int3).is_some());
        assert!(gc.get_object(list).is_some());

        gc.collect();

        assert!(gc.get_object(int1).is_none());
        assert!(gc.get_object(int2).is_none());
        assert!(gc.get_object(int3).is_none());
        assert!(gc.get_object(list).is_none());

        let int1 = gc.new_object(integer_cls, FSRValue::Integer(10));
        let int2 = gc.new_object(integer_cls, FSRValue::Integer(20));
        let int3 = gc.new_object(integer_cls, FSRValue::Integer(30));

        let mut list_val = vec![];
        list_val.push(int1);
        list_val.push(int2);
        let list = gc
            .new_object(list_cls, FSRList::new_value(list_val));


        gc.add_root(list);
        gc.collect();

        assert!(gc.get_object(int1).is_some());
        assert!(gc.get_object(int2).is_some());
        assert!(gc.get_object(list).is_some());

        assert!(gc.get_object(int3).is_none());

        gc.remove_root(list);
        gc.collect();


        assert!(gc.get_object(int1).is_none());
        assert!(gc.get_object(int2).is_none());
        assert!(gc.get_object(list).is_none());


        let before_alloc = gc.objects.len();
        let free_count = gc.free_slots.len();


        let new_int = gc.new_object(integer_cls, FSRValue::Integer(100));


        assert!(gc.get_object(new_int).is_some());


        assert_eq!(gc.objects.len(), before_alloc);
        assert_eq!(gc.free_slots.len(), free_count - 1);

        println!("{:#?}", gc);
    }

    #[test]
    fn test_mark_sweep_gc_list() {

        let _vm = FSRVM::new();
        let mut gc = MarkSweepGarbageCollector::new();


        let integer_cls = FSRGlobalObjId::IntegerCls as ObjId;
        let list_cls = FSRGlobalObjId::ListCls as ObjId;


        let int1 = gc.new_object(integer_cls, FSRValue::Integer(10));
        let int2 = gc.new_object(integer_cls, FSRValue::Integer(20));
        let int3 = gc.new_object(integer_cls, FSRValue::Integer(30));


        let mut list_val = vec![];
        list_val.push(int1);
        list_val.push(int2);
        let list = gc
            .new_object(list_cls, FSRList::new_value(list_val));


        assert!(gc.get_object(int1).is_some());
        assert!(gc.get_object(int2).is_some());
        assert!(gc.get_object(int3).is_some());
        assert!(gc.get_object(list).is_some());


        gc.collect();


        assert!(gc.get_object(int1).is_none());
        assert!(gc.get_object(int2).is_none());
        assert!(gc.get_object(int3).is_none());
        assert!(gc.get_object(list).is_none());


        let int1 = gc.new_object(integer_cls, FSRValue::Integer(10));
        let int2 = gc.new_object(integer_cls, FSRValue::Integer(20));
        let int3 = gc.new_object(integer_cls, FSRValue::Integer(30));

        let mut list_val = vec![];
        list_val.push(int1);
        list_val.push(int2);
        let list = gc
            .new_object(list_cls, FSRList::new_value(list_val));
    }
}
