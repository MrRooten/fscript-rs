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

use super::thread::FSRThread;

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
    threads: HashMap<u64, FSRThread<'a>>,
    meta: FSRMeta,
    is_ret: bool,
    ret_value: Option<u64>,
}

pub enum FSRArg<'a> {
    String(&'a str),
    Expr(&'a FSRToken<'a>),
    Assign(&'a FSRAssign<'a>),
}

impl<'a> FSRThreadRuntime<'a> {
    pub fn get_cur_token(&self) -> &Option<*const FSRToken<'a>> {
        let cur = self.get_cur_stack();
        return cur.get_cur_token();
    }

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

    fn get_cur_stack(&self) -> &VMCallState<'a> {
        let len = self.call_stack.len();
        return self.call_stack.get(len - 1).unwrap();
    }

    fn get_mut_cur_stack(&mut self) -> &mut VMCallState<'a> {
        let len = self.call_stack.len();
        return self.call_stack.get_mut(len - 1).unwrap();
    }

    fn get_stack(&mut self, id: usize) -> &mut VMCallState<'a> {
        return self.call_stack.get_mut(id).unwrap();
    }

    fn get_stack_len(&self) -> usize {
        return self.call_stack.len();
    }

    pub fn new() -> FSRThreadRuntime<'a> {
        let mut module = Self {
            call_stack: vec![VMCallState::new("base")],
            meta: FSRMeta::new(),
            modules: HashMap::new(),
            threads: HashMap::new(),
            is_ret: false,
            ret_value: None,
        };
        return module;
    }

    pub fn init(&mut self, vars: &HashMap<&'a str, u64>, vm: &'a FSRVirtualMachine<'a>) {
        for kv in vars {
            self.assign_module_variable(kv.0, *kv.1, vm);
        }
    }

    pub fn init_with_vm(&mut self, vm: &'a FSRVirtualMachine<'a>) {
        let module = register_path(vm);
        self.modules.insert(module.get_name().to_string(), module);
        let self_module = FSRModule::self_module();
        self.modules
            .insert(self_module.get_name().to_string(), self_module);
    }

    pub fn find_symbol(
        &self,
        name: &str,
        vm: &'a FSRVirtualMachine<'a>,
        stack_id: Option<usize>,
    ) -> Result<u64, FSRRuntimeError> {
        let f = name.find("::");
        if let Some(s) = f {
            let module_name = &name[0..s];
            let variable = &name[s + 2..];
            if let Some(module) = self.modules.get(module_name) {
                let v = match module.colon_operator(variable) {
                    Some(s) => s,
                    None => {
                        let err = FSRRuntimeError::new(
                            &self.call_stack,
                            FSRRuntimeType::NotFoundObject,
                            format!("not found object in module: {}: {}", variable, module_name),
                            self.get_cur_meta(),
                        );
                        return Err(err);
                    }
                };

                return Ok(v);
            }
            let cls_name = module_name;
            let cls_id = self.find_symbol(cls_name, vm, None)?;

            let cls_obj = vm.get_obj_by_id(&cls_id).unwrap();
            if let FSRValue::Class(c) = cls_obj.get_value() {
                let v = match c.get_cls_attr(variable) {
                    Some(s) => s,
                    None => {
                        let err = FSRRuntimeError::new(
                            &self.call_stack,
                            FSRRuntimeType::NotFoundObject,
                            format!("not found object in class: {}: {}", variable, cls_name),
                            self.get_cur_meta(),
                        );
                        return Err(err);
                    }
                };

                return Ok(v.clone());
            }

            let err = FSRRuntimeError::new(
                &self.call_stack,
                FSRRuntimeType::TypeNotMatch,
                format!("{} is not a class", cls_name),
                self.get_cur_meta(),
            );
            return Err(err);
        }
        if let Some(id) = stack_id {
            for local_var in i_to_m(self).get_stack(id).local_vars.iter().rev() {
                if let Some(id) = local_var.get_var(name) {
                    return Ok(id);
                }
            }

            if let Some(id) = self.get_module_variable(name) {
                return Ok(id.clone());
            }

            if let Some(id) = vm.get_global_by_name(name) {
                return Ok(id.clone());
            }

            let err = FSRRuntimeError::new(
                &self.call_stack,
                FSRRuntimeType::NotFoundObject,
                format!("not found object: {}", name),
                self.get_cur_meta(),
            );
            return Err(err);
        }
        for local_var in i_to_m(self).get_mut_cur_stack().local_vars.iter().rev() {
            if let Some(id) = local_var.get_var(name) {
                return Ok(id);
            }
        }
        if let Some(id) = vm.get_global_by_name(name) {
            return Ok(id.clone());
        }

        let module_name = "Self";
        let module = self.modules.get(module_name).unwrap();

        if let Some(id) = module.colon_operator(name) {
            return Ok(id.clone());
        }

        let err = FSRRuntimeError::new(
            &self.call_stack,
            FSRRuntimeType::NotFoundObject,
            format!("not found object: {}", name),
            self.get_cur_meta(),
        );
        return Err(err);
    }

    fn invoke_binary_op(
        &mut self,
        left: &'a FSRObject<'a>,
        op: &str,
        right: &'a FSRObject<'a>,
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<u64, FSRRuntimeError> {
        i_to_m(self).assign_variable(FSRArg::String("other"), right.get_id(), vm)?;
        let mut v = 0;
        if op.eq("+") {
            v = left.invoke_method("__add__", vm, i_to_m(self))?;
        } else if op.eq("-") {
            v = left.invoke_method("__sub__", vm, i_to_m(self))?;
        } else if op.eq("*") {
            v = left.invoke_method("__mul__", vm, i_to_m(self))?;
        } else if op.eq("/") {
            v = left.invoke_method("__div__", vm, i_to_m(self))?;
        } else if op.eq("==") {
            v = left.invoke_method("__eq__", vm, i_to_m(self))?;
        } else if op.eq("!=") {
            v = left.invoke_method("__not_eq__", vm, i_to_m(self))?;
        } else if op.eq("<<") {
            v = left.invoke_method("__left_shift__", vm, i_to_m(self))?;
        } else if op.eq(">>") {
            v = left.invoke_method("__right_shift__", vm, i_to_m(self))?;
        } else if op.eq(">") {
            v = left.invoke_method("__gt__", vm, i_to_m(self))?;
        } else if op.eq("<") {
            v = left.invoke_method("__lt__", vm, i_to_m(self))?;
        } else if op.eq(">=") {
            v = left.invoke_method("__gte__", vm, i_to_m(self))?;
        } else if op.eq("<=") {
            v = left.invoke_method("__lte__", vm, i_to_m(self))?;
        } else if op.eq("+=") {
            v = left.invoke_method("__self_add__", vm, i_to_m(self))?;
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

    fn new_object(
        &self,
        cls: &'a FSRClassBackEnd<'a>,
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<u64, FSRRuntimeError> {
        let mut inst_attrs = HashMap::new();
        for attr in cls.get_attrs() {
            let v = self.run_token(attr.1, vm, None)?;
            inst_attrs.insert(*attr.0, v);
        }

        let inst = FSRClassInstance {
            attrs: inst_attrs,
            cls: cls,
        };

        

        let inst = FSRClassInstance::from_inst(inst, vm)?;


        return Ok(inst);
    }

    fn call_func(
        &self,
        c: &FSRCall,
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<u64, FSRRuntimeError> {
        let fn_id = self.find_symbol(&c.get_name(), vm, None).unwrap();
        let fn_obj = vm.get_obj_by_id(&fn_id).unwrap();

        if let FSRValue::Class(cls) = fn_obj.get_value() {
            let f = match cls.get_cls_attr("__new__") {
                Some(s) => s,
                None => {
                    return Ok(vm.get_none_id());
                }
            };
            let fn_obj = vm.get_obj_by_id(&f).unwrap();
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
            let mut new_args = vec![];
            for a in args {
                new_args.push(a.to_string())
            }
            let target_args = c.get_args();
            let mut i = 0;
            let mut fn_args = vec![];

            i_to_m(self).push_call_stack(c.get_name());
            let mut start = true;
            while i < target_args.len() {
                let mut v = 0;
                v = self.run_token(&target_args[i], vm, None).unwrap();
                fn_args.push((&new_args[i+1], v));
                i += 1;

            }

            for a in fn_args {
                i_to_m(self).assign_variable(FSRArg::String(&a.0), a.1, vm);
            }

            let cls_obj_id = self.new_object(cls, vm)?;
            let cls_obj = vm.get_obj_by_id(&cls_obj_id).unwrap();
            if cls_obj.has_method("__new__", vm) {
                cls_obj.invoke_method("__new__", vm, self);
            }
            i_to_m(self).pop_call_stack();

            return Ok(cls_obj_id);
        }

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
            i_to_m(self).assign_variable(FSRArg::String(a.0), a.1, vm)?;
        }
        let v = fn_obj.invoke(vm, i_to_m(self))?;

        i_to_m(self).pop_call_stack();
        return Ok(v);
    }

    pub fn assign_variable(
        &mut self,
        left_value: FSRArg<'a>,
        id: u64,
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<(), FSRRuntimeError> {
        let obj = i_to_m(vm).get_mut_obj_by_id(&id).unwrap();
        obj.ref_object();
        if let FSRArg::Expr(left_value) = left_value {
            if let FSRToken::Variable(v) = left_value {
                // Process ::, like os::evn = []
                let name = v.get_name();
                let f = name.find("::");
                if let Some(s) = f {
                    let module_name = &name[0..s];
                    let variable = &name[s + 2..];
                    if let Some(module) = self.modules.get_mut(module_name) {
                        module.set_colon_operator(variable, id);

                        return Ok(());
                    }
                    let cls_name = module_name;
                    let cls_id = self.find_symbol(cls_name, vm, None)?;

                    let cls_obj = i_to_m(vm).get_mut_obj_by_id(&cls_id).unwrap();
                    if let FSRValue::Class(c) = cls_obj.get_mut_value() {
                        c.set_cls_attr(variable, id);

                        return Ok(());
                    }

                    let err = FSRRuntimeError::new(
                        &self.call_stack,
                        FSRRuntimeType::TypeNotMatch,
                        format!("{} is not a class", cls_name),
                        self.get_cur_meta(),
                    );
                    return Err(err);
                }

                let l = self.get_mut_cur_stack().local_vars.len() - 1;
                let cur_stack = self.get_mut_cur_stack();
                let local_vars = cur_stack.get_local_vars();
                let mut found = false;
                for local in local_vars.iter_mut().rev() {
                    if local.get_var(name).is_some() {
                        local.local_vars.insert(name, obj.get_id());
                        found = true;
                    }
                }

                if found == false {
                    local_vars
                        .get_mut(l)
                        .unwrap()
                        .local_vars
                        .insert(name, obj.get_id());
                }

                return Ok(());
            }

            if let FSRToken::Expr(expr) = left_value {
                if expr.get_op().eq(".") {
                    let l_obj = self.run_token(expr.get_left(), vm, None)?;
                    let r_name = match &**expr.get_right() {
                        FSRToken::Variable(v) => v.get_name(),
                        _ => {
                            unimplemented!()
                        }
                    };

                    let obj = i_to_m(vm).get_mut_obj_by_id(&l_obj).unwrap();
                    obj.set_attr(r_name, id);
                    return Ok(());
                }

                unimplemented!()
            }
        }

        if let FSRArg::String(name) = left_value {
            // Process ::, like os::evn = []
            let f = name.find("::");
            if let Some(s) = f {
                let module_name = &name[0..s];
                let variable = &name[s + 2..];
                if let Some(module) = self.modules.get_mut(module_name) {
                    module.set_colon_operator(variable, id);

                    return Ok(());
                }
                let cls_name = module_name;
                let cls_id = self.find_symbol(cls_name, vm, None)?;

                let cls_obj = i_to_m(vm).get_mut_obj_by_id(&cls_id).unwrap();
                if let FSRValue::Class(c) = cls_obj.get_mut_value() {
                    c.set_cls_attr(variable, id);

                    return Ok(());
                }

                let err = FSRRuntimeError::new(
                    &self.call_stack,
                    FSRRuntimeType::TypeNotMatch,
                    format!("{} is not a class", cls_name),
                    self.get_cur_meta(),
                );
                return Err(err);
            }

            let l = self.get_mut_cur_stack().local_vars.len() - 1;
            let cur_stack = self.get_mut_cur_stack();
            let local_vars = cur_stack.get_local_vars();
            let mut found = false;
            for local in local_vars.iter_mut().rev() {
                if local.get_var(name).is_some() {
                    local.local_vars.insert(name, obj.get_id());
                    found = true;
                }
            }

            if found == false {
                local_vars
                    .get_mut(l)
                    .unwrap()
                    .local_vars
                    .insert(name, obj.get_id());
            }

            return Ok(());
        }

        unimplemented!()
    }

    pub fn assign_module_variable(
        &mut self,
        name: &'a str,
        id: u64,
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<(), &str> {
        let obj = i_to_m(vm).get_mut_obj_by_id(&id).unwrap();
        obj.ref_object();
        let self_module = self.modules.get_mut("Self").unwrap();
        self_module.register_obj(name, id);
        return Ok(());
    }

    pub fn get_module_variable(&self, name: &'a str) -> Option<&u64> {
        let self_module = self.modules.get("Self").unwrap();
        return self_module.get_obj(name);
    }

    fn run_list(
        &self,
        e: &'a FSRListFrontEnd<'a>,
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<u64, FSRRuntimeError> {
        let items = e.get_items();
        let mut vs = vec![];
        for t in items {
            let v = self.run_token(t, vm, None)?;
            vs.push(v);
        }

        let v = FSRList::from_list(vs, vm)?;
        return Ok(v);
    }

    fn run_assign(
        &self,
        e: &'a FSRAssign<'a>,
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<(), FSRRuntimeError> {
        let right = e.get_assign_expr();
        i_to_m(self).get_mut_cur_stack().set_cur_token(&*right);
        if let FSRToken::Expr(ex) = &**right {
            let v = self.run_expr(ex, vm)?;
            let cur_stack = i_to_m(self).get_mut_cur_stack();
            let local_vars = cur_stack.get_local_vars();
            for local_var in local_vars.iter_mut().rev() {
                if local_var.local_vars.get(e.get_name()).is_none() == false {
                    local_var.local_vars.insert(e.get_name(), v).unwrap();
                    return Ok(());
                }
            }

            if self.get_module_variable(e.get_name()).is_none() == false {
                i_to_m(self).assign_module_variable(e.get_name(), v, vm);
            }

            i_to_m(self).assign_variable(FSRArg::Expr(e.get_left()), v, vm)?;
            return Ok(());
        } else if let FSRToken::Constant(c) = &**right {
            let v = i_to_m(self).register_constant(c, vm)?;

            for local_var in i_to_m(self).get_mut_cur_stack().local_vars.iter_mut().rev() {
                if local_var.local_vars.get(e.get_name()).is_none() == false {
                    local_var.local_vars.insert(e.get_name(), v).unwrap();
                    return Ok(());
                }
            }

            if self.get_module_variable(e.get_name()).is_none() == false {
                i_to_m(self).assign_module_variable(e.get_name(), v, vm);
            }

            i_to_m(self).assign_variable(FSRArg::Expr(e.get_left()), v, vm)?;
            return Ok(());
        } else if let FSRToken::Variable(v) = &**right {
            let v = self.find_symbol(v.get_name(), vm, None)?;
            let obj = i_to_m(vm).get_mut_obj_by_id(&v).unwrap();

            for local_var in i_to_m(self).get_mut_cur_stack().local_vars.iter_mut().rev() {
                if local_var.local_vars.get(e.get_name()).is_none() == false {
                    local_var.local_vars.insert(e.get_name(), v).unwrap();
                    return Ok(());
                }
            }

            if self.get_module_variable(e.get_name()).is_none() == false {
                i_to_m(self).assign_module_variable(e.get_name(), v, vm);
            }

            i_to_m(self).assign_variable(FSRArg::Expr(e.get_left()), obj.get_id(), vm)?;
            return Ok(());
        } else if let FSRToken::List(l) = &**right {
            let v = self.run_list(l, vm)?;
            i_to_m(self).assign_variable(FSRArg::Expr(e.get_left()), v, vm)?;
            return Ok(());
        } else if let FSRToken::Call(c) = &**right {
            let v = self.call_func(c, vm)?;
            i_to_m(self).assign_variable(FSRArg::Expr(e.get_left()), v, vm)?;
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
            i_to_m(self)
                .get_mut_cur_stack()
                .set_cur_token(&*e.get_left());
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
            i_to_m(self)
                .get_mut_cur_stack()
                .set_cur_token(&*e.get_right());
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
                
                let fn_id = l_obj.get_attr(name, vm).unwrap();
                let fn_obj = vm.get_obj_by_id(fn_id).unwrap();
                if fn_obj.is_function() == false {
                    unimplemented!()
                }

                let fn_obj_1 = fn_obj.get_function().unwrap();
                i_to_m(self).push_call_stack(name);
                i_to_m(self).assign_variable(FSRArg::String("self"), l_obj.get_id(), vm);
                let args = fn_obj_1.get_args();
                let mut new_args = vec![];
                for a in args {
                    new_args.push(a.to_string());
                }
                let target_args = call.get_args();
                let mut i = 0;
                let mut fn_args = vec![];
                
                while i < target_args.len() {
                    let mut v = 0;
                    v = self.run_token(&target_args[i], vm, None).unwrap();
                    fn_args.push((&new_args[i+1], v));
                    i += 1;
                }
                
                for a in fn_args {
                    i_to_m(self).assign_variable(FSRArg::String(&a.0), a.1, vm);
                }
                
                let ret = l_obj.invoke_method(name, vm, i_to_m(self))?;
                i_to_m(self).pop_call_stack();
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
        i_to_m(self)
            .get_mut_cur_stack()
            .set_cur_token(&*e.get_left());
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

        i_to_m(self)
            .get_mut_cur_stack()
            .set_cur_token(&*e.get_right());
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
        self.get_mut_cur_stack().push_local_block_vars();
        let mut v = 0;
        for token in block.get_tokens() {
            self.get_mut_cur_stack().set_cur_token(token);
            v = match self.run_token(token, vm, None) {
                Ok(o) => o,
                Err(e) => return Err("run error"),
            };
            if self.is_ret == true {
                break;
            }
        }
        self.get_mut_cur_stack().pop_local_block_vars();
        return Ok(v);
    }

    pub fn run_if_block(
        &self,
        if_expr: &'a FSRIf<'a>,
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<(), FSRRuntimeError> {
        let expr = if_expr.get_test();
        i_to_m(self).get_mut_cur_stack().set_cur_token(&*expr);
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
                if_expr.get_meta(),
            );
            return Err(err);
        }

        let v = match l_value {
            Some(s) => s,
            None => {
                let err = FSRRuntimeError::new(
                    &self.call_stack,
                    FSRRuntimeType::TokenNotMatch,
                    format!("Not match token: {:?}", expr),
                    if_expr.get_meta(),
                );
                return Err(err);
            }
        };

        if v != vm.get_false_id() && v != vm.get_none_id() {
            let block = &**if_expr.get_block();
            i_to_m(self).run_block(block, vm);
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
            l_value = Some(self.run_expr(l, vm).unwrap());
        } else if let FSRToken::Constant(c) = &*token {
            l_value = Some(i_to_m(self).register_constant(c, vm).unwrap());
        } else if let FSRToken::Call(c) = &*token {
            l_value = Some(i_to_m(self).call_func(c, vm).unwrap());
        } else if let FSRToken::Variable(v) = &*token {
            l_value = Some(self.find_symbol(v.get_name(), vm, stack_id).unwrap());
        } else if let FSRToken::Assign(a) = &*token {
            self.run_assign(a, vm);
        } else if let FSRToken::IfExp(if_def) = &*token {
            self.run_if_block(if_def, vm);
        } else if let FSRToken::WhileExp(while_exp) = &*token {
            self.run_while_block(while_exp, vm);
        } else if let FSRToken::Return(r) = &*token {
            let ret = r.get_return_expr();
            let v = self.run_token(&**ret, vm, None)?;
            i_to_m(self).is_ret = true;
            i_to_m(self).ret_value = Some(v);
        } else {
            let err = FSRRuntimeError::new(
                &self.call_stack,
                FSRRuntimeType::TokenNotMatch,
                format!("Not match token: {:?}", token),
                token.get_meta(),
            );
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

    pub fn run_while_block(
        &self,
        for_expr: &'a FSRWhile<'a>,
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<(), FSRRuntimeError> {
        let expr = for_expr.get_test();
        let mut v = self.run_token(&**expr, vm, None)?;

        while v != vm.get_false_id() && v != vm.get_none_id() {
            i_to_m(self).run_block(&**for_expr.get_block(), vm);
            v = self.run_token(&**expr, vm, None)?;
            if self.is_ret == true {
                break;
            }
        }

        return Ok(());
    }

    pub fn run_return(
        &self,
        expr: &'a FSRToken<'a>,
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<u64, FSRRuntimeError> {
        let v = self.run_token(expr, vm, None)?;
        return Ok(v);
    }

    pub fn define_function(
        &mut self,
        fn_def: &FSRFnDef<'a>,
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<u64, &str> {
        let fn_obj = FSRFn::from_ast(&fn_def, vm);

        let fn_id = fn_obj.get_id();
        self.assign_variable(FSRArg::String(fn_def.get_name()), fn_id, vm);
        return Ok(fn_id);
    }

    pub fn define_class(
        &'a self,
        cls_def: &FSRClassFrontEnd<'a>,
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<u64, FSRRuntimeError> {
        let cls_id = FSRClassBackEnd::from_cls(cls_def, self, vm).unwrap();
        i_to_m(self).assign_variable(FSRArg::String(cls_def.get_name()), cls_id, vm)?;
        return Ok(cls_id);
    }

    pub fn run_ast_fn(
        &self,
        fn_def: &FSRFnDef<'a>,
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<u64, &str> {
        let mut v = 0;
        let body = fn_def.get_body();
        i_to_m(self).get_mut_cur_stack().push_local_block_vars();
        let mut v = 0;
        for token in body.get_tokens() {
            i_to_m(self).get_mut_cur_stack().set_cur_token(token);
            v = match self.run_token(token, vm, None) {
                Ok(o) => o,
                Err(e) => return Err("run error"),
            };
            if self.is_ret == true {
                i_to_m(self).is_ret = false;
                v = match self.ret_value {
                    Some(s) => s,
                    None => 0,
                };
                i_to_m(self).ret_value = None;
                break;
            }
        }
        i_to_m(self).get_mut_cur_stack().pop_local_block_vars();
        return Ok(v);
    }

    pub fn run_ast(
        &mut self,
        ast: &FSRToken<'a>,
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<(), FSRRuntimeError> {
        self.get_mut_cur_stack().set_cur_token(ast);
        if let FSRToken::Expr(e) = &ast {
            self.run_expr(e, vm);
        }
        if let FSRToken::Module(m) = &ast {
            for token in &m.tokens {
                self.run_ast(token, vm);
            }
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

        if let FSRToken::WhileExp(for_expr) = &ast {
            self.run_while_block(for_expr, vm);
        }

        if let FSRToken::Class(class_def) = &ast {
            self.define_class(class_def, vm)?;
        }

        return Ok(());
    }

    pub fn run_code(
        &mut self,
        code: &'a [u8],
        vm: &'a FSRVirtualMachine<'a>,
    ) -> Result<(), FSRRuntimeError> {
        let meta = FSRMeta::new();
        let code = FSRModuleFrontEnd::parse(code, meta).unwrap();
        self.init_with_vm(vm);
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
