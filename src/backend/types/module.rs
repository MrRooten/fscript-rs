use std::{borrow::Cow, cell::RefCell, collections::HashMap, fs, path::Path, sync::Mutex};

use crate::{backend::compiler::bytecode::{Bytecode, BytecodeArg}, utils::error::FSRError};

use super::{base::{FSRGlobalObjId, FSRObject, FSRValue, ObjId}, class::FSRClass};

#[derive(Debug)]
pub struct FSRModule<'a> {
    name: Cow<'a, str>,
    #[allow(unused)]
    bytecode: Bytecode,
    object_map  : Mutex<HashMap<String, ObjId>>
}

impl Clone for FSRModule<'_> {
    fn clone(&self) -> Self {
        unimplemented!()
    }
    
    fn clone_from(&mut self, _source: &Self) {
        unimplemented!()
    }
}

impl<'a> FSRModule<'a> {
    pub fn get_class() -> FSRClass<'static> {
        FSRClass::new("FSRModule")
    }

    pub fn from_file<P>(file: P) -> Result<Self, FSRError>
    where P: AsRef<Path> {
        let _ = fs::File::open(file);
        unimplemented!()
    }

    pub fn from_code(name: &str, code: &str) -> Result<FSRObject<'a>, FSRError> {
        let bytecode = Bytecode::compile(name, code);
        let module = Self {
            name: Cow::Owned(name.to_string()),
            bytecode,
            object_map: Mutex::new(HashMap::new()),
        };
        let mut object = FSRObject::new();
        object.delete_flag.store(false, std::sync::atomic::Ordering::Relaxed);
        object.value = FSRValue::Module(Box::new(module));
        object.cls = FSRGlobalObjId::ModuleCls as ObjId;

        Ok(object)
    }

    #[inline(always)]
    pub fn get_expr(&self, ip: &(usize, usize)) -> Option<&Vec<BytecodeArg>> {
        self.bytecode.get(ip)
    }

    pub fn get_bytecode(&self) -> &Bytecode {
        &self.bytecode
    }

    pub fn as_string(&self) -> String {
        format!("<Module `{}`>", self.name)
    }

    pub fn register_object(&self, name: &'a str, obj_id: ObjId) {
        self.object_map.lock().unwrap().insert(name.to_string(), obj_id);
    }

    pub fn get_object(&self, name: &str) -> Option<ObjId> {
        self.object_map.lock().unwrap().get(name).copied()
    }
}