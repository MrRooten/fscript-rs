#![allow(unused)]

use std::collections::HashMap;

use crate::{
    backend::{base_type::{base::{FSRClass, FSRObject, FSRObjectManager, FSRVMClsMgr}, utils::i_to_m}, internal_lib::io::register_io},
    utils::error::FSRRuntimeError,
};
use crate::backend::base_type::bool::FSRBool;
use crate::backend::base_type::none::FSRNone;
use crate::backend::vm::module::FSRRuntimeModule;

use super::thread::FSRThread;


pub struct FSRVirtualMachine<'a> {
    base_id     : u64,
    var_mgr     : FSRObjectManager<'a>,
    register    : Option<FSRVMClsMgr>,
    threads     : HashMap<u64, FSRThread>,
    global_var  : HashMap<&'static str, u64>,
}

impl<'a> FSRVirtualMachine<'a> {
    pub fn get_cls(&self, name: &str) -> Option<&FSRClass> {
        if let Some(s) = &self.register {
            return s.get_cls(name);
        }
        return None;
    }

    pub fn get_obj_by_id(&self, var_id: &u64) -> Option<&FSRObject<'a>> {
        return self.var_mgr.get_obj_by_id(&var_id);
    }

    pub fn get_mut_obj_by_id(&mut self, var_id: &u64) -> Option<&mut FSRObject<'a>> {
        return self.var_mgr.get_mut_obj_by_id(&var_id);
    }

    
    pub fn get_true_id(&self) -> u64 {
        return 1;
    }
    
    pub fn get_false_id(&self) -> u64 {
        return 2;
    }
    
    pub fn get_none_id(&self) -> u64 {
        return 0;
    }
    
    pub fn register_global_with_name(&mut self, name: &'static str, obj: u64) -> Result<(),&str> {
        if self.get_obj_by_id(&obj).is_none() {
            return  Err("");
        }

        self.global_var.insert(name, obj);
        return Ok(());
    }

    fn init_global_obj(&mut self) {
        let true_obj = FSRBool::new(true, &self, 1);
        self.global_var.insert("true", true_obj.get_id());
        let false_obj = FSRBool::new(false, &self, 2);
        self.global_var.insert("false", false_obj.get_id());
        let none_obj = FSRNone::new(&self);
        self.global_var.insert("none", none_obj.get_id());

        register_io(self);
    }

    pub fn get_global_by_name(&self, name: &str) -> Option<&u64> {
        return self.global_var.get(name);
    }

    pub fn new() -> Result<FSRVirtualMachine<'a>, FSRRuntimeError<'a>> {
        let obj_mgr = FSRObjectManager::new();
        
        let mut s = Self {
            var_mgr: obj_mgr,
            register: None,
            threads: HashMap::new(),
            base_id: 1000,
            global_var: Default::default(),

        };
        let register = FSRVMClsMgr::new(&s);
        s.register = Some(register);
        s.init_global_obj();

        return Ok(s);
    }

    pub fn new_id(&mut self) -> u64 {
        self.base_id += 1;
        return self.base_id;
    }

    pub fn init_context(&self, context: &mut FSRRuntimeModule) {
        context.init(&self.global_var);
    }

    pub fn register_obj(&mut self, obj: FSRObject<'a>) {
        self.var_mgr.register_obj(obj.get_id(), obj);
    }

    pub fn run_code(&mut self, code: &[u8], context: &FSRRuntimeModule) {
        i_to_m(context).run_code(code, self);

    }

    pub fn get_obj_by_name(&mut self, name: &'a str, context: &'a FSRRuntimeModule<'a>) -> Option<u64> {
        let obj = context.get_obj_by_name(name, self).unwrap();
        return Some(obj.get_id());
    }
}
