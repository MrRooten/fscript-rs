use std::{
    any::Any,
    fmt::{Debug, Formatter},
    sync::{atomic::{AtomicUsize, Ordering}, Arc},
};

use ahash::AHashMap;
use indexmap::{map::Iter, IndexMap};
use smallvec::SmallVec;

use crate::{
    backend::{
        compiler::bytecode::BinaryOffset,
        memory::GarbageCollector,
        types::{
            any::{ExtensionTrait, FSRExtension},
            base::{Area, AtomicObjId, FSRObject, FSRRetValue, FSRValue, GlobalObj, ObjId},
            class::FSRClass,
            error::FSRException,
            fn_def::FSRFn,
            iterator::{FSRInnerIterator, FSRIterator, FSRIteratorReferences},
            list::FSRList, string::FSRInnerString,
        },
        vm::{thread::FSRThreadRuntime, virtual_machine::get_object_by_global_id},
    },
    utils::error::FSRError,
};

const MAX_SEGMENT_SIZE: usize = 409600;

struct SegmentHashSet {
    // is_dirty: bool,
    // area: Area,
    hashset: IndexMap<u64, SmallVec<[(AtomicObjId); 1]>, ahash::RandomState>,
}

impl Debug for SegmentHashSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SegmentHashSet")
            // .field("is_dirty", &self.is_dirty)
            .finish()
    }
}

impl SegmentHashSet {
    pub fn new() -> Self {
        Self {
            // is_dirty: true,
            hashset: IndexMap::with_hasher(ahash::RandomState::new()), // area: Area::Minjor,
        }
    }

    pub fn len(&self) -> usize {
        self.hashset.len()
    }

    pub fn get(&self, key: u64) -> Option<&SmallVec<[(AtomicObjId); 1]>> {
        self.hashset.get(&key)
    }

    pub fn get_mut(&mut self, key: u64) -> Option<&mut SmallVec<[(AtomicObjId); 1]>> {
        self.hashset.get_mut(&key)
    }

    pub fn insert(&mut self, key: u64, value: SmallVec<[(AtomicObjId); 1]>) {
        self.hashset.insert(key, value);
    }

    pub fn remove(&mut self, key: u64) {
        self.hashset.swap_remove(&key);
    }

    pub fn clear(&mut self) {
        self.hashset.clear();
    }
    // pub fn is_dirty(&self) -> bool {
    //     self.is_dirty
    // }
}

pub struct FSRHashSet {
    // inner_map: AHashMap<u64, Vec<(AtomicObjId, AtomicObjId)>>,
    segment_map: Vec<SegmentHashSet>,
}

impl Debug for FSRHashSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FSRHashMap")
            .field("inner_map", &self.segment_map)
            .finish()
    }
}

impl ExtensionTrait for FSRHashSet {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_reference<'a>(
        &'a self,
        full: bool,
        worklist: &mut Vec<ObjId>,
        is_add: &mut bool,
    ) -> Box<dyn Iterator<Item = ObjId> + 'a> {
        //let mut v = Vec::with_capacity(self.len() * 2);
        for segment in self.segment_map.iter() {
            for (_, vec) in segment.hashset.iter() {
                for (key) in vec.iter() {
                    //v.push(key.load(Ordering::Relaxed));
                    //v.push(value.load(Ordering::Relaxed));
                    #[allow(clippy::never_loop)]
                    loop {
                        let ref_id = key.load(Ordering::Relaxed);
                        let obj = FSRObject::id_to_obj(ref_id);
                        if obj.area == Area::Minjor {
                            *is_add = true;
                        } else if full {
                            break;
                        }

                        if !obj.is_marked() {
                            worklist.push(ref_id);
                        }
                        break;
                    }

                    // {
                    //     let ref_id = value.load(Ordering::Relaxed);
                    //     let obj = FSRObject::id_to_obj(ref_id);
                    //     if obj.area == Area::Minjor {
                    //         *is_add = true;
                    //     } else if !full {
                    //         continue;
                    //     }

                    //     if !obj.is_marked() {
                    //         worklist.push(ref_id);
                    //     }
                    // }
                }
            }
        }

        Box::new(std::iter::empty())
    }

    fn set_undirty(&mut self) {
        // for segment in self.segment_map.iter_mut() {
        //     segment.is_dirty = false;
        // }
    }
}

