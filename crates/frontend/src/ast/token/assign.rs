#![allow(unused)]

use std::rc::Rc;


use super::{base::{FSRPosition, FSRToken}, ASTContext};

#[derive(Debug, Clone)]
pub struct FSRAssign {
    pub(crate) expr: Rc<FSRToken>,
    pub(crate) left: Rc<FSRToken>,
    pub(crate) name: String,
    pub(crate) len: usize,
    pub(crate) meta: FSRPosition,
    pub(crate) op_assign: String,
}

#[derive(PartialEq)]
enum FSRAssignState {
    Start,
    IdKeyLetStart,
    IdKeyLetEnd,
    NameStart,
    NameEnd,
    LeftValue,
    RightValue,
}

impl FSRAssign {
    pub fn get_left(&self) -> &Rc<FSRToken> {
        &self.left
    }

    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_assign_expr(&self) -> &Rc<FSRToken> {
        &self.expr
    }

    pub fn get_len(&self) -> usize {
        self.len
    }
}
