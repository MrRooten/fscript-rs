use std::{collections::LinkedList, sync::Mutex};

use crate::backend::types::base::{FSRObject, ObjId};

use super::size_alloc::FSRObjectAllocator;



pub struct ObjectGeneration {}

type GarbageId = u32;

pub struct GarbageCollector<'a> {
    objects: Vec<Option<&'a FSRObject<'a>>>,
    object_map: Vec<bool>,
    locker: Mutex<()>,
    last_index: usize,
}

impl<'a> GarbageCollector<'a> {
    pub fn new() -> Self {
        Self {
            objects: vec![],
            last_index: 0,
            object_map: vec![],
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

    pub fn new_object(&mut self, obj: &'a FSRObject) {
        let id = FSRObject::obj_to_id(obj);
        let garbage_id = Self::try_insert(&mut self.objects, self.last_index, Some(obj));
        obj.set_garbage_id(garbage_id as GarbageId);
        self.last_index += 1;
    }

    pub fn sort(&mut self) {
        let mut first = 0;
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
            }
        }
    }

    pub fn iter_from_obj(&mut self, obj: ObjId) {
        let obj = FSRObject::id_to_obj(obj);
        let mut sk = vec![];
        sk.push(obj);
        while !sk.is_empty() {
            let cur = sk.pop().unwrap();
            let id = FSRObject::obj_to_id(cur);
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

    pub fn clear_map(&mut self) {
        self.object_map.fill(false);
    }
}

mod test {
    // use std::collections::{HashMap, LinkedList};
    // use std::hint::black_box;
    // use std::time::Instant;

    // #[test]
    // #[ignore]
    // fn bench_hashmap_vs_vec_vs_linkedlist() {
    //     let mut tmp_vec = vec![];
    //     for i in 0..1_000_000 {
    //         tmp_vec.push(i);
    //     }

    //     let mut tmp_list = LinkedList::new();
    //     for i in 0..1_000_000 {
    //         tmp_list.push_back(i);
    //     }

    //     let mut tmp_map = HashMap::new();
    //     for i in 0..1_000_000 {
    //         tmp_map.insert(i, i);
    //     }

    //     // 测试 Vec 遍历性能
    //     let st = Instant::now();
    //     let mut sum = 0;
    //     for _ in 0..1000 {
    //         for &x in &tmp_vec {
    //             sum += black_box(x);
    //         }
    //     }
    //     let et = Instant::now();
    //     println!("Vec traversal time: {:?}", et - st);

    //     // 测试 LinkedList 遍历性能
    //     let st3 = Instant::now();
    //     let mut sum3 = 0;
    //     for _ in 0..1000 {
    //         for &x in &tmp_list {
    //             sum3 += black_box(x);
    //         }
    //     }
    //     let et3 = Instant::now();
    //     println!("LinkedList traversal time: {:?}", et3 - st3);

    //     // 测试 HashMap 遍历性能
    //     let st2 = Instant::now();
    //     let mut sum2 = 0;
    //     for _ in 0..1000 {
    //         for &x in tmp_map.values() {
    //             sum2 += black_box(x);
    //         }
    //     }
    //     let et2 = Instant::now();
    //     println!("HashMap traversal time: {:?}", et2 - st2);

    //     // 防止编译器优化
    //     black_box(sum);
    //     black_box(sum2);
    //     black_box(sum3);
    // }
}
