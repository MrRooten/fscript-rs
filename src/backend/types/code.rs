use std::{borrow::Cow, cell::RefCell, collections::HashMap, fs, path::Path, sync::Mutex};

use ahash::AHashMap;

use crate::{
    backend::compiler::bytecode::{Bytecode, BytecodeArg},
    utils::error::FSRError,
};

use super::{
    base::{FSRGlobalObjId, FSRObject, FSRValue, ObjId},
    class::FSRClass,
};

#[derive(Debug)]
pub struct FSRCode<'a> {
    name: Cow<'a, str>,
    #[allow(unused)]
    bytecode: Bytecode,
    object_map: Mutex<AHashMap<String, ObjId>>,
}

impl<'a> FSRCode<'a> {
    pub fn get_class() -> FSRClass<'static> {
        FSRClass::new("FSRCode")
    }

    pub fn from_file<P>(file: P) -> Result<Self, FSRError>
    where
        P: AsRef<Path>,
    {
        let _ = fs::File::open(file);
        unimplemented!()
    }

    pub fn from_code(name: &str, code: &str) -> Result<HashMap<String, FSRObject<'a>>, FSRError> {
        let bytecode = Bytecode::compile(name, code);
        let mut res = HashMap::new();
        for code in bytecode {
            let code = Self {
                name: Cow::Owned(code.0),
                bytecode: code.1,
                object_map: Mutex::new(AHashMap::new()),
            };

            let mut object = FSRObject::new();
            // object.delete_flag.store(false, std::sync::atomic::Ordering::Relaxed);
            object
                .ref_count
                .store(1, std::sync::atomic::Ordering::Relaxed);
            let tmp = code.name.to_string();
            object.value = FSRValue::Code(Box::new(code));
            object.cls = FSRGlobalObjId::CodeCls as ObjId;
            res.insert(tmp.to_string(), object);
        }
        // let module = Self {
        //     name: Cow::Owned(name.to_string()),
        //     bytecode,
        //     object_map: Mutex::new(AHashMap::new()),
        // };
        Ok(res)
    }

    #[inline(always)]
    pub fn get_expr(&self, ip_1: usize) -> Option<&Vec<BytecodeArg>> {
        self.bytecode.get(ip_1)
    }

    pub fn get_bytecode(&self) -> &Bytecode {
        &self.bytecode
    }

    pub fn as_string(&self) -> String {
        format!("<Module `{}`>", self.name)
    }

    pub fn register_object(&self, name: &'a str, obj_id: ObjId) {
        self.object_map
            .lock()
            .unwrap()
            .insert(name.to_string(), obj_id);
    }

    pub fn get_object(&self, name: &str) -> Option<ObjId> {
        self.object_map.lock().unwrap().get(name).copied()
    }
}
