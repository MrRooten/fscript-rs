use std::{
    fmt::{Debug, Formatter},
    sync::{atomic::Ordering, Arc},
};

use ahash::AHashMap;

use crate::{
    backend::{
        compiler::bytecode::BinaryOffset,
        memory::GarbageCollector,
        types::{
            base::{FSRObject, FSRValue},
            iterator::FSRInnerIterator,
        },
        vm::{thread::FSRThreadRuntime, virtual_machine::gid},
    }, to_rs_list, utils::error::{FSRErrCode, FSRError}
};

use super::{
    base::{Area, AtomicObjId, GlobalObj, FSRRetValue, ObjId},
    class::FSRClass,
    fn_def::FSRFn,
    iterator::{FSRIterator, FSRIteratorReferences},
    string::FSRInnerString,
};

pub struct FSRListIterator<'a> {
    pub(crate) list_obj: ObjId,
    pub(crate) iter: std::slice::Iter<'a, std::sync::atomic::AtomicUsize>,
}

impl FSRIteratorReferences for FSRListIterator<'_> {
    fn ref_objects(&self) -> Vec<ObjId> {
        vec![self.list_obj]
    }
}

impl FSRIterator for FSRListIterator<'_> {
    fn next(&mut self, thread: &mut FSRThreadRuntime) -> Result<Option<ObjId>, FSRError> {
        // let c = self.iter.next();
        // c.map(|x| {
        //     let obj_id = x.load(Ordering::Relaxed);
        //     Ok(obj_id)
        // })
        if let Some(x) = self.iter.next() {
            let obj_id = x.load(Ordering::Relaxed);
            return Ok(Some(obj_id));
        }
        Ok(None)
    }
}

pub struct FSRList {
    vs: Vec<AtomicObjId>,
}

impl Debug for FSRList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FSRList").field("vs", &"[...]").finish()
    }
}

fn list_len(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    if len != 1 {
        return Err(FSRError::new("List::len must has 1 arguments", FSRErrCode::NotValidArgs));
    }
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);

    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(

    if let FSRValue::List(self_s) = &self_object.value {
        // return Ok(FSRRetValue::Value(
        //     Box::new(FSRInteger::new_inst(self_s.vs.len() as i64)),
        // ));
        return Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
            FSRValue::Integer(self_s.get_items().len() as i64),
            gid(GlobalObj::IntegerCls),
        )));
    }

    unimplemented!()
}

fn list_string(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    let mut s = FSRInnerString::new("");
    s.push('[');
    let obj_id = args[0];
    let obj = FSRObject::id_to_obj(obj_id);
    if let FSRValue::List(l) = &obj.value {
        let size = l.get_items().len();
        for (count, id) in l.get_items().iter().enumerate() {
            let obj = FSRObject::id_to_obj(id.load(Ordering::Relaxed));
            let s_value = obj.to_string(thread);
            if let FSRValue::String(_s) = &s_value {
                s.push_inner_str(_s);
                if count < size - 1 {
                    s.push_str(", ");
                }
            }
        }
    }

    s.push(']');
    let obj_id = thread.garbage_collect.new_object(
        FSRValue::String(Arc::new(s)),
        gid(GlobalObj::StringCls),
    );
    Ok(FSRRetValue::GlobalId(obj_id))
}

fn iter(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    let self_id = args[0];
    if let FSRValue::List(s) = &FSRObject::id_to_obj(self_id).value {
        let iterator = FSRListIterator {
            list_obj: self_id,
            iter: s.vs.iter(),
        };

        let inner_obj = thread.garbage_collect.new_object(
            FSRValue::Iterator(Box::new(FSRInnerIterator {
                obj: self_id,
                iterator: Some(Box::new(iterator)),
            })),
            gid(GlobalObj::InnerIterator),
        );
        return Ok(FSRRetValue::GlobalId(inner_obj));
    }
    unimplemented!()
}

