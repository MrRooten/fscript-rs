use std::collections::HashMap;

use crate::{
    backend::{
        base_type::{
            base::{FSRObject, FSRValue},
            function::FSRFn, string::FSRString,
        },
        vm::{runtime::FSRThreadRuntime, vm::FSRVirtualMachine},
    },
    frontend::ast::token::{base::FSRToken, class::FSRClassFrontEnd},
    utils::error::FSRRuntimeError,
};

use super::base::{FSRBaseType, IFSRObject};

#[derive(Debug)]
pub struct FSRClassBackEnd<'a> {
    name    : &'a str,
    attrs: HashMap<&'a str, FSRToken<'a>>,
    cls_attrs: HashMap<&'a str, u64>,
}

impl<'a> FSRClassBackEnd<'a> {
    pub fn get_name(&self) -> &'a str {
        return self.name
    }

    pub fn get_attrs(&self) -> &HashMap<&'a str, FSRToken<'a>> {
        return &self.attrs
    }

    pub fn get_cls_attrs(&self) -> &HashMap<&'a str, u64> {
        return &self.cls_attrs
    }

    pub fn from_cls(
        cls: &'a FSRClassFrontEnd<'a>,
        rt: &'a FSRThreadRuntime<'a>,
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<u64, FSRRuntimeError<'a>> {
        unimplemented!()
    }

    pub fn get_cls_attr(&self, name: &str) -> Option<&u64> {
        match self.cls_attrs.get(name) {
            Some(s) => Some(s),
            None => None
        }
    }

    pub fn set_cls_attr(&mut self, name: &'a str, value: u64) {
        self.cls_attrs.insert(name, value);
    }

    
}

fn register_to_string_func<'a>(vm: &'a FSRVirtualMachine, rt: &'a mut FSRThreadRuntime) -> Result<u64, FSRRuntimeError<'a>> {
    unimplemented!()
}

impl IFSRObject for FSRClassBackEnd<'_> {
    fn init(&mut self) {
        todo!()
    }

    fn get_class_name() -> &'static str {
        "Class"
    }

    fn get_class(vm: &FSRVirtualMachine) -> super::base::FSRBaseType {
        let mut cls = FSRBaseType::new("Class");
        let fn_obj = FSRFn::from_func(register_to_string_func, vm, vec!["self"]);
        cls.register_obj("__str__", fn_obj.get_id());
        return cls;
    }
}