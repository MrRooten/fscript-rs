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
            any::{AnyDebugSend, AnyType, GetReference},
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

struct SegmentHashMap {
    // is_dirty: bool,
    // area: Area,
    hashmap: IndexMap<u64, SmallVec<[(AtomicObjId, AtomicObjId); 1]>, ahash::RandomState>,
}

impl Debug for SegmentHashMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SegmentHashMap")
            // .field("is_dirty", &self.is_dirty)
            .finish()
    }
}

impl SegmentHashMap {
    pub fn new() -> Self {
        Self {
            // is_dirty: true,
            hashmap: IndexMap::with_hasher(ahash::RandomState::new()), // area: Area::Minjor,
        }
    }

    pub fn len(&self) -> usize {
        self.hashmap.len()
    }

    pub fn get(&self, key: u64) -> Option<&SmallVec<[(AtomicObjId, AtomicObjId); 1]>> {
        self.hashmap.get(&key)
    }

    pub fn get_mut(&mut self, key: u64) -> Option<&mut SmallVec<[(AtomicObjId, AtomicObjId); 1]>> {
        self.hashmap.get_mut(&key)
    }

    pub fn insert(&mut self, key: u64, value: SmallVec<[(AtomicObjId, AtomicObjId); 1]>) {
        self.hashmap.insert(key, value);
    }

    pub fn remove(&mut self, key: u64) {
        self.hashmap.swap_remove(&key);
    }

    pub fn clear(&mut self) {
        self.hashmap.clear();
    }
    // pub fn is_dirty(&self) -> bool {
    //     self.is_dirty
    // }
}

pub struct FSRHashMap {
    // inner_map: AHashMap<u64, Vec<(AtomicObjId, AtomicObjId)>>,
    segment_map: Vec<SegmentHashMap>,
}

impl Debug for FSRHashMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FSRHashMap")
            .field("inner_map", &self.segment_map)
            .finish()
    }
}

impl AnyDebugSend for FSRHashMap {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

type HashMapIterType<'a> = Iter<'a, u64, SmallVec<[(AtomicObjId, AtomicObjId); 1]>>;

struct FSRHashMapRefIterator<'a> {
    hashmap: &'a FSRHashMap,
    segment_idx: usize,
    vec_iter: Option<std::slice::Iter<'a, (AtomicObjId, AtomicObjId)>>,
    hash_iter: Option<HashMapIterType<'a>>,
    current_pair: Option<&'a (AtomicObjId, AtomicObjId)>,
    yield_key: bool,
}

impl<'a> FSRHashMapRefIterator<'a> {
    fn new(hashmap: &'a FSRHashMap) -> Self {
        let mut iter = Self {
            hashmap,
            segment_idx: 0,
            vec_iter: None,
            hash_iter: None,
            current_pair: None,
            yield_key: true,
        };

        // 初始化第一个segment的迭代器
        if !hashmap.segment_map.is_empty() {
            iter.hash_iter = Some(hashmap.segment_map[0].hashmap.iter());
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
        if self.segment_idx < self.hashmap.segment_map.len() {
            self.hash_iter = Some(self.hashmap.segment_map[self.segment_idx].hashmap.iter());
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

        // 尝试移动到下一个hashmap条目
        self.advance_hash_iterator()
    }
}

impl Iterator for FSRHashMapRefIterator<'_> {
    type Item = ObjId;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(pair) = self.current_pair {
            if self.yield_key {
                self.yield_key = false;
                return Some(pair.0.load(Ordering::Relaxed));
            } else {
                let value = pair.1.load(Ordering::Relaxed);
                self.advance_vec_iterator();
                return Some(value);
            }
        }

        None
    }
}

