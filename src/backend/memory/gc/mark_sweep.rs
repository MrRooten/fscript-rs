#![allow(clippy::vec_box)]

use std::sync::atomic::{AtomicBool, Ordering};

use crate::backend::types::base::{Area, GlobalObj};

use crate::backend::types::class::FSRClass;
use crate::backend::types::string::FSRInnerString;
use crate::backend::vm::virtual_machine::gid;
use crate::backend::{
    memory::{size_alloc::FSRObjectAllocator, GarbageCollector},
    types::base::{FSRObject, FSRValue, ObjId},
};

use super::Tracker;

#[derive(PartialEq, Debug)]
pub enum GcReason {
    Full,
    Minjor,
    AllocationFailure,
    ThresholdBased,
    ManulTrigger,
    TimeBased,
    SafePointTrigger,
}

const ESCAPE_COUNT: u32 = 2;

pub struct MarkSweepGarbageCollector<'a> {
    marjor_arena: Vec<Option<Box<FSRObject<'a>>>>,
    // Store all objects
    objects: Vec<Option<Box<FSRObject<'a>>>>,
    // Free slots for objects
    free_slots: Vec<usize>,
    // Object allocator
    allocator: FSRObjectAllocator<'a>,
    // // mark bitmap
    // marks: Vec<bool>,
    pub(crate) tracker: Tracker,

    check: AtomicBool,

    gc_reason: Option<GcReason>,
}

const THROLD: usize = 10240 * 2;

