#![allow(clippy::vec_box)]

use std::sync::atomic::{AtomicBool, Ordering};

use crate::backend::types::base::{Area, FSRGlobalObjId};

use crate::backend::{
    memory::{size_alloc::FSRObjectAllocator, GarbageCollector},
    types::base::{FSRObject, FSRValue, ObjId},
};

use super::Tracker;

pub struct MarkSweepGarbageCollector<'a> {
    marjor_arena: Vec<Option<Box<FSRObject<'a>>>>,
    // Store all objects
    objects: Vec<Option<Box<FSRObject<'a>>>>,
    // Free slots for objects
    free_slots: Vec<usize>,

    roots: Vec<ObjId>,
    // Object allocator
    allocator: FSRObjectAllocator<'a>,
    // // mark bitmap
    // marks: Vec<bool>,
    pub(crate) tracker: Tracker,

    self_id: u32,

    check: AtomicBool,
}

const THROLD: usize = 10240 * 2;

impl<'a> MarkSweepGarbageCollector<'a> {
    pub fn get_stop_time(&self) -> u64 {
        self.tracker.collect_time
    }

    pub fn get_speed(&self) -> f64 {
        if self.tracker.collect_time == 0 {
            return 0.0;
        }
        let speed = (self.tracker.count_free * 1000) as f64 / self.tracker.collect_time as f64;
        speed
    }

    pub fn get_collect_count(&self) -> u64 {
        self.tracker.collect_count
    }

    pub fn get_object_count(&self) -> u32 {
        self.tracker.object_count
    }

    pub fn new_gc() -> Self {
        Self {
            objects: Vec::with_capacity(THROLD),
            free_slots: Vec::with_capacity(THROLD),
            roots: vec![],
            allocator: FSRObjectAllocator::new(),
            // marks: Vec::with_capacity(THROLD),
            tracker: Tracker {
                object_count: 0,
                throld: THROLD / 5,
                collect_time: 0,
                count_free: 0,
                collect_count: 0,
            },
            self_id: 1,
            check: AtomicBool::new(false),
            marjor_arena: Vec::with_capacity(THROLD),
        }
    }

    pub fn clear_marks(&mut self) {
        // self.marks.iter_mut().for_each(|m| *m = false);
        self.objects.iter_mut().for_each(|m| {
            if let Some(obj) = m {
                obj.mark.store(false, Ordering::Relaxed);
            }
        });
    }

    #[inline(always)]
    fn alloc_object(&mut self, free_idx: usize, value: FSRValue<'a>, cls: ObjId) -> ObjId {
        debug_assert!(free_idx < self.objects.len(), "free_idx out of bounds");
        let obj = &mut self.objects[free_idx];
        if let Some(obj) = obj {
            obj.value = value;
            obj.cls = cls;
            obj.free = false;
            obj.area = Area::Minjor;
            return FSRObject::obj_to_id(obj);
        } else {
            let mut obj = self.allocator.new_object(value, cls);
            obj.cls = cls;
            obj.free = false;
            obj.area = Area::Minjor;
            self.objects[free_idx] = Some(obj);
            return FSRObject::obj_to_id(self.objects[free_idx].as_ref().unwrap());
        }
    }

    #[inline(always)]
    fn alloc_when_full(&mut self, value: FSRValue<'a>, cls: ObjId) -> ObjId {
        let slot_idx = self.objects.len();
        let mut obj = self
            .allocator
            .new_object(FSRValue::None, FSRGlobalObjId::None as ObjId);

        self.objects.push(Some(obj));
        let obj = &mut self.objects[slot_idx];
        if let Some(obj) = obj {
            obj.value = value;
            obj.cls = cls;
            obj.free = false;
            obj.area = Area::Minjor;

            return FSRObject::obj_to_id(obj);
        }

        unimplemented!()
    }

    pub fn shrink(&mut self) {
        self.free_slots.clear();
        self.objects.retain(|obj| {
            if let Some(obj) = obj {
                if obj.free {
                    return false;
                } else {
                    return true;
                }
            }
            false
        });
    }
}

impl<'a> GarbageCollector<'a> for MarkSweepGarbageCollector<'a> {
    #[inline(always)]
    fn new_object(&mut self, value: FSRValue<'a>, cls: ObjId) -> ObjId {
        // Reuse free slot if available
        self.tracker.object_count += 1;
        if let Some(free_idx) = self.free_slots.pop() {
            self.alloc_object(free_idx, value, cls)
        } else {
            self.alloc_when_full(value, cls)
        }
    }

    fn collect(&mut self, full: bool) {
        let mut i = 0;
        let mut freed_count = 0;

        while i < self.objects.len() {
            let obj = &mut self.objects[i];
            let mut is_mark = false;
            if let Some(obj) = obj {
                is_mark = obj.is_marked();
                if !is_mark && !obj.free {
                    if (!full && obj.area == Area::Minjor) || full {
                        obj.free = true;
                        self.free_slots.push(i);
                        freed_count += 1;
                    }
                }

                if is_mark {
                    obj.area = Area::Marjor;
                }
            }

            i += 1;
        }

        self.tracker.object_count -= freed_count;
        if self.tracker.object_count as usize > self.tracker.throld * 9 / 10 {
            self.tracker.throld *= 20;
        } else if self.tracker.throld / 20 > self.tracker.object_count as usize
            && self.tracker.throld / 5 > THROLD
        {
            self.tracker.throld /= 5;
        }
        self.tracker.count_free += freed_count as u64;
        self.tracker.collect_count += 1;
    }

    fn will_collect(&self) -> bool {
        self.tracker.object_count as usize > self.tracker.throld
            || self.check.load(Ordering::SeqCst)
    }
}
