#![allow(unused)]

use crate::backend::base_type::base::{FSRArgs, FSRObject, FSRValue, IFSRObject};
use crate::backend::base_type::class::FSRClassBackEnd;
use crate::backend::base_type::class_inst::FSRClassInstance;
use crate::backend::base_type::function::FSRFn;
use crate::backend::base_type::integer::FSRInteger;
use crate::backend::base_type::list::FSRList;
use crate::backend::base_type::module::{self, FSRModule};
use crate::backend::base_type::string::FSRString;
use crate::backend::base_type::utils::i_to_m;
use crate::backend::std::path::register_path;
use crate::backend::vm::vm::FSRVirtualMachine;
use crate::frontend::ast::token::assign::FSRAssign;
use crate::frontend::ast::token::base::{FSRMeta, FSRToken};
use crate::frontend::ast::token::block::FSRBlock;
use crate::frontend::ast::token::call::FSRCall;
use crate::frontend::ast::token::class::FSRClassFrontEnd;
use crate::frontend::ast::token::constant::{FSRConstant, FSRConstantType};
use crate::frontend::ast::token::expr::FSRExpr;
use crate::frontend::ast::token::function_def::FSRFnDef;
use crate::frontend::ast::token::if_statement::FSRIf;
use crate::frontend::ast::token::list::FSRListFrontEnd;
use crate::frontend::ast::token::module::FSRModuleFrontEnd;
use crate::frontend::ast::token::while_statement::FSRWhile;
use crate::utils::error::{FSRRuntimeError, FSRRuntimeType};
use std::collections::HashMap;
use std::rc::Weak;
use std::str;


#[derive(Debug, Clone)]
pub struct FSRLocalVars<'a> {
    local_vars: HashMap<&'a str, u64>,
}

#[derive(Debug, Clone)]
pub struct VMCallState<'a> {
    fn_name: String,
    local_vars: Vec<FSRLocalVars<'a>>,
    cur_token: Option<*const FSRToken<'a>>,
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

pub struct FSRThreadRuntime<'a> {
    call_stack: Vec<VMCallState<'a>>,
    modules: HashMap<String, FSRModule<'a>>,
    meta: FSRMeta,
    is_ret: bool,
    ret_value: Option<u64>,
}

pub enum FSRArg<'a> {
    String(&'a str),
    Expr(&'a FSRToken<'a>),
    Assign(&'a FSRAssign<'a>),
}
