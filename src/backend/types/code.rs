use std::{
    borrow::Cow, cell::RefCell, collections::HashMap, fs, path::Path, ptr::addr_of, sync::{atomic::AtomicUsize, Mutex}
};

use ahash::AHashMap;

use crate::{
    backend::compiler::bytecode::{Bytecode, BytecodeArg},
    utils::error::FSRError,
};

use std::fmt::Debug;

use super::{
    base::{AtomicObjId, FSRGlobalObjId, FSRObject, FSRValue, ObjId},
    class::FSRClass,
};


pub struct FSRCode<'a> {
    name: Cow<'a, str>,
    #[allow(unused)]
    bytecode: Bytecode,
    object_map: AHashMap<String, AtomicObjId>,
    const_table: Vec<Option<ObjId>>,
}

impl Debug for FSRCode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = addr_of!(self.bytecode);
        let s = format!("<Code `{}`>", v as usize);
        f.debug_struct("FSRCode")
            .field("name", &self.name)
            .field("bytecode", &self.bytecode)
            .finish()
    }
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
                object_map: AHashMap::new(),
                const_table: vec![],
            };

            let mut object = FSRObject::new();
            // object.delete_flag.store(false, std::sync::atomic::Ordering::Relaxed);
            let tmp = code.name.to_string();
            object.value = FSRValue::Code(Box::new(code));
            object.cls = FSRGlobalObjId::CodeCls as ObjId;
            res.insert(tmp.to_string(), object);
        }
        Ok(res)
    }

    pub fn insert_const(&mut self, const_index: usize, obj: ObjId) {
        if const_index >= self.const_table.len() {
            self.const_table.resize(const_index + 1, None);
        }
        self.const_table[const_index] = Some(obj);
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

    pub fn register_object(&mut self, name: &'a str, obj_id: ObjId) {
        self.object_map
            .insert(name.to_string(), AtomicUsize::new(obj_id));
    }

    pub fn get_object(&self, name: &str) -> Option<&AtomicObjId> {
        self.object_map.get(name)
    }
}