type HashSetIterType<'a> = Iter<'a, u64, SmallVec<[(AtomicObjId); 1]>>;

struct FSRHashMapRefIterator<'a> {
    hashset: &'a FSRHashSet,
    segment_idx: usize,
    vec_iter: Option<std::slice::Iter<'a, (AtomicObjId)>>,
    hash_iter: Option<HashSetIterType<'a>>,
    current_pair: Option<&'a (AtomicObjId)>,
    yield_key: bool,
}

impl<'a> FSRHashMapRefIterator<'a> {
    fn new(hashset: &'a FSRHashSet) -> Self {
        let mut iter = Self {
            hashset,
            segment_idx: 0,
            vec_iter: None,
            hash_iter: None,
            current_pair: None,
            yield_key: true,
        };

        // 初始化第一个segment的迭代器
        if !hashset.segment_map.is_empty() {
            iter.hash_iter = Some(hashset.segment_map[0].hashset.iter());
        }

        // 初始化第一个vec迭代器
        iter.advance_hash_iterator();

        iter
    }

    fn advance_hash_iterator(&mut self) -> bool {
        if let Some(hash_iter) = &mut self.hash_iter {
            if let Some((_, vec)) = hash_iter.next() {
                self.vec_iter = Some(vec.iter());
                self.advance_vec_iterator();
                return true;
            }
        }

        // 尝试移动到下一个segment
        self.segment_idx += 1;
        if self.segment_idx < self.hashset.segment_map.len() {
            self.hash_iter = Some(self.hashset.segment_map[self.segment_idx].hashset.iter());
            self.advance_hash_iterator()
        } else {
            self.vec_iter = None;
            self.current_pair = None;
            false
        }
    }

    fn advance_vec_iterator(&mut self) -> bool {
        if let Some(vec_iter) = &mut self.vec_iter {
            if let Some(pair) = vec_iter.next() {
                self.current_pair = Some(pair);
                self.yield_key = true;
                return true;
            }
        }

        // 尝试移动到下一个hashset条目
        self.advance_hash_iterator()
    }
}

impl Iterator for FSRHashMapRefIterator<'_> {
    type Item = ObjId;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(pair) = self.current_pair {
            if self.yield_key {
                self.yield_key = false;
                return Some(pair.load(Ordering::Relaxed));
            } else {
                let value = pair.load(Ordering::Relaxed);
                self.advance_vec_iterator();
                return Some(value);
            }
        }

        None
    }
}


pub struct FSRHashMapIterator<'a> {
    pub(crate) list_obj: ObjId,
    pub(crate) iter: Box<dyn Iterator<Item = (ObjId)> + Send + 'a>,
}

impl FSRIteratorReferences for FSRHashMapIterator<'_> {
    fn ref_objects(&self) -> Vec<ObjId> {
        vec![self.list_obj]
    }
}

impl FSRIterator for FSRHashMapIterator<'_> {
    fn next(&mut self, thread: &mut FSRThreadRuntime) -> Result<Option<ObjId>, FSRError> {
        let c = self.iter.next();
        // c.map(|x| {
        //     let vs = vec![x.0, x.1];
        //     let list = FSRList::new_value(vs);
        //     let ret = thread
        //         .garbage_collect
        //         .new_object(list, get_object_by_global_id(FSRGlobalObjId::ListCls) as ObjId);
        //     Ok(ret)
        // })
        if let Some((key)) = c {
            Ok(Some(key))
        } else {
            Ok(None)
        }
    }
}

