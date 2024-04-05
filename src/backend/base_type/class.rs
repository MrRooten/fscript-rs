use std::{collections::HashMap, rc::Rc};

use crate::{
    backend::{
        base_type::{
            base::{FSRObject, FSRValue},
            function::FSRFn,
        },
        vm::{runtime::FSRThreadRuntime, vm::FSRVirtualMachine},
    },
    frontend::ast::token::{base::FSRToken, class::FSRClassFrontEnd, function_def::FSRFnDef},
    utils::error::FSRRuntimeError,
};

#[derive(Debug)]
pub struct FSRClassBackEnd<'a> {
    name    : &'a str,
    attrs: HashMap<&'a str, FSRToken<'a>>,
    cls_attrs: HashMap<&'a str, u64>,
}

impl<'a> FSRClassBackEnd<'a> {
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
        let name = cls.get_name();
        let mut attrs = HashMap::new();
        let mut cls_attrs = HashMap::new();
        let block = cls.get_block();
        for token in block.get_tokens() {
            if let FSRToken::Assign(a) = token {
                let v_name = a.get_name();
                if v_name.starts_with("Self.") {
                    let s_name = &v_name[5..];
                    let v = rt.run_token(a.get_assign_expr(), vm, None)?;
                    cls_attrs.insert(s_name, v);
                    continue;
                }

                attrs.insert(a.get_name(), (**a.get_assign_expr()).clone());
            }

            else if let FSRToken::FunctionDef(fn_def) = token {
                if fn_def.get_name().eq("__new__") {

                }
                let fn_obj = FSRFn::from_ast(fn_def, vm);
                cls_attrs.insert(fn_def.get_name(), fn_obj.get_id());
            }
        }

        let cls_obj = FSRObject::new(vm);
        let v = Self { attrs, cls_attrs, name };
        cls_obj.set_value(FSRValue::Class(v));
        return Ok(cls_obj.get_id());
    }

    pub fn get_cls_attr(&self, name: &str) -> Option<u64> {
        match self.cls_attrs.get(name) {
            Some(s) => Some(s.clone()),
            None => None
        }
    }

    pub fn set_cls_attr(&mut self, name: &'a str, value: u64) {
        self.cls_attrs.insert(name, value);
    }

    pub fn init_object(&self, rt: &'a FSRThreadRuntime<'a>, vm: &'a FSRVirtualMachine<'a>) {
        
    }
}
