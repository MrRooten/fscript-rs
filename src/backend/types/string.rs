use std::{
    fmt,
    hash::{Hash, Hasher},
    str::{Chars, Split},
    sync::Arc,
};

use ahash::AHasher;

use crate::{
    backend::{
        compiler::bytecode::BinaryOffset,
        memory::GarbageCollector,
        types::{base::FSRValue, bytes::FSRInnerBytes},
        vm::{thread::FSRThreadRuntime, virtual_machine::gid},
    }, to_rs_list, utils::error::FSRError
};

use super::{
    base::{FSRObject, FSRRetValue, GlobalObj, ObjId},
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

    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
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
    fn next(&mut self, thread: &mut FSRThreadRuntime) -> Result<Option<ObjId>, FSRError> {
        // let c = self.iter.next();
        // c.map(|x| {
        //     let obj_id = thread.garbage_collect.new_object(
        //         FSRValue::String(Arc::new(FSRInnerString::new_from_char(x))),
        //         get_object_by_global_id(FSRGlobalObjId::StringCls),
        //     );
        //     Ok(obj_id)
        // })
        if let Some(c) = self.iter.next() {
            let obj_id = thread.garbage_collect.new_object(
                FSRValue::String(Arc::new(FSRInnerString::new_from_char(c))),
                gid(GlobalObj::StringCls),
            );
            Ok(Some(obj_id))
        } else {
            Ok(None)
        }
    }
}

pub struct FSRSplitStringIterator<'a> {
    pub(crate) str_obj: ObjId,
    pub(crate) split_str: ObjId,
    pub(crate) iter: Split<'a, &'a str>,
}

impl FSRIteratorReferences for FSRSplitStringIterator<'_> {
    fn ref_objects(&self) -> Vec<ObjId> {
        vec![self.str_obj, self.split_str]
    }
}

impl FSRIterator for FSRSplitStringIterator<'_> {
    fn next(&mut self, thread: &mut FSRThreadRuntime) -> Result<Option<ObjId>, FSRError> {
        if let Some(c) = self.iter.next() {
            let s = FSRInnerString::new(c);
            let obj_id = thread.garbage_collect.new_object(
                FSRValue::String(Arc::new(s)),
                gid(GlobalObj::StringCls),
            );
            Ok(Some(obj_id))
        } else {
            Ok(None)
        }
    }
}

pub struct FSRString {}

fn string_iter(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
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
            gid(GlobalObj::InnerIterator),
        );
        return Ok(FSRRetValue::GlobalId(inner_obj));
    }
    unimplemented!()
}

fn string_len(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);

    if let FSRValue::String(self_s) = &self_object.value {
        return Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
            FSRValue::Integer(self_s.len() as i64),
            gid(GlobalObj::IntegerCls),
        )));
    }

    unimplemented!()
}

fn string_as_bytes(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);

    if let FSRValue::String(self_s) = &self_object.value {
        return Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
            FSRValue::Bytes(Box::new(FSRInnerBytes::new(
                self_s.chars.as_bytes().to_vec(),
            ))),
            gid(GlobalObj::BytesCls),
        )));
    }

    unimplemented!()
}

pub fn add(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(

    if let FSRValue::String(self_str) = &self_object.value {
        if let FSRValue::String(other_str) = &other_object.value {
            let s = FSRInnerString::new(format!("{}{}", self_str.chars, other_str.chars));
            let obj_id = thread.garbage_collect.new_object(
                FSRValue::String(Arc::new(s)),
                gid(GlobalObj::StringCls),
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

pub fn equal(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
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
        }
    }

    Ok(FSRRetValue::GlobalId(FSRObject::false_id()))
}

fn neq(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);
    let other_object = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(

    if let FSRValue::String(self_str) = &self_object.value {
        if let FSRValue::String(other_str) = &other_object.value {
            if self_str.eq(other_str) {
                return Ok(FSRRetValue::GlobalId(FSRObject::false_id()));
            } else {
                return Ok(FSRRetValue::GlobalId(FSRObject::true_id()));
            }
        }
    }

    Ok(FSRRetValue::GlobalId(FSRObject::true_id()))
}

fn get_sub_char(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
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
                    gid(GlobalObj::StringCls),
                );
                Ok(FSRRetValue::GlobalId(obj_id))
            } else {
                Err(FSRError::new(
                    "index out of range of string",
                    crate::utils::error::FSRErrCode::IndexOutOfRange,
                ))
            }
        } else if let FSRValue::Range(r) = &index.value {
            let start = r.range.start as usize;
            let end = r.range.end as usize;

            if start >= self_str.len() || end > self_str.len() || start > end {
                return Err(FSRError::new(
                    "range out of bounds for string",
                    crate::utils::error::FSRErrCode::IndexOutOfRange,
                ));
            }

            let sub_str = self_str
                .chars
                .chars()
                .skip(start)
                .take(end - start)
                .collect::<String>();
            let obj_id = thread.garbage_collect.new_object(
                FSRValue::String(Arc::new(FSRInnerString::new(sub_str))),
                gid(GlobalObj::StringCls),
            );

            return Ok(FSRRetValue::GlobalId(obj_id));
        } else {
            Err(FSRError::new(
                "index is not an integer of string",
                crate::utils::error::FSRErrCode::NotValidArgs,
            ))
        }
    } else {
        Err(FSRError::new(
            "left value is not a string",
            crate::utils::error::FSRErrCode::NotValidArgs,
        ))
    }
}

