use std::sync::Arc;

use crate::{
    backend::{
        compiler::bytecode::BinaryOffset,
        types::{
            base::{FSRObject, FSRRetValue, FSRValue, GlobalObj, ObjId},
            class::FSRClass,
            fn_def::FSRFn,
            string::FSRInnerString,
        },
        vm::thread::FSRThreadRuntime,
    },
    to_rs_list,
    utils::error::{FSRErrCode, FSRError},
};

#[derive(Debug, PartialEq, Clone)]
pub struct FSRInnerBytes {
    pub(crate) bytes: Vec<u8>,
}

impl FSRInnerBytes {
    pub fn new(bytes: Vec<u8>) -> Self {
        FSRInnerBytes { bytes }
    }

    pub fn get_bytes(&self) -> &Vec<u8> {
        &self.bytes
    }

    pub fn bs_len(&self) -> usize {
        self.bytes.len()
    }

    pub fn get_class() -> FSRClass {
        let mut cls = FSRClass::new("Bytes");
        cls.insert_offset_attr(
            BinaryOffset::GetItem,
            FSRFn::from_rust_fn_static(get_sub_bytes, "bytes_get_sub_bytes"),
        );
        cls.insert_offset_attr(
            BinaryOffset::SetItem,
            FSRFn::from_rust_fn_static(set_item, "bytes_set_item"),
        );

        let as_hex = FSRFn::from_rust_fn_static(as_hex, "bytes_as_hex");
        cls.insert_attr("as_hex", as_hex);
        let get_len = FSRFn::from_rust_fn_static(get_len, "bytes_get_len");
        cls.insert_attr("len", get_len);
        cls
    }
}

fn get_sub_bytes(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    let self_object = FSRObject::id_to_obj(args[0]);
    let index = FSRObject::id_to_obj(args[1]);
    // let self_object = vm.get_obj_by_id(&self_id).unwrap().borrow();
    // let other_object = vm.get_obj_by_id(&other_id).unwrap().borrow(

    if let FSRValue::Bytes(self_bytes) = &self_object.value {
        if let FSRValue::Integer(index) = &index.value {
            let index = *index as usize;
            if index < self_bytes.bs_len() {
                let obj_id = thread.garbage_collect.new_object(
                    FSRValue::Integer(self_bytes.bytes[index] as i64),
                    GlobalObj::IntegerCls.get_id(),
                );
                Ok(FSRRetValue::GlobalId(obj_id))
            } else {
                Err(FSRError::new(
                    "index out of range of bytes",
                    crate::utils::error::FSRErrCode::IndexOutOfRange,
                ))
            }
        } else if let FSRValue::Range(r) = &index.value {
            let start = r.range.start as usize;
            let end = r.range.end as usize;

            if start >= self_bytes.bs_len() || end > self_bytes.bs_len() || start > end {
                return Err(FSRError::new(
                    "range out of bounds for bytes",
                    crate::utils::error::FSRErrCode::IndexOutOfRange,
                ));
            }

            let sub_bytes = self_bytes.bytes[start..end].to_vec();
            let obj_id = thread.garbage_collect.new_object(
                FSRValue::Bytes(Box::new(FSRInnerBytes::new(sub_bytes))),
                GlobalObj::BytesCls.get_id(),
            );
            Ok(FSRRetValue::GlobalId(obj_id))
        } else {
            Err(FSRError::new(
                "index is not an integer of bytes",
                crate::utils::error::FSRErrCode::NotValidArgs,
            ))
        }
    } else {
        Err(FSRError::new(
            "left value is not a bytes",
            crate::utils::error::FSRErrCode::NotValidArgs,
        ))
    }
}

pub fn get_len(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    if args.len() != 1 {
        return Err(FSRError::new(
            "get_len requires exactly 1 argument",
            FSRErrCode::RuntimeError,
        ));
    }
    let self_id = args[0];
    let obj = FSRObject::id_to_obj(self_id);

    if let FSRValue::Bytes(bytes) = &obj.value {
        let len = bytes.bytes.len() as i64;
        // let obj_id = thread.garbage_collect.new_object(
        //     FSRValue::Integer(len),
        //     GlobalObj::IntegerCls.get_id(),
        // );
        let obj_id = thread.garbage_collect.get_integer(len);
        Ok(FSRRetValue::GlobalId(obj_id))
    } else {
        Err(FSRError::new(
            "left value is not a bytes",
            FSRErrCode::NotValidArgs,
        ))
    }
}

pub fn set_item(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    if args.len() != 3 {
        return Err(FSRError::new(
            "set_item requires exactly 3 arguments",
            FSRErrCode::RuntimeError,
        ));
    }
    let self_id = args[0];
    let index_id = args[1];
    let target_id = args[2];

    let obj = FSRObject::id_to_mut_obj(self_id).unwrap();
    let index_obj = FSRObject::id_to_obj(index_id);
    let target_obj = FSRObject::id_to_obj(target_id);
    let FSRValue::Bytes(l) = &obj.value else {
        return Err(FSRError::new(
            "left value is not a bytes",
            FSRErrCode::NotValidArgs,
        ));
    };
    let FSRValue::Integer(index) = &index_obj.value else {
        return Err(FSRError::new(
            "index is not an integer",
            FSRErrCode::NotValidArgs,
        ));
    };
    let FSRValue::Integer(target) = &target_obj.value else {
        return Err(FSRError::new(
            "target is not an integer",
            FSRErrCode::NotValidArgs,
        ));
    };
    if *target > 255 || *target < 0 {
        return Err(FSRError::new(
            "target value out of range for bytes",
            FSRErrCode::OutOfRange,
        ));
    }
    if l.bytes.is_empty() {
        return Err(FSRError::new("bytes is empty", FSRErrCode::NotValidArgs));
    }
    if *index < 0 {
        return Err(FSRError::new(
            "index cannot be negative",
            FSRErrCode::NotValidArgs,
        ));
    }
    if *index >= l.bytes.len() as i64 {
        return Err(FSRError::new(
            "index out of range of bytes",
            FSRErrCode::OutOfRange,
        ));
    }

    let index = *index as usize;
    let target = *target as u8;
    let mut bytes = l.bytes.clone();
    bytes[index] = target;

    Ok(FSRRetValue::GlobalId(FSRObject::none_id()))
}

pub fn as_hex(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    let args = to_rs_list!(args, len);
    if args.len() != 1 {
        return Err(FSRError::new(
            "as_hex requires exactly 1 argument",
            FSRErrCode::RuntimeError,
        ));
    }
    let self_id = args[0];
    let obj = FSRObject::id_to_obj(self_id);

    if let FSRValue::Bytes(bytes) = &obj.value {
        let hex_string = bytes
            .bytes
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        let hex_obj_id = thread.garbage_collect.new_object(
            FSRValue::String(Arc::new(FSRInnerString::new(hex_string))),
            GlobalObj::StringCls.get_id(),
        );
        Ok(FSRRetValue::GlobalId(hex_obj_id))
    } else {
        Err(FSRError::new(
            "left value is not a bytes",
            FSRErrCode::NotValidArgs,
        ))
    }
}
