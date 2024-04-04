use std::collections::HashMap;

use crate::{backend::vm::runtime::FSRThreadRuntime, utils::error::RuntimeBaseError};

use super::base::IFSRObject;

#[derive(Debug)]
pub struct FSRModule<'a> {
    export_object   : HashMap<&'a str, u64>,
    module_name     : String,
}

impl<'a> FSRModule<'a> {
    pub fn new(name: &'a str) -> Self {
        let module = Self {
            export_object: HashMap::new(),
            module_name: name.to_string(),
        };
        return module;
    }

    pub fn get_name(&self) -> &str {
        return &self.module_name;
    }

    pub fn get_obj(&self, name: &str) -> Option<&u64> {
        return self.export_object.get(name);
    }

    pub fn register_obj(&mut self, name: &'a str, obj_id: u64) {
        self.export_object.insert(name, obj_id);
    }

    pub fn load_from_runtime(name: &str, rt: &mut FSRThreadRuntime) -> Result<Self, RuntimeBaseError> {
        let module = Self {
            export_object: HashMap::new(),
            module_name: name.to_string(),
        };
        return Ok(module);
    }

    pub fn colon_operator(&self, name: &str) -> Option<u64> {
        let obj = match self.export_object.get(name) {
            Some(o) => o,
            None => {
                return None;
            }
        };

        return Some(obj.clone());
    }

    pub fn self_module() -> Self {
        let module = Self {
            export_object: HashMap::new(),
            module_name: "Self".to_string(),
        };
        return module;
    }
}

impl IFSRObject for FSRModule<'_> {
    fn init(&mut self) {
        todo!()
    }

    fn get_class_name() -> &'static str {
        todo!()
    }

    fn get_class(vm: &crate::backend::vm::vm::FSRVirtualMachine) -> super::base::FSRClass {
        todo!()
    }
}