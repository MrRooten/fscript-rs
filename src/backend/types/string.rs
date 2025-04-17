use std::{any::Any, borrow::Cow, fmt, hash::{DefaultHasher, Hash, Hasher}, str::Chars, sync::Arc};

use crate::{
    backend::{
        compiler::bytecode::BinaryOffset,
        memory::GarbageCollector,
        types::{base::FSRValue, integer::FSRInteger},
        vm::thread::FSRThreadRuntime,
    },
    utils::error::FSRError,
};

use super::{
    base::{FSRGlobalObjId, FSRObject, FSRRetValue, ObjId},
    class::FSRClass,
    fn_def::FSRFn,
    iterator::{FSRInnerIterator, FSRIterator, FSRIteratorReferences},
};

#[derive(Debug, PartialEq, Clone)]
pub struct FSRInnerString {
    chars: String,
}

impl FSRInnerString {
    pub fn new(chars: impl Into<String>) -> Self {
        Self {
            chars: chars.into(),
        }
    }

    pub fn new_from_char(c: char) -> Self {
        Self {
            chars: c.to_string(),
        }
    }

    pub fn len(&self) -> usize {
        self.chars.len()
    }

    pub fn push_inner_str(&mut self, s: &FSRInnerString) {
        self.chars.push_str(&s.chars);
    }

    pub fn push_str(&mut self, s: &str) {
        self.chars.push_str(s);
    }

    pub fn push(&mut self, c: char) {
        self.chars.push(c);
    }

    pub fn as_str(&self) -> &str {
        &self.chars
    }
}

impl fmt::Display for FSRInnerString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.chars)
    }
}

pub struct FSRStringIterator<'a> {
    pub(crate) str_obj: ObjId,
    pub(crate) iter: Chars<'a>,
}

impl FSRIteratorReferences for FSRStringIterator<'_> {
    fn ref_objects(&self) -> Vec<ObjId> {
        vec![self.str_obj]
    }
}

impl FSRIterator for FSRStringIterator<'_> {
    fn next(&mut self, thread: &mut FSRThreadRuntime) -> Option<Result<ObjId, FSRError>> {
        let c = self.iter.next();
        c.map(|x| {
            let obj_id = thread.garbage_collect.new_object(
                FSRValue::String(Arc::new(FSRInnerString::new_from_char(x))),
                FSRGlobalObjId::StringCls as ObjId,
            );
            Ok(obj_id)
        })
    }
}

pub struct FSRString {}

fn string_iter<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId,
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_id = args[0];
    if let FSRValue::String(s) = &FSRObject::id_to_obj(self_id).value {
        let iterator = FSRStringIterator {
            str_obj: self_id,
            iter: s.chars.chars(),
        };
        let inner_obj = thread.garbage_collect.new_object(
            FSRValue::Iterator(Box::new(FSRInnerIterator {
                obj: self_id,
                iterator: Some(Box::new(iterator)),
            })),
            FSRGlobalObjId::InnerIterator as ObjId,
        );
        return Ok(FSRRetValue::GlobalId(inner_obj));
    }
    unimplemented!()
}

fn string_len<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId,
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);

    if let FSRValue::String(self_s) = &self_object.value {
        return Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
            FSRValue::Integer(self_s.len() as i64),
            FSRGlobalObjId::IntegerCls as ObjId,
        )));
    }

    unimplemented!()
}

pub fn add<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId,
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(

    if let FSRValue::String(self_str) = &self_object.value {
        if let FSRValue::String(other_str) = &other_object.value {
            let s = FSRInnerString::new(format!("{}{}", self_str.chars, other_str.chars));
            let obj_id = thread.garbage_collect.new_object(
                FSRValue::String(Arc::new(s)),
                FSRGlobalObjId::StringCls as ObjId,
            );

            return Ok(FSRRetValue::GlobalId(obj_id));
        } else {
            return Err(FSRError::new(
                "right value is not a string",
                crate::utils::error::FSRErrCode::NotValidArgs,
            ));
        }
    } else {
        return Err(FSRError::new(
            "left value is not a string",
            crate::utils::error::FSRErrCode::NotValidArgs,
        ));
    }

    unimplemented!()
}

