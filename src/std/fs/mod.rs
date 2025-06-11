use crate::backend::types::{base::FSRObject, module::FSRModule};

pub struct FSRFileSystem {

}

impl FSRFileSystem {
    pub fn new_module() -> FSRModule<'static> {
        let module = FSRModule::new_module("fs");
        module
    }
}