pub fn get_item(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    let self_id = args[0];
    let index_id = args[1];
    let obj = FSRObject::id_to_obj(self_id);
    let index_obj = FSRObject::id_to_obj(index_id);
    if let FSRValue::List(l) = &obj.value {
        if let FSRValue::Integer(i) = &index_obj.value {
            let index = *i as usize;
            if let Some(s) = l.vs.get(index) {
                return Ok(FSRRetValue::GlobalId(s.load(Ordering::Relaxed)));
            } else {
                return Err(FSRError::new("list index of range", FSRErrCode::OutOfRange));
            }
        } else if let FSRValue::Range(range) = &index_obj.value {
            let start = range.range.start as usize;
            let end = range.range.end as usize;
            let sub = l.vs[start..end]
                .iter()
                .map(|x| AtomicObjId::new(x.load(Ordering::Relaxed)))
                .collect::<Vec<_>>();
            let range = thread.garbage_collect.new_object(
                FSRList::new_value_ref(sub),
                gid(GlobalObj::ListCls) as ObjId,
            );
            return Ok(FSRRetValue::GlobalId(range));
        }
    }
    unimplemented!()
}

pub fn set_item(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    if args.len() != 3 {
        return Err(FSRError::new(
            "set_item args error",
            FSRErrCode::RuntimeError,
        ));
    }
    let self_id = args[0];
    let index_id = args[1];
    let target_id = args[2];
    
    let obj = FSRObject::id_to_mut_obj(self_id).unwrap();
    let index_obj = FSRObject::id_to_obj(index_id);
    if obj.area.is_long() && FSRObject::id_to_obj(target_id).area == Area::Minjor {
        obj.set_write_barrier(true);
    }
    if let FSRValue::List(l) = &obj.value {
        if let FSRValue::Integer(i) = &index_obj.value {
            let index = *i as usize;
            if let Some(s) = l.vs.get(index) {
                s.store(target_id, Ordering::Relaxed);
                return Ok(FSRRetValue::GlobalId(FSRObject::none_id()));
            } else {
                return Err(FSRError::new("list index of range", FSRErrCode::OutOfRange));
            }
        }

        unimplemented!()
    }
    unimplemented!()
}

pub fn sort(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    if args.len() != 1 {
        return Err(FSRError::new("sort args error", FSRErrCode::RuntimeError));
    }
    let obj_id = args[0];
    let obj = FSRObject::id_to_mut_obj(obj_id).expect("msg: not a list");
    if let FSRValue::List(l) = &mut obj.value {
        l.vs.sort_by(|a, b| {
            let l_id = a.load(Ordering::Relaxed);
            let r_id = b.load(Ordering::Relaxed);
            let v = FSRThreadRuntime::compare(
                l_id,
                r_id,
                crate::backend::compiler::bytecode::CompareOperator::Greater,
                thread,
            )
            .unwrap();
            if v {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Less
            }
        });
    }
    Ok(FSRRetValue::GlobalId(FSRObject::none_id()))
}

pub fn sort_by(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    if args.len() != 2 {
        return Err(FSRError::new(
            "sort_by args error",
            FSRErrCode::RuntimeError,
        ));
    }
    let obj_id = args[0];
    let obj = FSRObject::id_to_mut_obj(obj_id).expect("msg: not a list");
    let compare_fn_id = args[1];
    let compare_fn = FSRObject::id_to_obj(compare_fn_id);
    if let FSRValue::List(l) = &mut obj.value {
        //let thread_ptr = thread as *mut FSRThreadRuntime;
        l.vs.sort_by(|a, b| {
            let l_id = a.load(Ordering::Relaxed);
            let r_id = b.load(Ordering::Relaxed);
            //let thread = unsafe { &mut *thread_ptr }; // Use raw pointer to avoid borrowing issues

            let ret = compare_fn
                .call(&[l_id, r_id], thread)
                .unwrap();
            if !FSRObject::id_to_obj(ret.get_id()).is_false() {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Less
            }
        });
    }
    Ok(FSRRetValue::GlobalId(FSRObject::none_id()))
}

pub fn reverse(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    let obj_id = args[0];
    let obj = FSRObject::id_to_mut_obj(obj_id).expect("msg: not a list");
    if let FSRValue::List(l) = &mut obj.value {
        l.vs.reverse();
    } else {
        return Err(FSRError::new(
            "reverse args error not a list",
            FSRErrCode::RuntimeError,
        ));
    }
    Ok(FSRRetValue::GlobalId(FSRObject::none_id()))
}

