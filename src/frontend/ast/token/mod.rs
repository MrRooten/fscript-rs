use std::{cell::RefCell, collections::{HashMap, HashSet}, rc::Rc};

use base::{FSRToken, FSRType};

pub mod if_statement;
pub mod statement;
pub mod function_def;
pub mod constant;
pub mod base;
pub mod name;
pub mod call;
pub mod assign;
pub mod expr;
pub mod hashtable;
pub mod variable;
pub mod block;
pub mod return_def;
pub mod slice;
pub mod while_statement;
pub mod import;
pub mod module;
pub mod list;
pub mod hashmap;
pub mod class;
pub mod r#else;
pub mod for_statement;
pub mod try_expr;
pub mod match_pattern;

#[derive(Debug, Clone)]
pub struct ASTVariableState {
    pub(crate) is_defined: bool,
    pub(crate) token: Option<FSRToken>,
}

impl ASTVariableState {
    pub fn new(is_defined: bool, token: Option<FSRToken>) -> Self {
        Self {
            is_defined,
            token,
        }
    }

    pub fn set_token(&mut self, token: Option<FSRToken>) {
        self.token = token;
    }
}

pub struct ASTContext {
    pub(crate)  variable_define: Vec<Rc<RefCell<HashMap<String, ASTVariableState>>>>
}

impl ASTContext {
    pub fn new_context() -> Self {
        Self {
            variable_define: vec![Rc::new(RefCell::new(HashMap::new()))]
        }
    }

    pub fn add_variable(&self, name: &str, token: Option<FSRToken>) {
        if let Some(s) = self.variable_define.last() {
            if let Some(s) = s.borrow_mut().get_mut(name) {
                // variable already defined, keep closure ref
                s.token = token;
                return ;
            }
        }
        self.variable_define.last().unwrap().borrow_mut().insert(name.to_string(), ASTVariableState::new(false, token));
    }


    pub fn set_variable_token(&self, name: &str, token: Option<FSRToken>) {
        self.variable_define.last().unwrap().borrow_mut().get_mut(name).map(|x| {
            x.set_token(token);
        });
    }

    pub fn get_token(&self, name: &str) -> Option<FSRToken> {
        for scope in self.variable_define.iter().rev() {
            if scope.borrow().contains_key(name) {
                return scope.borrow().get(name).unwrap().token.clone();
            }
        }
        None
    }

    pub fn get_token_var_type(&self, name: &str, context: &ASTContext) -> Option<FSRType> {
        for scope in self.variable_define.iter().rev() {
            if scope.borrow().contains_key(name) {
                return scope.borrow().get(name).unwrap().token.as_ref().and_then(|x| x.deduction_type(context));
            }
        }
        None
    }

    pub fn add_variable_prev_one(&self, name: &str, token: Option<FSRToken>) {
        if let Some(s) = self.variable_define.get(self.variable_define.len() - 2) {
            if s.borrow().contains_key(name) {
                // variable already defined, keep closure ref
                return ;
            }
        }
        self.variable_define.get(self.variable_define.len() - 2).unwrap().borrow_mut().insert(name.to_string(), ASTVariableState::new(false, token));
    }

    pub fn ref_variable(&self, name: &str) {
        for scope in self.variable_define.iter().rev() {
            if scope.borrow().contains_key(name) {
                scope.borrow_mut().get_mut(name).map(|x| {
                    x.is_defined = true;
                });
                return;
            }
        }
    }

    pub fn push_scope(&mut self) {
        self.variable_define.push(Rc::new(RefCell::new(HashMap::new())));
    }

    pub fn pop_scope(&mut self) -> Rc<RefCell<HashMap<String, ASTVariableState>>> {
        self.variable_define.pop().unwrap()
    }

    pub fn is_variable_defined(&self, name: &str) -> bool {
        for scope in self.variable_define.iter().rev() {
            if scope.borrow().contains_key(name) {
                return true;
            }
        }
        false
    }

    pub fn is_variable_defined_in_curr(&self, name: &str) -> bool {
        self.variable_define.last().unwrap().borrow().contains_key(name)
    }

    pub fn set_variable_be_ref(&mut self, name: &str) -> Option<()> {
        for scope in self.variable_define.iter_mut().rev() {
            if scope.borrow().contains_key(name) {
                // scope.borrow_mut().insert(name.to_string(), ASTVariableState::new(true));
                scope.borrow_mut().get_mut(name).map(|x| {
                    x.is_defined = true;
                });
                return Some(());
            }
        }
        None
    }

}