pub fn fsr_fn_hashset_iter(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let hashset = FSRObject::id_to_obj(args[0]);
    if let FSRValue::Extension(any) = &hashset.value {
        if let Some(hashset) = any.value.as_any().downcast_ref::<FSRHashSet>() {
            let iter = hashset
                .segment_map
                .iter()
                .flat_map(|s| s.hashset.iter())
                .flat_map(|(k, v)| {
                    v.iter().map(move |(key)| {
                        (key.load(Ordering::Relaxed))
                    })
                });
            let iter_obj = FSRHashMapIterator {
                list_obj: args[0],
                iter: Box::new(iter),
            };
            let object = thread.garbage_collect.new_object(
                FSRValue::Iterator(Box::new(FSRInnerIterator {
                    obj: args[0],
                    iterator: Some(Box::new(iter_obj)),
                })),
                get_object_by_global_id(GlobalObj::InnerIterator),
            );
            Ok(FSRRetValue::GlobalId(object))
        } else {
            unimplemented!()
        }
    } else {
        unimplemented!()
    }
}

pub fn fsr_fn_hashset_new(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let hashset = FSRHashSet::new_hashset();
    let object = thread
        .garbage_collect
        .new_object(hashset.to_any_type(), get_object_by_global_id(GlobalObj::HashSetCls));
    Ok(FSRRetValue::GlobalId(object))
}

/// Insert a key-value pair into the hashset
/// accepts 3 arguments
/// 1. hashset object
/// 2. key
/// 3. value
pub fn fsr_fn_hashset_insert(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    if args.len() != 2 {
        return Err(FSRError::new(
            "not valid args",
            crate::utils::error::FSRErrCode::RuntimeError,
        ));
    }
    let hashset = FSRObject::id_to_mut_obj(args[0]).expect("msg: not a any and hashset");
    let key = args[1];
    if hashset.area.is_long() {
        let key_obj = FSRObject::id_to_obj(key);
        if key_obj.area == Area::Minjor {
            hashset.set_write_barrier(true);
        }
    }
    if let FSRValue::Extension(any) = &mut hashset.value {
        if let Some(hashset) = any.value.as_any_mut().downcast_mut::<FSRHashSet>() {
            hashset.insert(key, thread)?;
        } else {
            unimplemented!()
        }
    } else {
        unimplemented!()
    }
    Ok(FSRRetValue::GlobalId(FSRObject::none_id()))
}

pub fn fsr_fn_hashset_get(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let hashset = FSRObject::id_to_obj(args[0]);
    let key = args[1];

    if let FSRValue::Extension(any) = &hashset.value {
        if let Some(hashset) = any.value.as_any().downcast_ref::<FSRHashSet>() {
            if let Some(value) = hashset.get(key, thread) {
                return Ok(FSRRetValue::GlobalId(
                    value.load(std::sync::atomic::Ordering::Relaxed),
                ));
            }
        } else {
            unimplemented!()
        }
    } else {
        unimplemented!()
    }
    Ok(FSRRetValue::GlobalId(FSRObject::none_id()))
}

pub fn fsr_fn_hashset_get_reference(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let hashset_obj = FSRObject::id_to_mut_obj(args[0]).expect("msg: not a any and hashset");
    let key = args[1];
    let mut flag = false;
    if let FSRValue::Extension(any) = &hashset_obj.value {
        if let Some(hashset) = any.value.as_any().downcast_ref::<FSRHashSet>() {
            if let Some(value) = hashset.get(key, thread) {
                return Ok(FSRRetValue::GlobalId(value.load(Ordering::Relaxed)));
            }
        }
    } else {
        unimplemented!()
    }
    Ok(FSRRetValue::GlobalId(FSRObject::none_id()))
}

