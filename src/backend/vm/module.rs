#![allow(unused)]

use crate::backend::base_type::base::{FSRArgs, FSRObject};
use crate::backend::base_type::function::FSRFn;
use crate::backend::base_type::integer::FSRInteger;
use crate::backend::base_type::list::FSRList;
use crate::backend::base_type::string::FSRString;
use crate::backend::base_type::utils::i_to_m;
use crate::backend::vm::vm::FSRVirtualMachine;
use crate::frontend::ast::token::assign::FSRAssign;
use crate::frontend::ast::token::base::{FSRMeta, FSRToken};
use crate::frontend::ast::token::block::FSRBlock;
use crate::frontend::ast::token::call::FSRCall;
use crate::frontend::ast::token::constant::{FSRConstant, FSRConstantType};
use crate::frontend::ast::token::expr::FSRExpr;
use crate::frontend::ast::token::for_statement::FSRFor;
use crate::frontend::ast::token::function_def::FSRFnDef;
use crate::frontend::ast::token::if_statement::FSRIf;
use crate::frontend::ast::token::list::FSRListFrontEnd;
use crate::frontend::ast::token::module::FSRModule;
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
    cur_token   : Option<* const FSRToken<'a>>
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
}

impl<'a> VMCallState<'a> {
    pub fn get_local_vars(&mut self) -> &mut Vec<FSRLocalVars<'a>> {
        return &mut self.local_vars;
    }

    pub fn new(name: &str) -> VMCallState<'a> {
        Self {
            fn_name: name.to_string(),
            local_vars: vec![FSRLocalVars::new()],
            cur_token: None
        }
    }
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

pub struct FSRRuntimeModule<'a> {
    call_stack: Vec<VMCallState<'a>>,
    global_vars: HashMap<&'a str, u64>,
    meta        : FSRMeta
}

impl<'a> FSRRuntimeModule<'a> {
    pub fn get_cur_meta(&self) -> &FSRMeta {
        return &self.meta;
    }