impl GetReference for FSRHashMap {
    /// Try to process in here instead of return a iterator
    fn get_reference<'a>(
        &'a self,
        full: bool,
        worklist: &mut Vec<ObjId>,
        is_add: &mut bool,
    ) -> Box<dyn Iterator<Item = ObjId> + 'a> {
        //let mut v = Vec::with_capacity(self.len() * 2);
        for segment in self.segment_map.iter() {
            for (_, vec) in segment.hashmap.iter() {
                for (key, value) in vec.iter() {
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

                    {
                        let ref_id = value.load(Ordering::Relaxed);
                        let obj = FSRObject::id_to_obj(ref_id);
                        if obj.area == Area::Minjor {
                            *is_add = true;
                        } else if !full {
                            continue;
                        }

                        if !obj.is_marked() {
                            worklist.push(ref_id);
                        }
                    }
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

pub struct FSRHashMapIterator<'a> {
    pub(crate) list_obj: ObjId,
    pub(crate) iter: Box<dyn Iterator<Item = (ObjId, ObjId)> + Send + 'a>,
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
        if let Some((key, value)) = c {
            let vs = vec![key, value];
            let list = FSRList::new_value(vs);
            let ret = thread
                .garbage_collect
                .new_object(list, get_object_by_global_id(GlobalObj::ListCls) as ObjId);
            Ok(Some(ret))
        } else {
            Ok(None)
        }
    }
}

pub fn fsr_fn_hashmap_iter(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let hashmap = FSRObject::id_to_obj(args[0]);
    if let FSRValue::Any(any) = &hashmap.value {
        if let Some(hashmap) = any.value.as_any().downcast_ref::<FSRHashMap>() {
            let iter = hashmap
                .segment_map
                .iter()
                .flat_map(|s| s.hashmap.iter())
                .flat_map(|(k, v)| {
                    v.iter().map(move |(key, value)| {
                        (key.load(Ordering::Relaxed), value.load(Ordering::Relaxed))
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

pub fn fsr_fn_hashmap_new(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let hashmap = FSRHashMap::new_hashmap();
    let object = thread
        .garbage_collect
        .new_object(hashmap.to_any_type(), get_object_by_global_id(GlobalObj::HashMapCls));
    Ok(FSRRetValue::GlobalId(object))
}

/// Insert a key-value pair into the hashmap
/// accepts 3 arguments
/// 1. hashmap object
/// 2. key
/// 3. value
pub fn fsr_fn_hashmap_insert(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    if args.len() != 3 {
        return Err(FSRError::new(
            "not valid args",
            crate::utils::error::FSRErrCode::RuntimeError,
        ));
    }
    let hashmap = FSRObject::id_to_mut_obj(args[0]).expect("msg: not a any and hashmap");
    let key = args[1];
    let value = args[2];
    if hashmap.area.is_long() {
        let key_obj = FSRObject::id_to_obj(key);
        if key_obj.area == Area::Minjor {
            hashmap.set_write_barrier(true);
        }

        let value_obj = FSRObject::id_to_obj(value);
        if value_obj.area == Area::Minjor {
            hashmap.set_write_barrier(true);
        }
    }
    if let FSRValue::Any(any) = &mut hashmap.value {
        if let Some(hashmap) = any.value.as_any_mut().downcast_mut::<FSRHashMap>() {
            hashmap.insert(key, value, thread)?;
        } else {
            unimplemented!()
        }
    } else {
        unimplemented!()
    }
    Ok(FSRRetValue::GlobalId(FSRObject::none_id()))
}

pub fn fsr_fn_hashmap_get(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let hashmap = FSRObject::id_to_obj(args[0]);
    let key = args[1];

    if let FSRValue::Any(any) = &hashmap.value {
        if let Some(hashmap) = any.value.as_any().downcast_ref::<FSRHashMap>() {
            if let Some(value) = hashmap.get(key, thread) {
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

fn hashmap_string(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId,
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let mut s = FSRInnerString::new("HashMap");
    s.push('(');
    let obj_id = args[0];
    let obj = FSRObject::id_to_obj(obj_id);
    if let FSRValue::Any(l) = &obj.value {
        let l = l.value.as_any().downcast_ref::<FSRHashMap>()
            .ok_or(FSRError::new(
                "not a hashset",
                crate::utils::error::FSRErrCode::RuntimeError,
            ))?;
        
        let mut vs = vec![];
        for seg in l.segment_map.iter() {
            for (hash_id, bucket) in seg.hashmap.iter() {
                for (key, value) in bucket.iter() {
                    let key_obj = FSRObject::id_to_obj(key.load(Ordering::Relaxed));
                    let value_obj = FSRObject::id_to_obj(value.load(Ordering::Relaxed));
                    let key_str = key_obj.to_string(thread, code);
                    let value_str = value_obj.to_string(thread, code);
                    if let FSRValue::String(k) = &key_str {
                        if let FSRValue::String(v) = &value_str {
                            vs.push(format!("{} => {}", k, v));
                        } else {
                            return Err(FSRError::new(
                                "HashMap contains non-string value",
                                crate::utils::error::FSRErrCode::RuntimeError,
                            ));
                        }
                    } else {
                        return Err(FSRError::new(
                            "HashMap contains non-string key",
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

pub fn fsr_fn_hashmap_get_reference(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let hashmap_obj = FSRObject::id_to_mut_obj(args[0]).expect("msg: not a any and hashmap");
    let key = args[1];
    let mut flag = false;
    if let FSRValue::Any(any) = &hashmap_obj.value {
        if let Some(hashmap) = any.value.as_any().downcast_ref::<FSRHashMap>() {
            if let Some(value) = hashmap.get(key, thread) {
                return Ok(FSRRetValue::GlobalId(value.load(Ordering::Relaxed)));
            }
        }
    } else {
        unimplemented!()
    }
    Ok(FSRRetValue::GlobalId(FSRObject::none_id()))
}

pub fn fsr_fn_hashmap_contains(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let hashmap = FSRObject::id_to_obj(args[0]);
    let key = args[1];

    if let FSRValue::Any(any) = &hashmap.value {
        if let Some(hashmap) = any.value.as_any().downcast_ref::<FSRHashMap>() {
            if hashmap.get(key, thread).is_some() {
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

pub fn fsr_fn_hashmap_remove(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let hashmap = FSRObject::id_to_mut_obj(args[0]).expect("msg: not a any and hashmap");
    let key = args[1];

    if let FSRValue::Any(any) = &mut hashmap.value {
        if let Some(hashmap) = any.value.as_any_mut().downcast_mut::<FSRHashMap>() {
            hashmap.remove(key, thread);
        } else {
            unimplemented!()
        }
    } else {
        unimplemented!()
    }
    Ok(FSRRetValue::GlobalId(FSRObject::none_id()))
}

impl FSRHashMap {
    pub fn new_hashmap() -> Self {
        Self {
            segment_map: vec![SegmentHashMap::new()],
        }
    }

    pub fn to_any_type(self) -> FSRValue<'static> {
        FSRValue::Any(Box::new(AnyType {
            value: Box::new(self),
        }))
    }

    pub fn len(&self) -> usize {
        self.segment_map.iter().map(|s| s.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get_item(&self, key: u64) -> Option<&SmallVec<[(AtomicObjId, AtomicObjId); 1]>> {
        for segment in self.segment_map.iter() {
            if let Some(value) = segment.get(key) {
                return Some(value);
            }
        }
        None
    }

    pub fn get_mut(&mut self, key: u64) -> Option<&mut SmallVec<[(AtomicObjId, AtomicObjId); 1]>> {
        for segment in self.segment_map.iter_mut() {
            if let Some(value) = segment.get_mut(key) {
                return Some(value);
            }
        }
        None
    }

    pub fn insert_item(&mut self, hash: u64, key: ObjId, value: ObjId) -> Option<()> {
        for segment in self.segment_map.iter_mut() {
            if segment.len() < MAX_SEGMENT_SIZE {
                segment.insert(
                    hash,
                    [(AtomicObjId::new(key), AtomicObjId::new(value))].into(),
                );
                // segment.is_dirty = true;
                return Some(());
            }
        }

        let mut new_segment = SegmentHashMap::new();
        new_segment.insert(
            hash,
            [(AtomicObjId::new(key), AtomicObjId::new(value))].into(),
        );
        // new_segment.is_dirty = true;
        self.segment_map.push(new_segment);

        Some(())
    }

    pub fn try_insert_item(&mut self, hash: u64, key: ObjId, value: ObjId) -> Option<()> {
        for segment in self.segment_map.iter_mut() {
            if segment.len() < MAX_SEGMENT_SIZE {
                // segment.insert(
                //     hash,
                //     [(AtomicObjId::new(key), AtomicObjId::new(value))].into(),
                // );

                match segment.hashmap.entry(hash) {
                    indexmap::map::Entry::Occupied(occupied_entry) => return None,
                    indexmap::map::Entry::Vacant(vacant_entry) => {
                        vacant_entry.insert([(AtomicObjId::new(key), AtomicObjId::new(value))].into());
                    },
                };
                // segment.is_dirty = true;
                return Some(());
            }
        }

        let mut new_segment = SegmentHashMap::new();
        new_segment.insert(
            hash,
            [(AtomicObjId::new(key), AtomicObjId::new(value))].into(),
        );
        // new_segment.is_dirty = true;
        self.segment_map.push(new_segment);

        Some(())
    }

    pub fn remove_item(&mut self, key: u64) {
        for segment in self.segment_map.iter_mut() {
            if segment.hashmap.contains_key(&key) {
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
        let hash = hash_fn.call(&[key], thread, 0, hash_fn_id)?;
        let hash_id = FSRObject::id_to_obj(hash.get_id());
        let hash = if let FSRValue::Integer(i) = &hash_id.value {
            *i as u64
        } else {
            unimplemented!()
        };

        Ok(hash)
    }

    pub fn try_insert_if_not_exist(
        &mut self,
        key: ObjId,
        value: ObjId,
        thread: &mut FSRThreadRuntime,
    ) -> Result<(), FSRError> {
        let hash = Self::call_hash(key, thread)?;

        if self.get_item(hash).is_none() {
            self.insert_item(hash, key, value);
            return Ok(());
        }
        let res = {
            let res = self.get_mut(hash).unwrap();
            for save_item in res.iter() {
                let save_key = save_item.0.load(std::sync::atomic::Ordering::Relaxed);
                if save_key == key {
                    // save_item
                    //     .1
                    //     .store(value, std::sync::atomic::Ordering::Relaxed);
                    return Ok(());
                }

                let eq_fn_id = FSRObject::id_to_obj(save_key)
                    .get_cls_offset_attr(BinaryOffset::Equal)
                    .unwrap()
                    .load(std::sync::atomic::Ordering::Relaxed);
                let eq_fn = FSRObject::id_to_obj(eq_fn_id);
                let is_same = eq_fn
                    .call(&[save_key, value], thread, 0, eq_fn_id)?
                    .get_id();

                if is_same == FSRObject::true_id() {
                    // save_item
                    //     .1
                    //     .store(value, std::sync::atomic::Ordering::Relaxed);
                    return Ok(());
                }
            }
            res
        };

        res.push((AtomicObjId::new(key), AtomicObjId::new(value)));

        Ok(())
    }

    pub fn insert(
        &mut self,
        key: ObjId,
        value: ObjId,
        thread: &mut FSRThreadRuntime,
    ) -> Result<(), FSRError> {
        let hash = Self::call_hash(key, thread)?;

        // if let None = self.get_mut(hash) {
        if self.try_insert_item(hash, key, value).is_some() {
            return Ok(());
        }

        let res = {
            let res = self.get_mut(hash).unwrap();
            for save_item in res.iter() {
                let save_key = save_item.0.load(std::sync::atomic::Ordering::Relaxed);
                if save_key == key {
                    save_item
                        .1
                        .store(value, std::sync::atomic::Ordering::Relaxed);
                    return Ok(());
                }

                let eq_fn_id = FSRObject::id_to_obj(save_key)
                    .get_cls_offset_attr(BinaryOffset::Equal)
                    .unwrap()
                    .load(std::sync::atomic::Ordering::Relaxed);
                let eq_fn = FSRObject::id_to_obj(eq_fn_id);
                let is_same = eq_fn
                    .call(&[save_key, value], thread, 0, eq_fn_id)?
                    .get_id();

                if is_same == FSRObject::true_id() {
                    save_item
                        .1
                        .store(value, std::sync::atomic::Ordering::Relaxed);
                    return Ok(());
                }
            }
            res
        };

        res.push((AtomicObjId::new(key), AtomicObjId::new(value)));

        Ok(())
    }

    pub fn get(&self, key: ObjId, thread: &mut FSRThreadRuntime) -> Option<&AtomicObjId> {
        let hash = Self::call_hash(key, thread).unwrap();

        let res = self.get_item(hash)?;
        for save_item in res.iter() {
            let save_key = save_item.0.load(std::sync::atomic::Ordering::Relaxed);
            if save_key == key {
                return Some(&save_item.1);
            }

            let eq_fn_id = FSRObject::id_to_obj(save_key)
                .get_cls_offset_attr(BinaryOffset::Equal)
                .unwrap()
                .load(std::sync::atomic::Ordering::Relaxed);
            let eq_fn = FSRObject::id_to_obj(eq_fn_id);
            let is_same = eq_fn
                .call(&[save_key, key], thread, 0, eq_fn_id)
                .unwrap()
                .get_id();

            if is_same == FSRObject::true_id() {
                return Some(&save_item.1);
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
        let hash = hash_fn.call(&[key], thread, 0, hash_fn_id).unwrap();
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
            let save_key = save_item.0.load(std::sync::atomic::Ordering::Relaxed);
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
                .call(&[save_key, key], thread, 0, eq_fn_id)
                .unwrap()
                .get_id();
            if is_same == FSRObject::true_id() {
                res.remove(i);
                return;
            }
        }
    }

    pub fn get_class() -> FSRClass<'static> {
        let mut cls = FSRClass::new("HashMap");
        // let len_m = FSRFn::from_rust_fn_static(string_len, "string_len");
        // cls.insert_attr("len", len_m);
        // let add_fn = FSRFn::from_rust_fn_static(add, "string_add");
        // //cls.insert_attr("__add__", add_fn);
        // cls.insert_offset_attr(BinaryOffset::Add, add_fn);
        let new = FSRFn::from_rust_fn_static(fsr_fn_hashmap_new, "new");
        cls.insert_attr("new", new);
        let insert = FSRFn::from_rust_fn_static(fsr_fn_hashmap_insert, "insert");
        cls.insert_attr("insert", insert);
        let set_item = FSRFn::from_rust_fn_static(fsr_fn_hashmap_insert, "hashmap__setitem__");
        cls.insert_offset_attr(BinaryOffset::SetItem, set_item);
        let get = FSRFn::from_rust_fn_static(fsr_fn_hashmap_get, "get");
        cls.insert_attr("get", get);
        let get_item = FSRFn::from_rust_fn_static(fsr_fn_hashmap_get, "__getitem__");
        cls.insert_offset_attr(BinaryOffset::GetItem, get_item);
        let iter = FSRFn::from_rust_fn_static(fsr_fn_hashmap_iter, "iter");
        cls.insert_attr("__iter__", iter);
        let contains = FSRFn::from_rust_fn_static(fsr_fn_hashmap_contains, "contains");
        cls.insert_attr("contains", contains);
        let remove = FSRFn::from_rust_fn_static(fsr_fn_hashmap_remove, "remove");
        cls.insert_attr("remove", remove);
        let get_item_ref =
            FSRFn::from_rust_fn_static(fsr_fn_hashmap_get_reference, "__getitem__ref");
        cls.insert_offset_attr(BinaryOffset::GetItem, get_item_ref);
        let to_str = FSRFn::from_rust_fn_static(hashmap_string, "to_string");
        cls.insert_attr("__str__", to_str);
        cls
    }
}
