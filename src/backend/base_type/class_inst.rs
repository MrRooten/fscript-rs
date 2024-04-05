use std::collections::HashMap;

use crate::{
    backend::{
        base_type::base::FSRValue,
        vm::{runtime::FSRThreadRuntime, vm::FSRVirtualMachine},
    },
    frontend::ast::token::base::FSRMeta,
    utils::error::{FSRRuntimeError, FSRRuntimeType},
};

use super::{base::FSRObject, class::FSRClassBackEnd};

#[derive(Debug)]
pub struct FSRClassInstance<'a> {
    pub(crate) attrs: HashMap<&'a str, u64>,
    pub(crate) cls: &'a FSRClassBackEnd<'a>,
}

impl<'a> FSRClassInstance<'a> {
    pub fn from_inst(
        inst: FSRClassInstance,
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<u64, FSRRuntimeError<'a>> {
        let object = FSRObject::new(vm);
        if object.has_method("__new__", vm) {

        }
        object.set_value(FSRValue::ClassInst(inst));
        return Ok(object.get_id());
    }

    pub fn get_attr(
        &self,
        name: &str,
        rt: &'a FSRThreadRuntime,
        meta: FSRMeta,
    ) -> Result<u64, FSRRuntimeError> {
        if let Some(s) = self.attrs.get(name) {
            return Ok(s.clone());
        }

        if let Some(s) = self.cls.get_cls_attr(name) {
            return Ok(s.clone());
        }

        let err = FSRRuntimeError::new(
            rt.get_call_stack(),
            FSRRuntimeType::NotFoundObject,
            format!("not found object {}", name),
            &meta,
        );
        return Err(err);
    }

    pub fn get_attr_option(
        &self,
        name: &str,
    ) -> Option<u64> {
        if let Some(s) = self.attrs.get(name) {
            return Some(s.clone());
        }

        if let Some(s) = self.cls.get_cls_attr(name) {
            return Some(s.clone());
        }

        return None;
    }

    pub fn set_attr(
        &mut self,
        name: &'a str,
        value: u64) {
        self.attrs.insert(name, value);
    }

    pub fn has_method(&self, method: &str, vm: &'a FSRVirtualMachine<'a>) -> bool {
        let obj = match self.get_attr_option(method) {
            Some(s) => s,
            None => {
                return false;
            }
        };
        let obj = match vm.get_obj_by_id(&obj) {
            Some(s) => s,
            None => {
                unimplemented!()
            }
        };

        return obj.is_function();
    }
}
