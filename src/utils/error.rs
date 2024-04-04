#![allow(unused)]
use std::error::Error;
use std::fmt::Display;

use crate::backend::vm::runtime::VMCallState;
use crate::frontend::ast::token::base::FSRMeta;

#[derive(Debug)]
pub enum SyntaxErrType {
    BracketNotMatch,
    OperatorError,
    QuoteNotClose,
    None,
}

#[derive(Debug)]
pub struct SyntaxError {
    meta: FSRMeta,
    msg: String,
    err_type: SyntaxErrType,
}

impl SyntaxError {
    pub fn new<S>(meta: &FSRMeta, msg: S) -> Self
    where
        S: ToString,
    {
        Self {
            meta: meta.clone(),
            msg: msg.to_string(),
            err_type: SyntaxErrType::None,
        }
    }

    pub fn new_with_type<S>(meta: &FSRMeta, msg: S, t: SyntaxErrType) -> Self
    where
        S: ToString,
    {
        Self {
            meta: meta.clone(),
            msg: msg.to_string(),
            err_type: t,
        }
    }
}

impl Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(offset: {}) {}", self.meta.offset, self.msg)
    }
}

impl Error for SyntaxError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        return self.msg.as_str();
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

pub struct RuntimeBaseError {}

#[derive(Debug, PartialEq)]
pub enum FSRRuntimeType {
    NotFoundSymbolInScope,
    TokenNotMatch,
    NoSuchMethod,
    OperatorError,
    NotFoundObject,
    NotValidAttr,
    TypeNotMatch
}

#[derive(Debug)]
pub struct FSRRuntimeError<'a> {
    msg: String,
    e_type: FSRRuntimeType,
    stack: &'a Vec<VMCallState<'a>>,
    meta: FSRMeta
}

impl Display for FSRRuntimeError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = vec![];
        for i in self.stack {
            s.push(i.get_string());
        }
        let s = s[0..].join("-");
        let exp = match self.e_type {
            FSRRuntimeType::NotFoundSymbolInScope => "Not Found Symbol In Scope",
            FSRRuntimeType::TokenNotMatch => "Token Not Match",
            FSRRuntimeType::NoSuchMethod => "No Such Method",
            FSRRuntimeType::OperatorError => "Operator Error",
            FSRRuntimeType::NotFoundObject => "Not Found Object",
            FSRRuntimeType::NotValidAttr => "Not a valid attr",
            FSRRuntimeType::TypeNotMatch => "Type not match",
        };
        writeln!(f, "{}: {}", s, exp)
    }
}

impl Error for FSRRuntimeError<'_> {}

impl<'a> FSRRuntimeError<'a> {
    pub fn new<S>(
        stack: &'a Vec<VMCallState<'a>>,
        err: FSRRuntimeType,
        msg: S,
        meta: &FSRMeta
    ) -> FSRRuntimeError<'a>
    where
        S: ToString,
    {
        Self {
            msg: msg.to_string(),
            e_type: err,
            stack,
            meta: meta.clone()
        }
    }
}
