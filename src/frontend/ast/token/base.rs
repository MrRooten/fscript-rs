use std::fmt::Display;

use crate::frontend::ast::token::block::FSRBlock;
use crate::frontend::ast::token::module::FSRModuleFrontEnd;
use crate::frontend::ast::token::slice::FSRSlice;

use super::{assign::FSRAssign, call::FSRCall, class::FSRClassFrontEnd, constant::FSRConstant, expr::FSRExpr, function_def::FSRFnDef, hashtable::FSRHashtable, if_statement::{FSRIf, FSRIfState}, import::FSRImport, list::FSRListFrontEnd, name::FSRName, return_def::FSRReturn, variable::FSRVariable, while_statement::FSRWhile};

#[derive(Debug, Clone)]
pub enum FSRToken<'a> {
    FunctionDef(FSRFnDef<'a>),
    IfExp(FSRIf<'a>),
    Constant(FSRConstant),
    Assign(FSRAssign<'a>),
    Expr(FSRExpr<'a>),
    Call(FSRCall<'a>),
    Variable(FSRVariable<'a>),
    Return(FSRReturn<'a>),
    Block(FSRBlock<'a>),
    WhileExp(FSRWhile<'a>),
    Module(FSRModuleFrontEnd<'a>),
    Import(FSRImport),
    List(FSRListFrontEnd<'a>),
    Class(FSRClassFrontEnd<'a>),
    EmptyExpr,
    None
}

impl<'a> FSRToken<'a> {
    pub fn get_meta(&self) -> &FSRMeta {
        match self {
            FSRToken::FunctionDef(e) => e.get_meta(),
            FSRToken::IfExp(e) => e.get_meta(),
            FSRToken::Constant(e) => e.get_meta(),
            FSRToken::Assign(e) => e.get_meta(),
            FSRToken::Expr(e) => e.get_meta(),
            FSRToken::Call(e) => e.get_meta(),
            FSRToken::Variable(e) => e.get_meta(),
            FSRToken::Return(e) => e.get_meta(),
            FSRToken::Block(e) => e.get_meta(),
            FSRToken::WhileExp(e) => e.get_meta(),
            FSRToken::Module(e) => e.get_meta(),
            FSRToken::Import(e) => e.get_meta(),
            FSRToken::EmptyExpr => todo!(),
            FSRToken::None => todo!(),
            FSRToken::List(e) => e.get_meta(),
            FSRToken::Class(e) => e.get_meta(),
        }
    }
}

pub enum FSRTokenState {
    If(FSRIfState),
}

pub trait FSRTokenMatcher {
    fn match_token() -> bool;
}

#[derive(Clone, Debug)]
pub struct FSRMeta {
    pub(crate) offset: usize,
}

impl FSRMeta {
    pub fn new() -> Self {
        return Self {
            offset: 0
        }
    }
}

impl Display for FSRMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.offset)
    }
}