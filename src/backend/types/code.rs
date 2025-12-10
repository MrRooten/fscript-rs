use std::{
    borrow::Cow,
    cell::RefCell,
    collections::HashMap,
    fs,
    path::Path,
    ptr::addr_of,
    sync::{atomic::AtomicUsize, Mutex},
};

use ahash::AHashMap;

use crate::{
    backend::{compiler::bytecode::{Bytecode, BytecodeArg}, vm::virtual_machine::gid},
    utils::error::FSRError,
};

use std::fmt::Debug;

use super::{
    base::{AtomicObjId, GlobalObj, FSRObject, FSRValue, ObjId},
    class::FSRClass,
};

pub struct FSRCode {
    name: String,
    bytecode: Bytecode,
    pub(crate) module: ObjId,
}

impl Debug for FSRCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = addr_of!(self.bytecode);
        let s = format!("<Code `{}`>", v as usize);
        f.debug_struct("FSRCode")
            .field("name", &self.name)
            .field("bytecode", &self.bytecode)
            .finish()
    }
}

impl FSRCode {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_class() -> FSRClass {
        FSRClass::new("FSRCode")
    }

    pub fn from_file<P>(file: P) -> Result<Self, FSRError>
    where
        P: AsRef<Path>,
    {
        let _ = fs::File::open(file);
        unimplemented!()
    }

    pub fn from_code<'a>(name: &str, code: &str, module: ObjId) -> Result<HashMap<String, FSRObject<'a>>, FSRError> {
        let bytecode = Bytecode::compile(name, code);
        let mut res = HashMap::new();
        for code in bytecode {
            let code = Self {
                name: code.0.to_string(),
                bytecode: code.1,
                //object_map: AHashMap::new(),
                //const_table: vec![],
                module,

            };

            let mut object = FSRObject::new();
            // object.delete_flag.store(false, std::sync::atomic::Ordering::Relaxed);
            let tmp = code.name.to_string();
            object.value = FSRValue::Code(Box::new(code));
            object.set_cls(gid(GlobalObj::CodeCls));
            res.insert(tmp.to_string(), object);
        }
        Ok(res)
    }

    

    #[cfg_attr(feature = "more_inline", inline(always))]
    pub fn get_expr(&self, first_ip: usize) -> Option<&Vec<BytecodeArg>> {
        self.bytecode.get(first_ip)
    }

    pub fn get_bytecode(&self) -> &Bytecode {
        &self.bytecode
    }

    pub fn as_string(&self) -> String {
        format!("<Module `{}`>", self.name)
    }

}