fn hash_string(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(

    if let FSRValue::String(self_str) = &self_object.value {
        let mut hasher = AHasher::default();
        self_str.chars.hash(&mut hasher);
        let hash = hasher.finish();
        return Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
            FSRValue::Integer(hash as i64),
            gid(GlobalObj::IntegerCls),
        )));
    }

    unimplemented!()
}

fn split(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);
    let sep_object = FSRObject::id_to_obj(args[1]);

    if let FSRValue::String(self_str) = &self_object.value {
        if let FSRValue::String(sep_str) = &sep_object.value {
            let v = self_str.chars.split(sep_str.as_str());
            let iter = FSRSplitStringIterator {
                str_obj: args[0],
                split_str: args[1],
                iter: v,
            };
            let inner_obj = thread.garbage_collect.new_object(
                FSRValue::Iterator(Box::new(FSRInnerIterator {
                    obj: args[0],
                    iterator: Some(Box::new(iter)),
                })),
                gid(GlobalObj::InnerIterator),
            );
            return Ok(FSRRetValue::GlobalId(inner_obj));
        }
    }

    unimplemented!()
}

fn find(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);
    let sub_object = FSRObject::id_to_obj(args[1]);

    if let FSRValue::String(self_str) = &self_object.value {
        if let FSRValue::String(sub_str) = &sub_object.value {
            if let Some(index) = self_str.chars.find(sub_str.as_str()) {
                return Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
                    FSRValue::Integer(index as i64),
                    gid(GlobalObj::IntegerCls),
                )));
            }
        }
    }

    Ok(FSRRetValue::GlobalId(FSRObject::false_id()))
}

fn rfind(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);
    let sub_object = FSRObject::id_to_obj(args[1]);

    if let FSRValue::String(self_str) = &self_object.value {
        if let FSRValue::String(sub_str) = &sub_object.value {
            if let Some(index) = self_str.chars.rfind(sub_str.as_str()) {
                return Ok(FSRRetValue::GlobalId(thread.garbage_collect.new_object(
                    FSRValue::Integer(index as i64),
                    gid(GlobalObj::IntegerCls),
                )));
            }
        }
    }

    Ok(FSRRetValue::GlobalId(FSRObject::false_id()))
}

fn trim(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);
    if let FSRValue::String(self_str) = &self_object.value {
        let trimmed = self_str.chars.trim();
        let obj_id = thread.garbage_collect.new_object(
            FSRValue::String(Arc::new(FSRInnerString::new(trimmed))),
            gid(GlobalObj::StringCls),
        );
        return Ok(FSRRetValue::GlobalId(obj_id));
    }
    unimplemented!()
}

fn uppercase(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);
    if let FSRValue::String(self_str) = &self_object.value {
        let uppercased = self_str.chars.to_uppercase();
        let obj_id = thread.garbage_collect.new_object(
            FSRValue::String(Arc::new(FSRInnerString::new(uppercased))),
            gid(GlobalObj::StringCls),
        );
        return Ok(FSRRetValue::GlobalId(obj_id));
    }
    unimplemented!()
}

fn lowercase(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);
    if let FSRValue::String(self_str) = &self_object.value {
        let lowercased = self_str.chars.to_lowercase();
        let obj_id = thread.garbage_collect.new_object(
            FSRValue::String(Arc::new(FSRInnerString::new(lowercased))),
            gid(GlobalObj::StringCls),
        );
        return Ok(FSRRetValue::GlobalId(obj_id));
    }
    unimplemented!()
}

fn starts_with(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);
    let prefix_object = FSRObject::id_to_obj(args[1]);

    if let FSRValue::String(self_str) = &self_object.value {
        if let FSRValue::String(prefix_str) = &prefix_object.value {
            return Ok(FSRRetValue::GlobalId(if self_str.chars.starts_with(prefix_str.as_str()) {
                FSRObject::true_id()
            } else {
                FSRObject::false_id()
            }));
        }
    }

    Err(FSRError::new(
        "Invalid arguments for starts_with",
        crate::utils::error::FSRErrCode::NotValidArgs,
    ))
}


pub fn fsr_fn_strip(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);
    if let FSRValue::String(self_str) = &self_object.value {
        let stripped = self_str.chars.trim();
        let obj_id = thread.garbage_collect.new_object(
            FSRValue::String(Arc::new(FSRInnerString::new(stripped))),
            gid(GlobalObj::StringCls),
        );
        return Ok(FSRRetValue::GlobalId(obj_id));
    }
    Err(FSRError::new(
        "Invalid argument for strip",
        crate::utils::error::FSRErrCode::NotValidArgs,
    ))
}

