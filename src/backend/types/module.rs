use std::{fs, path::Path};

use crate::{backend::compiler::bytecode::Bytecode, utils::error::FSRError};

pub struct FSRModule {
    #[allow(unused)]
    bytecode: Bytecode
}

impl FSRModule {
    pub fn from_file<P>(file: P) -> Result<Self, FSRError>
    where P: AsRef<Path> {
        let _ = fs::File::open(file);
        unimplemented!()
    }

    pub fn from_code(name: &str, code: &str) -> Result<Self, FSRError> {
        let bytecode = Bytecode::compile(name, code);
        Ok(Self {
            bytecode,
        })
    }
}