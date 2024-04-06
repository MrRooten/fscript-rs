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
        let name = cls.get_name();
        let attrs = HashMap::new();
        let mut cls_attrs = HashMap::new();
        let block = cls.get_block();
        for token in block.get_tokens() {
            if let FSRToken::Assign(a) = token {
                let v_name = a.get_name();
                let v = rt.run_token(a.get_assign_expr(), vm, None)?;
                cls_attrs.insert(v_name, v);

            }

            else if let FSRToken::FunctionDef(fn_def) = token {
                if fn_def.get_name().eq("__new__") {

                }
                let fn_obj = FSRFn::from_ast(fn_def, vm);
                cls_attrs.insert(fn_def.get_name(), fn_obj.get_id());
            }
        }

        let cls_obj = FSRObject::new(vm);
        //cls_obj.set_cls(vm.get_cls(Self::get_class_name()).unwrap());
        let v = Self { attrs, cls_attrs, name };
        cls_obj.set_value(FSRValue::Class(v));
        return Ok(cls_obj.get_id());
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
    let s = rt.find_symbol("self", vm, None).unwrap();
    let self_obj = vm.get_obj_by_id(&s).unwrap();

    if let FSRValue::Class(c) = self_obj.get_value() {
        let s = format!("<Class '{}'>", c.get_name());
        let obj = FSRString::from(s, vm);
        return Ok(obj.get_id());
    }
    
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