pub fn fsr_fn_hashset_contains(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let hashset = FSRObject::id_to_obj(args[0]);
    let key = args[1];

    if let FSRValue::Extension(any) = &hashset.value {
        if let Some(hashset) = any.value.as_any().downcast_ref::<FSRHashSet>() {
            if hashset.get(key, thread).is_some() {
                return Ok(FSRRetValue::GlobalId(FSRObject::true_id()));
            }
        } else {
            unimplemented!()
        }
    } else {
        unimplemented!()
    }
    Ok(FSRRetValue::GlobalId(FSRObject::false_id()))
}

pub fn fsr_fn_hashset_remove(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let hashset = FSRObject::id_to_mut_obj(args[0]).expect("msg: not a any and hashset");
    let key = args[1];

    if let FSRValue::Extension(any) = &mut hashset.value {
        if let Some(hashset) = any.value.as_any_mut().downcast_mut::<FSRHashSet>() {
            hashset.remove(key, thread);
        } else {
            unimplemented!()
        }
    } else {
        unimplemented!()
    }
    Ok(FSRRetValue::GlobalId(FSRObject::none_id()))
}

fn hashset_string(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId,
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let mut s = FSRInnerString::new("HashSet");
    s.push('(');
    let obj_id = args[0];
    let obj = FSRObject::id_to_obj(obj_id);
    if let FSRValue::Extension(l) = &obj.value {
        let l = l.value.as_any().downcast_ref::<FSRHashSet>()
            .ok_or(FSRError::new(
                "not a hashset",
                crate::utils::error::FSRErrCode::RuntimeError,
            ))?;
        
        let mut vs = vec![];
        for seg in l.segment_map.iter() {
            for (hash_id, bucket) in seg.hashset.iter() {
                for item in bucket.iter() {
                    let item_id = item.load(Ordering::Relaxed);
                    let item_obj = FSRObject::id_to_obj(item_id);
                    let s_value = item_obj.to_string(thread, code);
                    if let FSRValue::String(v) = &s_value {
                        vs.push(format!("{}", v));
                    } else {
                        return Err(FSRError::new(
                            "HashSet contains non-string value",
                            crate::utils::error::FSRErrCode::RuntimeError,
                        ));
                    }
                }
            }
        }

        let join_str = vs.join(", ");
        if !join_str.is_empty() {
            s.push_str(&join_str);
        }
    } else {
        return Err(FSRError::new(
            "not a hashset",
            crate::utils::error::FSRErrCode::RuntimeError,
        ));
    }

    s.push(')');
    let obj_id = thread.garbage_collect.new_object(
        FSRValue::String(Arc::new(s)),
        get_object_by_global_id(GlobalObj::StringCls),
    );
    Ok(FSRRetValue::GlobalId(obj_id))
}

impl FSRHashSet {
    pub fn new_hashset() -> Self {
        Self {
            segment_map: vec![SegmentHashSet::new()],
        }
    }

