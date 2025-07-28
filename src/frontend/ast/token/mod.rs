use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use base::{FSRToken, FSRType};

use crate::frontend::ast::token::base::FSRPosition;

pub mod assign;
pub mod base;
pub mod block;
pub mod call;
pub mod class;
pub mod constant;
pub mod r#else;
pub mod expr;
pub mod for_statement;
pub mod function_def;
pub mod hashmap;
pub mod hashtable;
pub mod if_statement;
pub mod import;
pub mod list;
pub mod match_pattern;
pub mod module;
pub mod name;
pub mod return_def;
pub mod slice;
pub mod statement;
pub mod tell;
pub mod try_expr;
pub mod variable;
pub mod while_statement;

#[derive(Debug, Clone)]
pub struct ASTVariableState {
    pub(crate) is_defined: bool,
    pub(crate) token: Option<FSRToken>,
}

impl ASTVariableState {
    pub fn new(is_defined: bool, token: Option<FSRToken>) -> Self {
        Self { is_defined, token }
    }

    pub fn set_token(&mut self, token: Option<FSRToken>) {
        self.token = token;
    }
}

pub struct ASTContext {
    pub(crate) variable_define: Vec<Rc<RefCell<HashMap<String, ASTVariableState>>>>,
}

impl ASTContext {
    pub fn new_context() -> Self {
        Self {
            variable_define: vec![Rc::new(RefCell::new(HashMap::new()))],
        }
    }


    pub fn add_variable(&self, name: &str, token: Option<FSRToken>) {
        if let Some(s) = self.variable_define.last() {
            if let Some(s) = s.borrow_mut().get_mut(name) {
                // variable already defined, keep closure ref
                s.token = token;
                return;
            }
        }
        self.variable_define
            .last()
            .unwrap()
            .borrow_mut()
            .insert(name.to_string(), ASTVariableState::new(false, token));
    }

    pub fn set_variable_token(&self, name: &str, token: Option<FSRToken>) {
        self.variable_define
            .last()
            .unwrap()
            .borrow_mut()
            .get_mut(name)
            .map(|x| {
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
                return scope
                    .borrow()
                    .get(name)
                    .unwrap()
                    .token
                    .as_ref()
                    .and_then(|x| x.deduction_type(context));
            }
        }
        None
    }

    pub fn add_variable_prev_one(&self, name: &str, token: Option<FSRToken>) {
        if let Some(s) = self.variable_define.get(self.variable_define.len() - 2) {
            if s.borrow().contains_key(name) {
                // variable already defined, keep closure ref
                return;
            }
        }
        self.variable_define
            .get(self.variable_define.len() - 2)
            .unwrap()
            .borrow_mut()
            .insert(name.to_string(), ASTVariableState::new(false, token));
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
        self.variable_define
            .push(Rc::new(RefCell::new(HashMap::new())));
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
        self.variable_define
            .last()
            .unwrap()
            .borrow()
            .contains_key(name)
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

pub struct FSRSourceChar {
    char: u8,
    pub(crate) line: usize,
    pub(crate) column: usize
}

pub struct FSRSourceBytes {
    pub(crate) source: Vec<u8>,
    lines: Vec<usize>,
}

impl FSRSourceBytes {
    pub fn new(source: Vec<u8>) -> Self {
        let mut lines = vec![0];
        for (i, &byte) in source.iter().enumerate() {
            if byte == b'\n' {
                lines.push(i + 1);
            }
        }
        Self { source, lines }
    }

    pub fn get_char_at(&self, pos: usize) -> Option<FSRSourceChar> {
        if pos >= self.source.len() {
            return None;
        }
        let line = self.lines.iter().position(|&x| x > pos).unwrap_or(self.lines.len() - 1);
        let column = pos % line;
        Some(FSRSourceChar {
            char: self.source[pos],
            line,
            column
        })
    }
}