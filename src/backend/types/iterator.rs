use crate::{
    backend::vm::thread::FSRThreadRuntime,
    utils::error::{FSRErrCode, FSRError},
};

use super::{
    base::{FSRGlobalObjId, FSRObject, FSRRetValue, FSRValue, ObjId},
    class::FSRClass,
    fn_def::FSRFn, module::FSRModule,
};

#[derive(Debug, Clone)]
pub struct FSRInnerIterator {
    pub(crate) obj: ObjId,
    pub(crate) index: usize
}

fn next_obj<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: Option<ObjId>
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_obj = args[0];
    let self_obj = FSRObject::id_to_mut_obj(self_obj);
    let mut result = None;
    if let FSRValue::Iterator(it) = &mut self_obj.value {
        let from_obj = FSRObject::id_to_obj(it.obj);
        if let FSRValue::ClassInst(inst) = &from_obj.value {
            let vm = thread.get_vm();
            let cls = match vm.get_global_obj_by_name(inst.get_cls_name()) {
                Some(s) => s,
                None => {
                    return Err(FSRError::new("Not such a cls", FSRErrCode::NoSuchObject));
                }
            };
            let cls = FSRObject::id_to_obj(*cls);
            let cls = cls.as_class();
            let v = cls.get_attr("__index__");
            if let Some(obj_id) = v {
                let obj = FSRObject::id_to_obj(obj_id);
                let ret = obj.call(&[it.obj], thread, module);
                result = Some(ret?);
            }
        } else if let FSRValue::List(l) = &from_obj.value {
            let vs = l.get_items();
            result = Some(match vs.get(it.index) {
                Some(s) => FSRRetValue::GlobalId(*s),
                None => FSRRetValue::GlobalId(0),
            });
        }
        it.index += 1;
    }

    Ok(match result {
        Some(s) => s,
        None => FSRRetValue::GlobalId(0),
    })
}

impl FSRInnerIterator {
    pub fn get_class<'a>() -> FSRClass<'a> {
        let mut cls = FSRClass::new("InnerIterator");
        let next = FSRFn::from_rust_fn(next_obj);
        cls.insert_attr("__next__", next);
        cls
    }

    pub fn new_inst<'a>(iterator: FSRInnerIterator) -> FSRObject<'a> {
        let mut object = FSRObject::new();
        object.set_cls(FSRGlobalObjId::InnerIterator as ObjId);
        object.set_value(FSRValue::Iterator(iterator));
        object
    }
}