pub fn sort_key(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    if args.len() != 2 {
        return Err(FSRError::new(
            "sort_by args error",
            FSRErrCode::RuntimeError,
        ));
    }
    let obj_id = args[0];
    let obj = FSRObject::id_to_mut_obj(obj_id).expect("msg: not a list");
    let key_fn_id = args[1];
    let key_fn = FSRObject::id_to_obj(key_fn_id);
    if let FSRValue::List(l) = &mut obj.value {
        l.vs.sort_by_cached_key(|a| {
            let l_id = a.load(Ordering::Relaxed);
            //let thread = unsafe { &mut *thread_ptr }; // Use raw pointer to avoid borrowing issues

            let ret = key_fn.call(&[l_id], thread).unwrap();
            let ret_id = ret.get_id();
            let obj = FSRObject::id_to_obj(ret_id);
            if let FSRValue::Integer(i) = &obj.value {
                return *i;
            } else {
                let ord_fn = obj.get_cls_offset_attr(BinaryOffset::Order).unwrap();
                let ord_fn_id = ord_fn.load(Ordering::Relaxed);
                let ord_fn = FSRObject::id_to_obj(ord_fn_id);
                let ord_value = ord_fn.call(&[ret_id], thread).unwrap();
                let ord_value_id = ord_value.get_id();
                if let FSRValue::Integer(i) = &FSRObject::id_to_obj(ord_value_id).value {
                    return *i;
                }
            }
            panic!("only support integer as order")
        });
    }
    Ok(FSRRetValue::GlobalId(FSRObject::none_id()))
}

pub fn push(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    if args.len() != 2 {
        return Err(FSRError::new("push args error", FSRErrCode::RuntimeError));
    }
    let self_id = args[0];
    let obj = FSRObject::id_to_mut_obj(self_id).expect("msg: not a list");
    if obj.area.is_long() && FSRObject::id_to_obj(args[1]).area == Area::Minjor {
        obj.set_write_barrier(true);
    }
    if let FSRValue::List(l) = &mut obj.value {
        l.vs.push(AtomicObjId::new(args[1]));
    }
    Ok(FSRRetValue::GlobalId(FSRObject::none_id()))
}

pub fn map(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    if args.len() != 2 {
        return Err(FSRError::new("map args error", FSRErrCode::RuntimeError));
    }
    let self_id = args[0];
    let map_fn_id = args[1];
    let map_fn = FSRObject::id_to_obj(map_fn_id);
    let obj = FSRObject::id_to_mut_obj(self_id).expect("msg: not a list");

    if let FSRValue::List(l) = &mut obj.value {
        let mut ret_list = Vec::with_capacity(l.vs.len());
        for id in l.get_items() {
            let ret = map_fn.call(&[id.load(Ordering::Relaxed)], thread)?;
            let ret_id = ret.get_id();
            ret_list.push(AtomicObjId::new(ret_id));
        }

        return Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
            FSRList::new_value_ref(ret_list),
            gid(GlobalObj::ListCls) as ObjId,
        )));
    }
    Ok(FSRRetValue::GlobalId(FSRObject::none_id()))
}

pub fn filter(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    if args.len() != 2 {
        return Err(FSRError::new("filter args error", FSRErrCode::RuntimeError));
    }
    let self_id = args[0];
    let filter_fn_id = args[1];
    let filter_fn = FSRObject::id_to_obj(filter_fn_id);
    let obj = FSRObject::id_to_mut_obj(self_id).expect("msg: not a list");

    if let FSRValue::List(l) = &mut obj.value {
        let mut ret_list = Vec::with_capacity(l.vs.len());
        for id in l.get_items() {
            let ret = filter_fn.call(&[id.load(Ordering::Relaxed)], thread)?;
            let ret_id = ret.get_id();
            if ret_id == FSRObject::true_id() {
                ret_list.push(AtomicObjId::new(id.load(Ordering::Relaxed)));
            }
        }

        return Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
            FSRList::new_value_ref(ret_list),
            gid(GlobalObj::ListCls) as ObjId,
        )));
    }
    Ok(FSRRetValue::GlobalId(FSRObject::none_id()))
}

