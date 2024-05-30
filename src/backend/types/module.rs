use std::{cell::RefCell, collections::HashMap, fs, path::Path};

use crate::{backend::compiler::bytecode::{Bytecode, BytecodeArg}, utils::error::FSRError};

#[derive(Debug)]
pub struct FSRModule<'a> {
    name: &'a str,
    #[allow(unused)]
    bytecode: Bytecode,
    object_map  : RefCell<HashMap<&'a str, u64>>
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
    pub fn from_file<P>(file: P) -> Result<Self, FSRError>
    where P: AsRef<Path> {
        let _ = fs::File::open(file);
        unimplemented!()
    }

    pub fn from_code(name: &'a str, code: &str) -> Result<Self, FSRError> {
        let bytecode = Bytecode::compile(name, code);
        
        Ok(Self {
            name,
            bytecode,
            object_map: RefCell::new(HashMap::new()),
        })
    }

    #[inline(always)]
    pub fn get_bc(&self, ip: &(usize, usize)) -> Option<&Vec<BytecodeArg>> {
        self.bytecode.get(ip)
    }

    pub fn get_bytecode(&self) -> &Bytecode {
        &self.bytecode
    }

    pub fn as_string(&self) -> String {
        format!("<Module `{}`>", self.name)
    }

    pub fn registerl_object(&mut self, name: &'a str, obj_id: u64) {
        self.object_map.borrow_mut().insert(name, obj_id);
    }
}