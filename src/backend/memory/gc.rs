use std::{collections::LinkedList, sync::Mutex};

use crate::backend::types::base::{FSRObject, ObjId};

use super::{size_alloc::FSRObjectAllocator, FSRAllocator};



pub struct ObjectGeneration {}

type GarbageId = u32;

pub struct GarbageCollector {
    objects: Vec<Option<ObjId>>,
    object_map: Vec<bool>,
    locker: Mutex<()>,
    len: usize,
}

impl GarbageCollector {
    pub fn new() -> Self {
        Self {
            // keep the first element as None, because the garbage id starts from 1, 0 means the object is not in the garbage collector
            objects: vec![None],
            len: 0,
            // keep the first element as true, because the garbage id starts from 1, 0 means the object is not in the garbage collector
            object_map: vec![true],
            locker: Mutex::new(()),
        }
    }

    fn try_insert<T>(list: &mut Vec<T>, index: usize, value: T) -> GarbageId {
        if index < list.len() {
            list[index] = value;
            index as GarbageId
        } else {
            list.push(value);
            list.len() as GarbageId - 1
        }
    }

    pub fn add_object(&mut self, obj_id: ObjId) {
        let obj = FSRObject::id_to_obj(obj_id);

        if obj.get_garbage_id() > 0 {
            // if the object is already in the garbage collector
            return;
        }

        let garbage_id = Self::try_insert(&mut self.objects, self.len, Some(obj_id));
        
        obj.set_garbage_id(garbage_id as GarbageId);
        self.len += 1;
    }

    fn remove_object(&mut self, obj: ObjId) {
        let obj = FSRObject::id_to_obj(obj);

        if obj.get_garbage_id() == 0 {
            // if the object is not in the garbage collector
            return;
        }

        let garbage_id = obj.get_garbage_id();
        self.objects[garbage_id as usize] = None;
    }

    pub fn sort(&mut self) {
        let mut first = 1;
        let mut last = self.objects.len() - 1;

        while first < last {
            while self.objects[first].is_some() {
                first += 1;
            }
            while self.objects[last].is_none() {
                last -= 1;
            }
            if first < last {
                self.objects.swap(first, last);
                self.object_map.swap(first, last);
            }
        }
    }

    pub fn iter_from_obj(&mut self, obj: ObjId) {
        let obj = FSRObject::id_to_obj(obj);
        let mut sk = vec![];
        sk.push(obj);
        while !sk.is_empty() {
            let cur = sk.pop().unwrap();
            let id = cur.get_garbage_id() as usize;
            if id == 0 {
                continue;
            }
            if self.object_map[id] {
                // if the object is already visited
                continue;
            }
            self.object_map[id] = true; // mark the object as visited
            for &id in cur.iter_object() {
                self.object_map[id] = true;
                let obj = FSRObject::id_to_obj(id);
                sk.push(obj);
            }
        }
    }

    pub fn iter(&mut self) -> ObjIterator {
        ObjIterator {
            gc: self,
            index: 1,
        }
    }

    pub fn clear_map(&mut self) {
        self.object_map.fill(false);
    }
}

pub struct ObjIterator<'a> {
    gc: &'a mut GarbageCollector,
    index: usize,
}

impl Iterator for ObjIterator<'_> {
    type Item = ObjId;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.gc.objects.len() {
            let is_ref = self.gc.object_map[self.index];
            self.index += 1;
            if !is_ref && self.gc.objects[self.index - 1].is_some() {
                let out = self.gc.objects[self.index - 1].take();
                self.gc.object_map[self.index - 1] = false;
                self.gc.len -= 1;
                return out;
            }
        }
        None
    }
}

mod test {
    use crate::backend::types::{base::{FSRObject, FSRValue}, integer::FSRInteger};

    use super::GarbageCollector;

    #[test]
    fn test_garbege_collector() {
        let mut gc = GarbageCollector::new();

        let value = Box::new(FSRInteger::new_inst(1));

        let obj_id = FSRObject::obj_to_id(&value);

        gc.add_object(obj_id);

        println!("{:?}", gc.objects);
    }
}
