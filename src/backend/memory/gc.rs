use std::collections::LinkedList;

use crate::backend::types::base::{FSRObject, ObjId};

use super::size_alloc::FSRObjectAllocator;

pub struct ObjectGeneration {
    
}

pub struct GarbageCollector<'a> {
    allocator: FSRObjectAllocator<'a>,
    objects: LinkedList<Box<FSRObject<'a>>>,
    object_map: Vec<bool>,
    clear_list: Vec<ObjId>,
}

impl<'a> GarbageCollector<'a> {
    pub fn new(allocator: FSRObjectAllocator<'a>) -> Self {
        Self {
            allocator,
            objects: LinkedList::new(),
            clear_list: Vec::new(),
            object_map: vec![],
        }
    }

    pub fn new_object(&mut self, obj: Box<FSRObject<'a>>) -> ObjId {
        let id = FSRObject::obj_to_id(&obj);
        self.objects.push_back(obj);
        id
    }

    pub fn iter_from_obj(&self, obj: ObjId, store: &mut Vec<ObjId>) {
        unimplemented!()
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
