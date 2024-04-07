use std::collections::HashMap;

use crate::{backend::base_type::module::FSRModule, frontend::ast::token::base::{FSRMeta, FSRToken}};

#[derive(Debug, Clone)]
pub struct FSRLocalVars<'a> {
    local_vars: HashMap<&'a str, u64>,
}

impl FSRLocalVars<'_> {
    pub fn get_var(&self, name: &str) -> Option<u64> {
        if let Some(s) = self.local_vars.get(name) {
            return Some(s.clone());
        }

        return None;
    }

    pub fn new() -> Self {
        return Self {
            local_vars: Default::default(),
        };
    }
}

#[derive(Debug, Clone)]
pub struct VMCallState<'a> {
    fn_name: String,
    local_vars: Vec<FSRLocalVars<'a>>,
    cur_token: Option<*const FSRToken<'a>>,
}

impl<'a> VMCallState<'a> {
    pub fn get_string(&self) -> &str {
        return &self.fn_name;
    }

    pub fn push_local_block_vars(&mut self) {
        self.local_vars.push(FSRLocalVars::new());
    }

    pub fn pop_local_block_vars(&mut self) {
        self.local_vars.pop();
    }

    pub fn set_cur_token(&mut self, token: &FSRToken<'a>) {
        self.cur_token = Some(token);
    }

    pub fn get_cur_token(&self) -> &Option<*const FSRToken<'a>> {
        return &self.cur_token;
    }
}

impl<'a> VMCallState<'a> {
    pub fn get_local_vars(&mut self) -> &mut Vec<FSRLocalVars<'a>> {
        return &mut self.local_vars;
    }

    pub fn new(name: &str) -> VMCallState<'a> {
        Self {
            fn_name: name.to_string(),
            local_vars: vec![FSRLocalVars::new()],
            cur_token: None,
        }
    }
}


pub struct FSRThreadRuntime2<'a> {
    call_stack: Vec<VMCallState<'a>>,
    modules: HashMap<String, FSRModule<'a>>,
    meta: FSRMeta,
    is_ret: bool,
    ret_value: Option<u64>,
}