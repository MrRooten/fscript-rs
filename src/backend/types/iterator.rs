use crate::{
    backend::{compiler::bytecode::BinaryOffset, memory::GarbageCollector, vm::thread::FSRThreadRuntime},
    utils::error::{FSRErrCode, FSRError},
};

use super::{
    base::{FSRGlobalObjId, FSRObject, FSRRetValue, FSRValue, ObjId},
    class::FSRClass,
    fn_def::FSRFn,
    code::FSRCode,
};

#[derive(Debug, Clone)]
pub struct FSRInnerIterator {
    pub(crate) obj: ObjId,
    pub(crate) index: usize,
}

fn next_obj<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId,
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_obj = FSRObject::id_to_mut_obj(args[0]);
    let mut result = None;
    if let FSRValue::Iterator(it) = &mut self_obj.value {
        let from_obj = FSRObject::id_to_obj(it.obj);
        if let FSRValue::List(l) = &from_obj.value {
            let vs = l.get_items();
            result = Some(match vs.get(it.index) {
                Some(s) => FSRRetValue::GlobalId(*s),
                None => FSRRetValue::GlobalId(0),
            });
        } if let FSRValue::Range(r) = &from_obj.value {
            if it.index as i64 + r.range.start >= r.range.end {
                return Ok(FSRRetValue::GlobalId(0));
            }

            // let obj = thread.garbage_collect.new_object(
            //     FSRValue::Integer((it.index as i64 + r.range.start) as i64),
            //     FSRGlobalObjId::IntegerCls as ObjId,
            // );

            let obj_id = thread.garbage_collect.new_object_with_ptr();
            let obj = FSRObject::id_to_mut_obj(obj_id);
            obj.value = FSRValue::Integer((it.index as i64 + r.range.start) as i64);
            obj.set_cls(FSRGlobalObjId::IntegerCls as ObjId);

            it.index += 1;

            return Ok(FSRRetValue::GlobalId(obj_id))
        } else if let FSRValue::ClassInst(inst) = &from_obj.value {
            let cls = from_obj.cls;
            let cls = FSRObject::id_to_obj(cls);
            let cls = cls.as_class();
            let v = cls.get_offset_attr(BinaryOffset::Index);
            if let Some(obj_id) = v {
                let obj = FSRObject::id_to_obj(obj_id);
                let ret = obj.call(&[it.obj], thread, module, obj_id);
                result = Some(ret?);
            }
        }
        it.index += 1;
    } else {
        panic!("not a iterator");
    }

    

    Ok(match result {
        Some(s) => s,
        None => FSRRetValue::GlobalId(0),
    })
}

impl FSRInnerIterator {
    pub fn get_class<'a>() -> FSRClass<'a> {
        let mut cls = FSRClass::new("InnerIterator");
        let next = FSRFn::from_rust_fn_static(next_obj, "inner_iterator_next");

        // cls.insert_attr("__next__", next);
        cls.insert_offset_attr(BinaryOffset::NextObject, next);
        cls
    }

    pub fn new_inst<'a>(iterator: FSRInnerIterator) -> FSRObject<'a> {
        let mut object = FSRObject::new();
        object.set_cls(FSRGlobalObjId::InnerIterator as ObjId);
        object.set_value(FSRValue::Iterator(Box::new(iterator)));
        object
    }
}