    pub fn to_any_type(self) -> FSRValue<'static> {
        FSRValue::Extension(Box::new(FSRExtension {
            value: Box::new(self),
        }))
    }

    pub fn len(&self) -> usize {
        self.segment_map.iter().map(|s| s.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get_item(&self, key: u64) -> Option<&SmallVec<[(AtomicObjId); 1]>> {
        for segment in self.segment_map.iter() {
            if let Some(value) = segment.get(key) {
                return Some(value);
            }
        }
        None
    }

    pub fn get_mut(&mut self, key: u64) -> Option<&mut SmallVec<[(AtomicObjId); 1]>> {
        for segment in self.segment_map.iter_mut() {
            if let Some(value) = segment.get_mut(key) {
                return Some(value);
            }
        }
        None
    }

    pub fn insert_item(&mut self, hash: u64, key: ObjId) -> Option<()> {
        for segment in self.segment_map.iter_mut() {
            if segment.len() < MAX_SEGMENT_SIZE {
                segment.insert(
                    hash,
                    [(AtomicObjId::new(key))].into(),
                );
                // segment.is_dirty = true;
                return Some(());
            }
        }

        let mut new_segment = SegmentHashSet::new();
        new_segment.insert(
            hash,
            [(AtomicObjId::new(key))].into(),
        );
        // new_segment.is_dirty = true;
        self.segment_map.push(new_segment);

        Some(())
    }

    pub fn try_insert_item(&mut self, hash: u64, key: ObjId) -> Option<()> {
        for segment in self.segment_map.iter_mut() {
            if segment.len() < MAX_SEGMENT_SIZE {
                // segment.insert(
                //     hash,
                //     [(AtomicObjId::new(key), AtomicObjId::new(value))].into(),
                // );

                match segment.hashset.entry(hash) {
                    indexmap::map::Entry::Occupied(occupied_entry) => return None,
                    indexmap::map::Entry::Vacant(vacant_entry) => {
                        vacant_entry.insert([(AtomicObjId::new(key))].into());
                    },
                };
                // segment.is_dirty = true;
                return Some(());
            }
        }

        let mut new_segment = SegmentHashSet::new();
        new_segment.insert(
            hash,
            [(AtomicObjId::new(key))].into(),
        );
        // new_segment.is_dirty = true;
        self.segment_map.push(new_segment);

        Some(())
    }

    pub fn remove_item(&mut self, key: u64) {
        for segment in self.segment_map.iter_mut() {
            if segment.hashset.contains_key(&key) {
                // segment.is_dirty = true;
            } else {
                continue;
            }
            segment.remove(key);
        }
    }

    fn call_hash(
        key: ObjId,
        thread: &mut FSRThreadRuntime,
    ) -> Result<u64, FSRError> {
        let obj = FSRObject::id_to_obj(key);
        if let FSRValue::Integer(i) = obj.value {
            return Ok(i as u64);
        }

        let key_obj = FSRObject::id_to_obj(key);
        let cls = key_obj.cls;
        let hash_fn_id = key_obj
            .get_cls_offset_attr(BinaryOffset::Hash)
            .unwrap()
            .load(std::sync::atomic::Ordering::Relaxed);
        let hash_fn = FSRObject::id_to_obj(hash_fn_id);
        let hash = hash_fn.call(&[key], thread, 0)?;
        let hash_id = FSRObject::id_to_obj(hash.get_id());
        let hash = if let FSRValue::Integer(i) = &hash_id.value {
            *i as u64
        } else {
            unimplemented!()
        };

        Ok(hash)
    }


    pub fn insert(
        &mut self,
        key: ObjId,
        thread: &mut FSRThreadRuntime,
    ) -> Result<(), FSRError> {
        let hash = Self::call_hash(key, thread)?;

        // if let None = self.get_mut(hash) {
        if self.try_insert_item(hash, key).is_some() {
            return Ok(());
        }

        let res = {
            let res = self.get_mut(hash).unwrap();
            for save_item in res.iter() {
                let eq_fn_id = FSRObject::id_to_obj(key)
                    .get_cls_offset_attr(BinaryOffset::Equal)
                    .unwrap()
                    .load(std::sync::atomic::Ordering::Relaxed);
                let eq_fn = FSRObject::id_to_obj(eq_fn_id);
                let is_same = eq_fn
                    .call(&[key, save_item.load(Ordering::Relaxed)], thread, 0)?
                    .get_id();

                if is_same == FSRObject::true_id() {
                    save_item
                        .store(key, std::sync::atomic::Ordering::Relaxed);
                    return Ok(());
                }
            }
            res
        };

        res.push((AtomicObjId::new(key)));

        Ok(())
    }

    pub fn get(&self, key: ObjId, thread: &mut FSRThreadRuntime) -> Option<&AtomicObjId> {
        let hash = Self::call_hash(key, thread).unwrap();

        let res = self.get_item(hash)?;
        for save_item in res.iter() {
            let save_key = save_item.load(std::sync::atomic::Ordering::Relaxed);
            if save_key == key {
                return Some(&save_item);
            }

            let eq_fn_id = FSRObject::id_to_obj(save_key)
                .get_cls_offset_attr(BinaryOffset::Equal)
                .unwrap()
                .load(std::sync::atomic::Ordering::Relaxed);
            let eq_fn = FSRObject::id_to_obj(eq_fn_id);
            let is_same = eq_fn
                .call(&[save_key, key], thread, 0)
                .unwrap()
                .get_id();

            if is_same == FSRObject::true_id() {
                return Some(&save_item);
            }
        }
        None
    }

    pub fn remove(&mut self, key: ObjId, thread: &mut FSRThreadRuntime) {
        let key_obj = FSRObject::id_to_obj(key);
        let hash_fn_id = key_obj
            .get_cls_offset_attr(BinaryOffset::Hash)
            .unwrap()
            .load(std::sync::atomic::Ordering::Relaxed);

        let hash_fn = FSRObject::id_to_obj(hash_fn_id);
        let hash = hash_fn.call(&[key], thread, 0).unwrap();
        let hash_id = FSRObject::id_to_obj(hash.get_id());
        let hash = if let FSRValue::Integer(i) = &hash_id.value {
            *i as u64
        } else {
            // unimplemented!()
            panic!("Hash function must return an integer");
        };

        let res = if let Some(s) = self.get_item(hash) {
            s
        } else {
            return;
        };

        let len = res.len();
        if len == 1 {
            self.remove_item(hash);
            return;
        }
        let res = self.get_mut(hash).unwrap();
        for i in 0..len {
            let save_item = &res[i];
            let save_key = save_item.load(std::sync::atomic::Ordering::Relaxed);
            if save_key == key {
                res.remove(i);
                return;
            }

            let eq_fn_id = FSRObject::id_to_obj(save_key)
                .get_cls_offset_attr(BinaryOffset::Equal)
                .unwrap()
                .load(std::sync::atomic::Ordering::Relaxed);
            let eq_fn = FSRObject::id_to_obj(eq_fn_id);
            let is_same = eq_fn
                .call(&[save_key, key], thread, 0)
                .unwrap()
                .get_id();
            if is_same == FSRObject::true_id() {
                res.remove(i);
                return;
            }
        }
    }

    pub fn get_class() -> FSRClass {
        let mut cls = FSRClass::new("HashSet");
        // let len_m = FSRFn::from_rust_fn_static(string_len, "string_len");
        // cls.insert_attr("len", len_m);
        // let add_fn = FSRFn::from_rust_fn_static(add, "string_add");
        // //cls.insert_attr("__add__", add_fn);
        // cls.insert_offset_attr(BinaryOffset::Add, add_fn);
        let new = FSRFn::from_rust_fn_static(fsr_fn_hashset_new, "new");
        cls.insert_attr("new", new);
        let insert = FSRFn::from_rust_fn_static(fsr_fn_hashset_insert, "insert");
        cls.insert_attr("insert", insert);
        let set_item = FSRFn::from_rust_fn_static(fsr_fn_hashset_insert, "hashset__setitem__");
        cls.insert_offset_attr(BinaryOffset::SetItem, set_item);
        let iter = FSRFn::from_rust_fn_static(fsr_fn_hashset_iter, "iter");
        cls.insert_attr("__iter__", iter);
        let contains = FSRFn::from_rust_fn_static(fsr_fn_hashset_contains, "contains");
        cls.insert_attr("contains", contains);
        let remove = FSRFn::from_rust_fn_static(fsr_fn_hashset_remove, "remove");
        cls.insert_attr("remove", remove);
        let get_item_ref =
            FSRFn::from_rust_fn_static(fsr_fn_hashset_get_reference, "__getitem__ref");
        cls.insert_offset_attr(BinaryOffset::GetItem, get_item_ref);
        let to_str = FSRFn::from_rust_fn_static(hashset_string, "to_string");
        cls.insert_attr("__str__", to_str);
        
        cls
    }
}