pub fn equal(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    if args.len() != 2 {
        return Err(FSRError::new(
            "list equal args error",
            FSRErrCode::RuntimeError,
        ));
    }
    let self_id = args[0];
    let other_id = args[1];
    let self_object = FSRObject::id_to_obj(self_id);
    let other_object = FSRObject::id_to_obj(other_id);

    if let FSRValue::List(self_s) = &self_object.value {
        if let FSRValue::List(other_s) = &other_object.value {
            if self_s.get_items().len() != other_s.get_items().len() {
                return Ok(FSRRetValue::GlobalId(FSRObject::false_id()));
            }
            for (i, id) in self_s.get_items().iter().enumerate() {
                let obj_id = id.load(Ordering::Relaxed);
                let obj = FSRObject::id_to_obj(obj_id);
                let eq_fn_id = obj
                    .get_cls_offset_attr(BinaryOffset::Equal)
                    .unwrap()
                    .load(Ordering::Relaxed);
                let eq_fn = FSRObject::id_to_obj(eq_fn_id);
                let equal_res = eq_fn
                    .call(
                        &[obj_id, other_s.vs[i].load(Ordering::Relaxed)],
                        thread,
                        
                    )?
                    .get_id();
                if equal_res != FSRObject::true_id() {
                    return Ok(FSRRetValue::GlobalId(FSRObject::false_id()));
                }
            }
            return Ok(FSRRetValue::GlobalId(FSRObject::true_id()));
        }
    }
    Ok(FSRRetValue::GlobalId(FSRObject::false_id()))
}

impl FSRList {
    pub fn get_class() -> FSRClass {
        let mut cls = FSRClass::new("List");
        let len_m = FSRFn::from_rust_fn_static(list_len, "list_len");
        cls.insert_attr("len", len_m);
        let to_string = FSRFn::from_rust_fn_static(list_string, "list_string");
        cls.insert_attr("__str__", to_string);
        let get_iter = FSRFn::from_rust_fn_static(iter, "list_iter");
        cls.insert_attr("__iter__", get_iter);
        let sort_fn = FSRFn::from_rust_fn_static(sort, "list_iter");
        cls.insert_attr("sort", sort_fn);
        let get_item = FSRFn::from_rust_fn_static(get_item, "list_get_item");
        cls.insert_offset_attr(BinaryOffset::GetItem, get_item);
        let sort_by_fn = FSRFn::from_rust_fn_static(sort_by, "list_sort_by");
        cls.insert_attr("sort_by", sort_by_fn);
        let push_fn = FSRFn::from_rust_fn_static(push, "list_push");
        cls.insert_attr("push", push_fn);
        let sort_key_fn = FSRFn::from_rust_fn_static(sort_key, "list_sort_key");
        cls.insert_attr("sort_key", sort_key_fn);
        let reverse_fn = FSRFn::from_rust_fn_static(reverse, "list_reverse");
        cls.insert_attr("reverse", reverse_fn);
        let map_fn = FSRFn::from_rust_fn_static(map, "list_map");
        cls.insert_attr("map", map_fn);
        let equal_fn = FSRFn::from_rust_fn_static(equal, "list_equal");
        cls.insert_offset_attr(BinaryOffset::Equal, equal_fn);
        let filter_fn = FSRFn::from_rust_fn_static(filter, "list_filter");
        cls.insert_attr("filter", filter_fn);
        let set_item = FSRFn::from_rust_fn_static(set_item, "list_set_item");
        cls.insert_offset_attr(BinaryOffset::SetItem, set_item);
        cls
    }

    pub fn as_string(&self) -> String {
        unimplemented!()
    }

    pub fn get_items(&self) -> Vec<&AtomicObjId> {
        self.vs.iter().collect()
    }

    pub fn new_value(vs: Vec<ObjId>) -> FSRValue<'static> {
        let vs = vs.into_iter().map(AtomicObjId::new).collect::<Vec<_>>();
        FSRValue::List(Box::new(Self { vs }))
    }

    pub fn new_value_ref(vs: Vec<AtomicObjId>) -> FSRValue<'static> {
        FSRValue::List(Box::new(Self { vs }))
    }

    pub fn iter_values(&self) -> impl Iterator<Item = &AtomicObjId> {
        self.vs.iter()
    }
}
