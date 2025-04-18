use std::{
    any::Any, collections::HashMap, fmt::{Debug, Formatter}, sync::atomic::{AtomicUsize, Ordering}
};

use ahash::AHashMap;

use crate::{
    backend::{
        compiler::bytecode::BinaryOffset,
        memory::GarbageCollector,
        types::{
            any::{AnyDebugSend, AnyType, GetReference}, base::{Area, AtomicObjId, FSRGlobalObjId, FSRObject, FSRRetValue, FSRValue, ObjId}, class::FSRClass, error::FSRException, fn_def::FSRFn, iterator::{FSRInnerIterator, FSRIterator, FSRIteratorReferences}, list::FSRList
        },
        vm::thread::FSRThreadRuntime,
    },
    utils::error::FSRError,
};


pub struct FSRHashMap {
    inner_map: AHashMap<u64, Vec<(AtomicObjId, AtomicObjId)>>,
}

impl Debug for FSRHashMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FSRHashMap")
            .field("inner_map", &"{...}")
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

impl GetReference for FSRHashMap {
    fn get_reference<'a>(&'a self) -> Box<dyn Iterator<Item = &'a AtomicObjId> + 'a> {
        let mut v = vec![];
        for (_, vec) in self.inner_map.iter() {
            for (key, value) in vec.iter() {
                v.push(key);
                v.push(value);
            }
        }
        Box::new(v.into_iter())
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
    fn next(&mut self, thread: &mut FSRThreadRuntime) -> Option<Result<ObjId, FSRError>> {
        let c = self.iter.next();
        c.map(|x| {
            let vs = vec![x.0, x.1];
            let list = FSRList::new_value(vs);
            let ret = thread
                .garbage_collect
                .new_object(list, FSRGlobalObjId::ListCls as ObjId);
            Ok(ret)
        })
    }
}

pub fn fsr_fn_hashmap_iter<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId,
) -> Result<FSRRetValue<'a>, FSRError> {
    let hashmap = FSRObject::id_to_obj(args[0]);
    if let FSRValue::Any(any) = &hashmap.value {
        if let Some(hashmap) = any.value.as_any().downcast_ref::<FSRHashMap>() {
            let iter = hashmap.inner_map.iter().flat_map(|(k, v)| {
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
                FSRGlobalObjId::InnerIterator as ObjId,
            );
            return Ok(FSRRetValue::GlobalId(object));
        } else {
            unimplemented!()
        }
    } else {
        unimplemented!()
    }
}

pub fn fsr_fn_hashmap_new<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId,
) -> Result<FSRRetValue<'a>, FSRError> {
    let hashmap = FSRHashMap::new();
    let object = thread
        .garbage_collect
        .new_object(hashmap.to_any_type(), FSRGlobalObjId::HashMapCls as ObjId);
    Ok(FSRRetValue::GlobalId(object))
}

/// Insert a key-value pair into the hashmap
/// accepts 3 arguments
/// 1. hashmap object
/// 2. key
/// 3. value
pub fn fsr_fn_hashmap_insert<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId,
) -> Result<FSRRetValue<'a>, FSRError> {
    if args.len() != 3 {
        return Err(FSRError::new("not valid args", crate::utils::error::FSRErrCode::RuntimeError));
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

pub fn fsr_fn_hashmap_get<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId,
) -> Result<FSRRetValue<'a>, FSRError> {
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

pub fn fsr_fn_hashmap_contains<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId,
) -> Result<FSRRetValue<'a>, FSRError> {
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

pub fn fsr_fn_hashmap_remove<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId,
) -> Result<FSRRetValue<'a>, FSRError> {
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
    pub fn new() -> Self {
        Self {
            inner_map: AHashMap::new(),
        }
    }

    pub fn to_any_type(self) -> FSRValue<'static> {
        FSRValue::Any(Box::new(AnyType {
            value: Box::new(self),
        }))
    }

    pub fn insert(
        &mut self,
        key: ObjId,
        value: ObjId,
        thread: &mut FSRThreadRuntime,
    ) -> Result<(), FSRError> {
        let key_obj = FSRObject::id_to_obj(key);
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

        if let None = self.inner_map.get(&hash) {
            self.inner_map
                .insert(hash, vec![(AtomicObjId::new(key), AtomicObjId::new(value))]);
            return Ok(());
        }
        let res = {
            let res = self.inner_map.get_mut(&hash).unwrap();
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
            unimplemented!()
        };

        if let None = self.inner_map.get(&hash) {
            return None;
        }

        let res = self.inner_map.get(&hash).unwrap();
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
            unimplemented!()
        };

        if let None = self.inner_map.get(&hash) {
            return ;
        }

        let res = self.inner_map.get(&hash).unwrap();
        let len = res.len();
        if len == 1 {
            self.inner_map.remove(&hash);
            return ;
        }
        let res = self.inner_map.get_mut(&hash).unwrap();
        for i in 0..len {
            let save_item = &res[i];
            let save_key = save_item.0.load(std::sync::atomic::Ordering::Relaxed);
            if save_key == key {
                res.remove(i);
                return ;
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
                return ;
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

        cls
    }
}
