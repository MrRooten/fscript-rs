use std::collections::{HashMap, HashSet};

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


pub struct ASTContext {
    pub(crate)  variable_define: Vec<HashMap<String, bool>>
}

impl ASTContext {
    pub fn new() -> Self {
        Self {
            variable_define: vec![HashMap::new()]
        }
    }

    pub fn add_variable(&mut self, name: &str) {
        self.variable_define.last_mut().unwrap().insert(name.to_string(), false);
    }

    pub fn push_scope(&mut self) {
        self.variable_define.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) -> HashMap<String, bool> {
        self.variable_define.pop().unwrap()
    }

    pub fn is_variable_defined(&self, name: &str) -> bool {
        for scope in self.variable_define.iter().rev() {
            if scope.contains_key(name) {
                return true;
            }
        }
        false
    }

    pub fn set_variable_be_ref(&mut self, name: &str) -> Option<()> {
        for scope in self.variable_define.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), true);
                return Some(());
            }
        }
        None
    }

}