fn eq<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId,
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(

    if let FSRValue::String(self_str) = &self_object.value {
        if let FSRValue::String(other_str) = &other_object.value {
            if self_str.eq(other_str) {
                return Ok(FSRRetValue::GlobalId(FSRObject::true_id()));
            } else {
                return Ok(FSRRetValue::GlobalId(FSRObject::false_id()));
            }
        } else {
            return Err(FSRError::new(
                "right value is not a string for eq string",
                crate::utils::error::FSRErrCode::NotValidArgs,
            ));
        }
    } else {
        return Err(FSRError::new(
            "left value is not a string for eq string",
            crate::utils::error::FSRErrCode::NotValidArgs,
        ));
    }

    unimplemented!()
}

fn get_sub_char<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId,
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);
    let index = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(

    if let FSRValue::String(self_str) = &self_object.value {
        if let FSRValue::Integer(index) = &index.value {
            let index = *index as usize;
            if index < self_str.len() {
                let obj_id = thread.garbage_collect.new_object(
                    FSRValue::String(Arc::new(FSRInnerString::new_from_char(
                        self_str.chars.chars().nth(index).unwrap(),
                    ))),
                    FSRGlobalObjId::StringCls as ObjId,
                );
                return Ok(FSRRetValue::GlobalId(obj_id));
            } else {
                return Err(FSRError::new(
                    "index out of range of string",
                    crate::utils::error::FSRErrCode::IndexOutOfRange,
                ));
            }
        } else {
            return Err(FSRError::new(
                "index is not an integer of string",
                crate::utils::error::FSRErrCode::NotValidArgs,
            ));
        }
    } else {
        return Err(FSRError::new(
            "left value is not a string",
            crate::utils::error::FSRErrCode::NotValidArgs,
        ));
    }

    unimplemented!()
}

fn hash_string<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId,
) -> Result<FSRRetValue<'a>, FSRError> {
    let self_object = FSRObject::id_to_obj(args[0]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(

    if let FSRValue::String(self_str) = &self_object.value {
        let mut hasher = DefaultHasher::new();
        self_str.chars.hash(&mut hasher);
        let hash = hasher.finish();
        return Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
            FSRValue::Integer(hash as i64),
            FSRGlobalObjId::IntegerCls as ObjId,
        )));
    }

    unimplemented!()
}

impl FSRString {
    pub fn get_class<'a>() -> FSRClass<'a> {
        let mut cls = FSRClass::new("String");
        let len_m = FSRFn::from_rust_fn_static(string_len, "string_len");
        cls.insert_attr("len", len_m);
        let add_fn = FSRFn::from_rust_fn_static(add, "string_add");
        //cls.insert_attr("__add__", add_fn);
        cls.insert_offset_attr(BinaryOffset::Add, add_fn);

        let eq_fn = FSRFn::from_rust_fn_static(eq, "string_eq");
        //cls.insert_attr("__add__", add_fn);
        cls.insert_offset_attr(BinaryOffset::Equal, eq_fn);

        cls.insert_offset_attr(
            BinaryOffset::GetItem,
            FSRFn::from_rust_fn_static(get_sub_char, "string_get_sub_char"),
        );

        let iter = FSRFn::from_rust_fn_static(string_iter, "string_iter");
        cls.insert_attr("__iter__", iter);

        let hash = FSRFn::from_rust_fn_static(hash_string, "string_hash");
        cls.insert_offset_attr(
            BinaryOffset::Hash,
            FSRFn::from_rust_fn_static(hash_string, "string_hash"),
        );
        cls
    }

    pub fn new_value(s: &str) -> FSRValue<'static> {
        FSRValue::String(Arc::new(FSRInnerString::new(s)))
    }

    pub fn new_inst_with_inner(s: Arc<FSRInnerString>) -> FSRValue<'static> {
        FSRValue::String(s)
    }
}