impl<'a> MarkSweepGarbageCollector<'a> {
    pub fn get_stop_time(&self) -> u64 {
        self.tracker.collect_time
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
            allocator: FSRObjectAllocator::new(),
            // marks: Vec::with_capacity(THROLD),
            tracker: Tracker {
                object_count: 0,
                throld: THROLD / 5,
                collect_time: 0,
                count_free: 0,
                collect_count: 0,
                minjar_object_count: 0,
                marjor_object_count: 0,
                #[cfg(feature = "track_memory_size")]
                memory_size: 0,
            },
            check: AtomicBool::new(false),
            marjor_arena: Vec::with_capacity(THROLD),
            gc_reason: None,
        }
    }

    pub fn set_reason(&mut self, reason: GcReason) {
        self.gc_reason = Some(reason);
    }

    pub fn clear_marks(&mut self) {
        // self.marks.iter_mut().for_each(|m| *m = false);
        self.objects.iter_mut().for_each(|m| {
            if let Some(obj) = m {
                obj.unmark();
            }
        });

        self.marjor_arena.iter_mut().for_each(|m| {
            if let Some(obj) = m {
                obj.unmark();
            }
        });
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn alloc_object(&mut self, free_idx: usize, value: FSRValue<'a>, cls: ObjId) -> ObjId {
        debug_assert!(free_idx < self.objects.len(), "free_idx out of bounds");
        let obj = &mut self.objects[free_idx];
        if let Some(obj) = obj {
            obj.value = value;
            let cls = FSRObject::id_to_obj(cls).as_class();
            obj.cls = cls;
            obj.free = false;
            obj.area = Area::Minjor;
            #[cfg(feature = "track_memory_size")]
            {
                self.tracker.memory_size += obj.get_size();
            }
            self.tracker.minjar_object_count += 1;
            FSRObject::obj_to_id(obj)
        } else {
            let mut obj = self.allocator.new_object(value, cls);
            let cls = FSRObject::id_to_obj(cls).as_class();
            obj.cls = cls;
            obj.free = false;
            obj.area = Area::Minjor;
            #[cfg(feature = "track_memory_size")]
            {
                self.tracker.memory_size += obj.get_size();
            }
            self.tracker.minjar_object_count += 1;
            self.objects[free_idx] = Some(obj);
            FSRObject::obj_to_id(self.objects[free_idx].as_ref().unwrap())
        }
    }

    #[cfg_attr(feature = "more_inline", inline(always))]
    fn alloc_when_full(&mut self, value: FSRValue<'a>, cls: ObjId) -> ObjId {
        let slot_idx = self.objects.len();
        let obj = self
            .allocator
            .new_object(FSRValue::None, GlobalObj::NoneObj as ObjId);

        self.objects.push(Some(obj));
        let obj = &mut self.objects[slot_idx];
        if let Some(obj) = obj {
            obj.value = value;
            let cls = FSRObject::id_to_obj(cls).as_class();
            obj.cls = cls;
            obj.free = false;
            obj.area = Area::Minjor;
            self.tracker.minjar_object_count += 1;
            #[cfg(feature = "track_memory_size")]
            {
                self.tracker.memory_size += obj.get_size();
            }
            return FSRObject::obj_to_id(obj);
        }

        unimplemented!()
    }

    pub fn preserve(&mut self) {
        let extend_size = if self.objects.len() / 2 == 0 {
            4096
        } else {
            self.objects.len() / 2
        };
        //let extend_size = self.objects.len() / 2;
        let last = self.objects.len();
        self.objects.extend((0..extend_size).map(|_| {
            let mut obj = Box::new(FSRObject::new_inst(
                FSRValue::None,
                gid(GlobalObj::NoneCls),
            ));
            obj.free = true;
            Some(obj)
        }));

        for i in 0..extend_size {
            let slot = last + i;
            self.free_slots.push(slot);
        }
    }

    pub fn shrink(&mut self) {
        self.free_slots.clear();
        self.objects.retain(|obj| {
            if let Some(obj) = obj {
                return !obj.free
            }
            false
        });
        self.tracker.object_count = self.objects.len() as u32 + self.marjor_arena.len() as u32;
    }

    fn process_object(&mut self, i: usize, full: bool, freed_count: &mut u32) {
        let obj = &mut self.objects[i];
        let mut is_mark = false;
        let mut count = 0;
        if let Some(obj) = obj {
            is_mark = obj.is_marked();
            if (!is_mark && !obj.free) && ((!full && obj.area == Area::Minjor) || full) {
                // if (!full && obj.area == Area::Minjor) || full {
                if obj.area == Area::Minjor {
                    self.tracker.minjar_object_count -= 1;
                } else {
                    self.tracker.marjor_object_count -= 1;
                }

                #[cfg(feature = "track_memory_size")]
                {
                    self.tracker.memory_size =
                        self.tracker.memory_size.saturating_sub(obj.get_size());
                }

                //self.tracker.memory_size -= obj.get_size();
                obj.free = true;
                self.free_slots.push(i);
                *freed_count += 1;
                //}
            }

            if is_mark {
                obj.gc_count += 1;
                count = obj.gc_count;
            }
        }

        // if is_mark && count > ESCAPE_COUNT {
        if is_mark && count >= ESCAPE_COUNT {
            let mut obj = obj.take().unwrap();
            obj.area = Area::Marjor;
            obj.undirty_object();
            self.tracker.marjor_object_count += 1;
            self.tracker.minjar_object_count -= 1;
            self.marjor_arena.push(Some(obj));
        }
    }

    pub fn alloc_object_in_place(&mut self) -> &mut FSRObject<'a> {
        let free_idx = self.free_slots.pop().unwrap();
        //debug_assert!(free_idx < self.objects.len(), "free_idx out of bounds");
        let obj = &mut self.objects[free_idx];
        if let Some(obj) = obj {
            obj.free = false;
            obj.area = Area::Minjor;
            #[cfg(feature = "track_memory_size")]
            {
                self.tracker.memory_size += obj.get_size();
            }
            return obj;
        }
        unimplemented!()
    }

    pub fn new_object_in_place(&mut self) -> &mut FSRObject<'a> {
        self.tracker.object_count += 1;
        if self.free_slots.is_empty() {
            self.preserve();
        }


        self.tracker.minjar_object_count += 1;
        self.alloc_object_in_place()

    }

    fn tracker_process(&mut self, freed_count: u32) {
        self.tracker.object_count -= freed_count;
        if self.tracker.object_count as usize > self.tracker.throld * 9 / 10 {
            self.tracker.throld *= 20;
        } else if self.tracker.throld / 20 > self.tracker.object_count as usize
            && self.tracker.throld / 5 > THROLD
        {
            self.tracker.throld /= 5;
        }

        if self.tracker.collect_count % 100 == 0 {
            self.shrink();
        }

        self.tracker.count_free += freed_count as u64;
        self.tracker.collect_count += 1;
    }
}

impl<'a> MarkSweepGarbageCollector<'a> {
    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn new_object(&mut self, value: FSRValue<'a>, cls: ObjId) -> ObjId {
        // Reuse free slot if available
        self.tracker.object_count += 1;
        if self.free_slots.is_empty() {
            self.preserve();
        }
        if let Some(free_idx) = self.free_slots.pop() {
            return self.alloc_object(free_idx, value, cls);
        }

        panic!("No free slots available for allocation");
        // else {
        //     self.alloc_when_full(value, cls)
        // }
    }

    pub fn new_string<IntoS>(&mut self, value: IntoS)
        -> ObjId
    where
        IntoS: Into<String>,
    {
        let value = FSRValue::String(FSRInnerString::new(value).into());
        self.new_object(value, gid(GlobalObj::StringCls))
    }

    pub fn collect(&mut self, full: bool) {
        let mut i = 0;
        let mut freed_count = 0;

        // if self.objects.len() > self.tracker.minjar_object_count as usize * 10 {
        //     self.shrink();
        // }

        while i < self.objects.len() {
            self.process_object(i, full, &mut freed_count);
            i += 1;
        }

        self.tracker_process(freed_count);
    }

    #[inline]
    pub fn will_collect(&self) -> bool {
        self.tracker.object_count as usize > self.tracker.throld * 3 || self.gc_reason.is_some()
    }
}