/// format string like, support format("{} {}", "hello", "world")
pub fn fsr_fn_format_string(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    if len < 1 {
        return Err(FSRError::new(
            "format_string requires at least 2 arguments",
            crate::utils::error::FSRErrCode::NotValidArgs,
        ));
    }

    let args = to_rs_list!(args, len);
    let format_str_id = args[0];
    let format_args = &args[1..len];

    let format_str = if let FSRValue::String(s) = &FSRObject::id_to_obj(format_str_id).value {
        s.as_str()
    } else {
        return Err(FSRError::new(
            "First argument must be a string",
            crate::utils::error::FSRErrCode::NotValidArgs,
        ));
    };

    let mut arg_strings = Vec::new();
    for &arg_id in format_args {
        let obj = FSRObject::id_to_obj(arg_id);
        let res = obj.to_string(thread);
        if let FSRValue::String(s) = &res {
            arg_strings.push(s.as_str().to_string());
        }
    }

    let mut result = String::new();
    let mut arg_iter = arg_strings.iter();
    let mut chars = format_str.chars().peekable();
    // Process the format string
    while let Some(c) = chars.next() {
        if c == '{' {
            if chars.peek() == Some(&'{') {
                chars.next(); // consume second '{'
                result.push('{');
            } else if chars.peek() == Some(&'}') {
                chars.next(); // consume '}'
                if let Some(val) = arg_iter.next() {
                    result.push_str(val);
                } else {
                    result.push_str("{}");
                }
            } else {
                result.push('{');
            }
        } else if c == '}' {
            if chars.peek() == Some(&'}') {
                chars.next(); // consume second '}'
                result.push('}');
            } else {
                result.push('}');
            }
        } else {
            result.push(c);
        }
    }

    let value = FSRString::new_value(result);
    let res = thread.garbage_collect.new_object(
        value,
        crate::backend::types::base::GlobalObj::StringCls.get_id(),
    );
    Ok(FSRRetValue::GlobalId(res))
}

impl FSRString {
    pub fn get_class() -> FSRClass {
        let mut cls = FSRClass::new("String");
        let len_m = FSRFn::from_rust_fn_static(string_len, "string_len");
        cls.insert_attr("len", len_m);
        let add_fn = FSRFn::from_rust_fn_static(add, "string_add");
        //cls.insert_attr("__add__", add_fn);
        cls.insert_offset_attr(BinaryOffset::Add, add_fn);

        let eq_fn = FSRFn::from_rust_fn_static(equal, "string_eq");
        //cls.insert_attr("__add__", add_fn);
        cls.insert_offset_attr(BinaryOffset::Equal, eq_fn);

        let neq_fn = FSRFn::from_rust_fn_static(neq, "string_neq");
        cls.insert_offset_attr(BinaryOffset::NotEqual, neq_fn);

        cls.insert_offset_attr(
            BinaryOffset::GetItem,
            FSRFn::from_rust_fn_static(get_sub_char, "string_get_sub_char"),
        );

        let iter = FSRFn::from_rust_fn_static(string_iter, "string_iter");
        cls.insert_attr("__iter__", iter);

        let hash = FSRFn::from_rust_fn_static(hash_string, "string_hash");
        cls.insert_offset_attr(BinaryOffset::Hash, hash);

        let split = FSRFn::from_rust_fn_static(split, "string_split");
        cls.insert_attr("split", split);

        let trim = FSRFn::from_rust_fn_static(trim, "string_trim");
        cls.insert_attr("trim", trim);

        let find = FSRFn::from_rust_fn_static(find, "string_find");
        cls.insert_attr("find", find);
        let rfind = FSRFn::from_rust_fn_static(rfind, "string_rfind");
        cls.insert_attr("rfind", rfind);
        let as_bytes = FSRFn::from_rust_fn_static(string_as_bytes, "string_as_bytes");
        cls.insert_attr("as_bytes", as_bytes);
        let uppercase = FSRFn::from_rust_fn_static(uppercase, "string_uppercase");
        cls.insert_attr("uppercase", uppercase);
        let lowercase = FSRFn::from_rust_fn_static(lowercase, "string_lowercase");
        cls.insert_attr("lowercase", lowercase);
        let format_string = FSRFn::from_rust_fn_static(fsr_fn_format_string, "format_string");
        cls.insert_attr("format", format_string);
        let starts_with = FSRFn::from_rust_fn_static(starts_with, "string_starts_with");
        cls.insert_attr("starts_with", starts_with);
        let strip = FSRFn::from_rust_fn_static(fsr_fn_strip, "string_strip");
        cls.insert_attr("strip", strip);
        cls
    }

    pub fn new_value(s: impl Into<String>) -> FSRValue<'static> {
        FSRValue::String(Arc::new(FSRInnerString::new(s)))
    }

    pub fn new_inst_with_inner(s: Arc<FSRInnerString>) -> FSRValue<'static> {
        FSRValue::String(s)
    }
}