    pub fn get_call_stack(&self) -> &Vec<VMCallState<'a>> {
        return &self.call_stack;
    }

    pub fn push_call_stack(&mut self, name: &str) {
        self.call_stack.push(VMCallState::new(name));
    }

    pub fn pop_call_stack(&mut self) {
        self.call_stack.pop();
    }

    fn get_cur_stack(&mut self) -> &mut VMCallState<'a> {
        let len = self.call_stack.len();
        return self.call_stack.get_mut(len - 1).unwrap();
    }

    fn get_stack(&mut self, id: usize) -> &mut VMCallState<'a> {
        return self.call_stack.get_mut(id).unwrap();
    }

    fn get_stack_len(&self) -> usize {
        return self.call_stack.len();
    }

    pub fn new() -> FSRRuntimeModule<'a> {
        let mut module = Self {
            call_stack: vec![],
            global_vars: Default::default(),
            meta: FSRMeta::new(),
        };
        module.call_stack.push(VMCallState::new("root"));
        return module;
    }

    pub fn init(&mut self, vars: &HashMap<&'static str, u64>) {
        for kv in vars {
            self.global_vars.insert(kv.0, *kv.1);
        }
    }

    pub fn find_symbol(
        &self,
        name: &str,
        vm: &'a FSRVirtualMachine<'a>,
        stack_id: Option<usize>,
    ) -> Result<u64, FSRRuntimeError> {
        if let Some(id) = stack_id {
            for local_var in i_to_m(self).get_stack(id).local_vars.iter().rev() {
                if let Some(id) = local_var.get_var(name) {
                    return Ok(id);
                }
            }

            if let Some(id) = self.global_vars.get(name) {
                return Ok(id.clone());
            }

            if let Some(id) = vm.get_global_by_name(name) {
                return Ok(id.clone());
            }

            let err = FSRRuntimeError::new(
                &self.call_stack, 
                FSRRuntimeType::NotFoundObject, 
                format!("not found object: {}", name), 
                self.get_cur_meta());
            return Err(err);
        }
        for local_var in i_to_m(self).get_cur_stack().local_vars.iter().rev() {
            if let Some(id) = local_var.get_var(name) {
                return Ok(id);
            }
        }

        if let Some(id) = self.global_vars.get(name) {
            return Ok(id.clone());
        }

        if let Some(id) = vm.get_global_by_name(name) {
            return Ok(id.clone());
        }

        let err = FSRRuntimeError::new(
            &self.call_stack, 
            FSRRuntimeType::NotFoundObject, 
            format!("not found object: {}", name), 
            self.get_cur_meta());
        return Err(err);
    }

    fn invoke_binary_op(
        &mut self,
        left: &'a FSRObject<'a>,
        op: &str,
        right: &'a FSRObject<'a>,
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<u64, FSRRuntimeError> {
        i_to_m(self).assign_variable("other", right.get_id(), vm)?;
        let mut v = 0;
        if op.eq("+") {
            v = left.invoke_method("add", vm, i_to_m(self))?;
        } else if op.eq("-") {
            v = left.invoke_method("sub", vm, i_to_m(self))?;
        } else if op.eq("*") {
            v = left.invoke_method("mul", vm, i_to_m(self))?;
        } else if op.eq("/") {
            v = left.invoke_method("div", vm, i_to_m(self))?;
        } else if op.eq("==") {
            v = left.invoke_method("eq", vm, i_to_m(self))?;
        } else if op.eq("!=") {
            v = left.invoke_method("not_eq", vm, i_to_m(self))?;
        } else if op.eq("<<") {
            v = left.invoke_method("left_shift", vm, i_to_m(self))?;
        } else if op.eq(">>") {
            v = left.invoke_method("right_shift", vm, i_to_m(self))?;
        } else if op.eq(">") {
            v = left.invoke_method("greater", vm, i_to_m(self))?;
        } else if op.eq("<") {
            v = left.invoke_method("less", vm, i_to_m(self))?;
        } else if op.eq(">=") {
            v = left.invoke_method("greater_equal", vm, i_to_m(self))?;
        } else if op.eq("<=") {
            v = left.invoke_method("less_equal", vm, i_to_m(self))?;
        }

        return Ok(v);
    }

    fn register_constant(
        &mut self,
        c: &FSRConstant,
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<u64, FSRRuntimeError> {
        let op = c.single_op;
        if let FSRConstantType::String(s) = c.get_constant() {
            let s = str::from_utf8(s).unwrap();
            let var = FSRString::from(s, vm);
            return Ok(var.get_id());
        }

        if let FSRConstantType::Integer(i) = c.get_constant() {
            let mut multi = 1;
            if let Some(s) = op {
                if s.eq("-") {
                    multi = -1;
                }
            }
            let var = FSRInteger::from_i64(i.clone() * multi, vm);

            return Ok(var.get_id());
        }

        if let FSRConstantType::Float(f) = c.get_constant() {
            unimplemented!()
        }
        unimplemented!()
    }

    fn call_func(&self, c: &FSRCall, vm: &'a FSRVirtualMachine<'a>) -> Result<u64, FSRRuntimeError> {
        let fn_id = self.find_symbol(&c.get_name(), vm, None).unwrap();
        let fn_obj = vm.get_obj_by_id(&fn_id).unwrap();
        let fn_obj = match fn_obj.get_value() {
            crate::backend::base_type::base::FSRValue::Function(c) => c,
            _ => {
                let err = FSRRuntimeError::new(
                    self.get_call_stack(),
                    FSRRuntimeType::NotFoundObject,
                    format!("Not found object id, {:?}", fn_id),
                    &c.get_meta(),
                );
                return Err(err);
            }
        };

        let args = fn_obj.get_args();
        let target_args = c.get_args();
        let mut i = 0;
        let mut fn_args = vec![];
        while i < args.len() {
            let mut v = 0;
            if i < target_args.len() {
                v = self.run_token(&target_args[i], vm, None).unwrap();
            }

            fn_args.push((&args[i], v));
            i += 1;
        }
        i_to_m(self).push_call_stack(c.get_name());
        for a in fn_args {
            i_to_m(self).assign_variable(a.0, a.1, vm)?;
        }
        let v = fn_obj.invoke(vm, i_to_m(self))?;

        i_to_m(self).pop_call_stack();
        return Ok(v);
    }

    pub fn assign_variable(
        &mut self,
        name: &'a str,
        id: u64,
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<(), FSRRuntimeError> {
        let obj = i_to_m(vm).get_mut_obj_by_id(&id).unwrap();
        obj.ref_object();
        let l = self.get_cur_stack().local_vars.len() - 1;
        let cur_stack = self.get_cur_stack();
        let local_vars = cur_stack.get_local_vars();
        local_vars
            .get_mut(l)
            .unwrap()
            .local_vars
            .insert(name, obj.get_id());
        return Ok(());
    }

    pub fn assign_global_variable(
        &mut self,
        name: &'a str,
        id: u64,
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<(), &str> {
        let obj = i_to_m(vm).get_mut_obj_by_id(&id).unwrap();
        obj.ref_object();
        self.global_vars.insert(name, obj.get_id());
        return Ok(());
    }

    fn run_list(&self, e: &'a FSRListFrontEnd<'a>, vm: &'a FSRVirtualMachine<'a>) -> Result<u64, FSRRuntimeError> {
        let items = e.get_items();
        let mut vs = vec![];
        for t in items {
            let v = self.run_token(t, vm, None)?;
            vs.push(v);
        }
        
        let v = FSRList::from_list(vs, vm)?;
        return Ok(v);
    }

    fn run_assign(&self, e: &'a FSRAssign<'a>, vm: &'a FSRVirtualMachine<'a>) -> Result<(), FSRRuntimeError> {
        let right = e.get_assign_expr();
        i_to_m(self).get_cur_stack().set_cur_token(&*right);
        if let FSRToken::Expr(ex) = &**right {
            let v = self.run_expr(ex, vm)?;
            let cur_stack = i_to_m(self).get_cur_stack();
            let local_vars = cur_stack.get_local_vars();
            for local_var in local_vars.iter_mut().rev() {
                if local_var.local_vars.get(e.get_name()).is_none() == false {
                    local_var.local_vars.insert(e.get_name(), v).unwrap();
                    return Ok(());
                }
            }

            if self.global_vars.get(e.get_name()).is_none() == false {
                i_to_m(self).global_vars.insert(e.get_name(), v);
            }

            i_to_m(self).assign_variable(e.get_name(), v, vm)?;
            return Ok(());
        } else if let FSRToken::Constant(c) = &**right {
            let v = i_to_m(self).register_constant(c, vm)?;

            for local_var in i_to_m(self).get_cur_stack().local_vars.iter_mut().rev() {
                if local_var.local_vars.get(e.get_name()).is_none() == false {
                    local_var.local_vars.insert(e.get_name(), v).unwrap();
                    return Ok(());
                }
            }

            if self.global_vars.get(e.get_name()).is_none() == false {
                i_to_m(self).global_vars.insert(e.get_name(), v);
            }

            i_to_m(self).assign_variable(e.get_name(), v, vm)?;
            return Ok(());
        } else if let FSRToken::Variable(v) = &**right {
            let v = self.find_symbol(v.get_name(), vm, None)?;
            let obj = i_to_m(vm).get_mut_obj_by_id(&v).unwrap();

            for local_var in i_to_m(self).get_cur_stack().local_vars.iter_mut().rev() {
                if local_var.local_vars.get(e.get_name()).is_none() == false {
                    local_var.local_vars.insert(e.get_name(), v).unwrap();
                    return Ok(());
                }
            }

            if self.global_vars.get(e.get_name()).is_none() == false {
                i_to_m(self).global_vars.insert(e.get_name(), v);
            }

            i_to_m(self).assign_variable(e.get_name(), obj.get_id(), vm)?;
            return Ok(());
        } else if let FSRToken::List(l) = &**right {
            let v = self.run_list(l, vm)?;
            i_to_m(self).assign_variable(e.get_name(), v, vm)?;
            return Ok(());
        }

        unimplemented!()
    }

    fn run_expr(
        &self,
        e: &'a FSRExpr<'a>,
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<u64, FSRRuntimeError> {
        if e.get_op().eq(".") {
            i_to_m(self).get_cur_stack().set_cur_token(&*e.get_left());
            if let FSRToken::Variable(v) = &**e.get_right() {
                let mut l_value = 0;
                if let FSRToken::Expr(l) = &**e.get_left() {
                    l_value = self.run_expr(l, vm)?;
                } else if let FSRToken::Constant(c) = &**e.get_left() {
                    l_value = i_to_m(self).register_constant(c, vm)?;
                } else if let FSRToken::Call(c) = &**e.get_left() {
                    l_value = i_to_m(self).call_func(c, vm)?;
                } else if let FSRToken::Variable(v) = &**e.get_left() {
                    l_value = self.find_symbol(v.get_name(), vm, None)?;
                } else {
                    let err = FSRRuntimeError::new(
                        &self.call_stack,
                        FSRRuntimeType::TokenNotMatch,
                        format!("Not match token: {:?}", e.get_left()),
                        e.get_meta(),
                    );
                    return Err(err);
                }

                let l_obj = vm.get_obj_by_id(&l_value).unwrap();
                let name = v.get_name();
                let attr_id = match l_obj.get_attr(name, vm) {
                    Some(s) => s,
                    None => {
                        let err = FSRRuntimeError::new(
                            &self.call_stack,
                            FSRRuntimeType::NotFoundObject,
                            format!("not found object: {}", name),
                            e.get_meta(),
                        );
                        return Err(err);
                    }
                };

                return Ok(attr_id.clone());
            }
            i_to_m(self).get_cur_stack().set_cur_token(&*e.get_right());
            if let FSRToken::Call(call) = &**e.get_right() {

                let mut l_value = 0;
                if let FSRToken::Expr(l) = &**e.get_left() {
                    l_value = self.run_expr(l, vm)?;
                } else if let FSRToken::Constant(c) = &**e.get_left() {
                    l_value = i_to_m(self).register_constant(c, vm)?;
                } else if let FSRToken::Call(c) = &**e.get_left() {
                    l_value = i_to_m(self).call_func(c, vm)?;
                } else if let FSRToken::Variable(v) = &**e.get_left() {
                    l_value = self.find_symbol(v.get_name(), vm, None)?;
                } else {
                    let err = FSRRuntimeError::new(
                        &self.call_stack,
                        FSRRuntimeType::TokenNotMatch,
                        format!("Not match token: {:?}", e.get_left()),
                        e.get_meta(),
                    );
                    return Err(err);
                }

                let l_obj = vm.get_obj_by_id(&l_value).unwrap();
                let name = call.get_name();
                let args = FSRArgs::new();
                let ret = l_obj.invoke_method(name, vm, i_to_m(self))?;
                return Ok(ret);
            }

            let err = FSRRuntimeError::new(
                &self.call_stack,
                FSRRuntimeType::NotValidAttr,
                format!(". operator right v is method or attr"),
                e.get_meta(),
            );
            return Err(err);
        }
        let mut l_value: Option<u64> = None;
        let mut r_value: Option<u64> = None;
        i_to_m(self).get_cur_stack().set_cur_token(&*e.get_left());
        if let FSRToken::Expr(l) = &**e.get_left() {
            l_value = Some(self.run_expr(l, vm)?);
        } else if let FSRToken::Constant(c) = &**e.get_left() {
            l_value = Some(i_to_m(self).register_constant(c, vm)?);
        } else if let FSRToken::Call(c) = &**e.get_left() {
            l_value = Some(i_to_m(self).call_func(c, vm)?);
        } else if let FSRToken::Variable(v) = &**e.get_left() {
            l_value = Some(self.find_symbol(v.get_name(), vm, None)?);
        } else {
            let err = FSRRuntimeError::new(
                &self.call_stack,
                FSRRuntimeType::TokenNotMatch,
                format!("Not match token: {:?}", e.get_left()),
                e.get_meta(),
            );
            return Err(err);
        }

        i_to_m(self).get_cur_stack().set_cur_token(&*e.get_right());
        if let FSRToken::Expr(r) = &**e.get_right() {
            r_value = Some(self.run_expr(r, vm)?);
        } else if let FSRToken::Constant(c) = &**e.get_right() {
            r_value = Some(i_to_m(self).register_constant(c, vm)?);
        } else if let FSRToken::Call(c) = &**e.get_right() {
            r_value = Some(i_to_m(self).call_func(c, vm)?);
        } else if let FSRToken::Variable(v) = &**e.get_right() {
            r_value = Some(self.find_symbol(v.get_name(), vm, None)?);
        } else {
            let err = FSRRuntimeError::new(
                &self.call_stack,
                FSRRuntimeType::TokenNotMatch,
                format!("Not match token: {:?}", e.get_left()),
                e.get_meta(),
            );
            return Err(err);
        }

        let l_obj = match vm.get_obj_by_id(&l_value.unwrap()) {
            Some(s) => s,
            None => {
                let err = FSRRuntimeError::new(
                    &self.call_stack,
                    FSRRuntimeType::NotFoundObject,
                    format!("Not found object id, {}", l_value.unwrap()),
                    e.get_meta(),
                );
                return Err(err);
            }
        };

        let r_obj = match vm.get_obj_by_id(&r_value.unwrap()) {
            Some(s) => s,
            None => {
                let err = FSRRuntimeError::new(
                    &self.call_stack,
                    FSRRuntimeType::NotFoundObject,
                    format!("Not found object id, {}", l_value.unwrap()),
                    e.get_meta(),
                );
                return Err(err);
            }
        };

        return i_to_m(self).invoke_binary_op(l_obj, e.get_op(), r_obj, vm);
    }

    pub fn run_block(
        &mut self,
        block: &FSRBlock<'a>,
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<u64, &str> {
        self.get_cur_stack().push_local_block_vars();
        let mut v = 0;
        for token in block.get_tokens() {
            self.get_cur_stack().set_cur_token(token);
            v = match self.run_token(token, vm, None) {
                Ok(o) => o,
                Err(e) => return Err("run error"),
            };
        }
        self.get_cur_stack().pop_local_block_vars();
        return Ok(v);
    }

    pub fn run_if_block(
        &self,
        if_expr: &'a FSRIf<'a>,
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<(), FSRRuntimeError> {
        let expr = if_expr.get_test();
        i_to_m(self).get_cur_stack().set_cur_token(&*expr);
        let mut l_value: Option<u64> = None;
        if let FSRToken::Expr(l) = &**expr {
            l_value = Some(self.run_expr(l, vm)?);
        } else if let FSRToken::Constant(c) = &**expr {
            l_value = Some(i_to_m(self).register_constant(c, vm)?);
        } else if let FSRToken::Call(c) = &**expr {
            l_value = Some(i_to_m(self).call_func(c, vm)?);
        } else if let FSRToken::Variable(v) = &**expr {
            l_value = Some(self.find_symbol(v.get_name(), vm, None)?);
        } else {
            let err = FSRRuntimeError::new(
                &self.call_stack, 
                FSRRuntimeType::TokenNotMatch, 
                format!("Not match token: {:?}", expr), 
                if_expr.get_meta());
            return Err(err);
        }

        let v = match l_value {
            Some(s) => s,
            None => {
                let err = FSRRuntimeError::new(
                    &self.call_stack, 
                    FSRRuntimeType::TokenNotMatch, 
                    format!("Not match token: {:?}", expr), 
                    if_expr.get_meta());
                return Err(err);
            }
        };

        if v != vm.get_false_id() && v != vm.get_none_id() {
            let block = &**if_expr.get_block();
            i_to_m(self).get_cur_stack().push_local_block_vars();
            i_to_m(self).run_block(block, vm);
            i_to_m(self).get_cur_stack().pop_local_block_vars();
        }

        return Ok(());
    }

    pub fn run_token(
        &self,
        token: &'a FSRToken<'a>,
        vm: &'a FSRVirtualMachine<'a>,
        stack_id: Option<usize>,
    ) -> Result<u64, FSRRuntimeError> {
        let mut l_value: Option<u64> = None;
        if let FSRToken::Expr(l) = &*token {
            l_value = Some(self.run_expr(l, vm)?);
        } else if let FSRToken::Constant(c) = &*token {
            l_value = Some(i_to_m(self).register_constant(c, vm)?);
        } else if let FSRToken::Call(c) = &*token {
            l_value = Some(i_to_m(self).call_func(c, vm)?);
        } else if let FSRToken::Variable(v) = &*token {
            l_value = Some(self.find_symbol(v.get_name(), vm, stack_id)?);
        } else if let FSRToken::Assign(a) = &*token {
            self.run_assign(a, vm);
        } else if let FSRToken::IfExp(if_def) = &*token {
            self.run_if_block(if_def, vm);
        } else {
            let err = FSRRuntimeError::new(
                &self.call_stack, 
                FSRRuntimeType::TokenNotMatch, 
                format!("Not match token: {:?}", token), 
                token.get_meta());
            return Err(err);
        }

        let v = match l_value {
            Some(s) => s,
            None => {
                return Ok(0);
            }
        };
        return Ok(v);
    }

    pub fn run_for_block(
        &self,
        for_expr: &'a FSRFor<'a>,
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<(), FSRRuntimeError> {
        let expr = for_expr.get_test();
        let mut v = self.run_token(&**expr, vm, None)?;

        while v != vm.get_false_id() && v != vm.get_none_id() {
            i_to_m(self).run_block(&**for_expr.get_block(), vm);
            v = self.run_token(&**expr, vm, None)?;
        }

        unimplemented!()
    }

    pub fn define_function(
        &mut self,
        fn_def: &FSRFnDef<'a>,
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<u64, &str> {
        let args = fn_def.get_args();
        let mut fn_args = vec![];
        for arg in args {
            if let FSRToken::Variable(v) = arg {
                fn_args.push(v.get_name());
            } else if let FSRToken::Assign(a) = arg {
                fn_args.push(a.get_name());
            } else {
                return Err("Not valid fn define args");
            }
        }
        let fn_obj = FSRFn::from_ast(fn_def.clone(), vm, fn_args);

        let fn_id = fn_obj.get_id();
        self.assign_global_variable(fn_def.get_name(), fn_id, vm);
        return Ok(fn_id);
    }

    pub fn run_ast_fn(
        &self,
        fn_def: &FSRFnDef<'a>,
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<u64, &str> {
        let body = fn_def.get_body();
        let v = i_to_m(self).run_block(body, vm);
        match v {
            Ok(o) => Ok(o),
            Err(e) => Err("run_ast_fn error"),
        }
    }

    pub fn run_ast(
        &mut self,
        ast: &FSRToken<'a>,
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<(), FSRRuntimeError> {
        self.get_cur_stack().set_cur_token(ast);
        if let FSRToken::Expr(e) = &ast {
            self.run_expr(e, vm);
        }
        if let FSRToken::Module(m) = &ast {
            self.get_cur_stack().push_local_block_vars();
            for token in &m.tokens {
                self.run_ast(token, vm);
            }
            self.get_cur_stack().pop_local_block_vars();
        }

        if let FSRToken::IfExp(if_block) = &ast {
            self.run_if_block(if_block, vm);
        }

        if let FSRToken::Block(b) = &ast {
            self.run_block(b, vm);
        }

        if let FSRToken::Assign(e) = &ast {
            self.run_assign(e, vm);
        }

        if let FSRToken::Call(c) = &ast {
            self.call_func(c, vm);
        }

        if let FSRToken::FunctionDef(fn_def) = &ast {
            self.define_function(fn_def, vm);
        }

        return Ok(());
    }

    pub fn run_code(
        &mut self,
        code: &'a [u8],
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<(), FSRRuntimeError> {
        let meta = FSRMeta::new();
        let code = FSRModule::parse(code, meta).unwrap();
        let m = FSRToken::Module(code);
        self.run_ast(&m, vm);
        return Ok(());
    }

    pub fn get_obj_by_name(
        &self,
        name: &'a str,
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Option<&'a FSRObject<'a>> {
        let id = self.find_symbol(name, vm, None).unwrap();
        return vm.get_obj_by_id(&id);
    }
}
