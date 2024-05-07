#![allow(unused)]
use std::error::Error;
use std::fmt::Display;

use crate::backend::vm::thread::CallState;
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

#[derive(Debug)]
pub enum FSRErrCode {
    EmptyExpStack,
    NoSuchMethod,
}

#[derive(Debug)]
pub struct FSRError {
    code: FSRErrCode,
    msg: String,
}

impl FSRError {
    pub fn new<S>(msg: S, code: FSRErrCode) -> Self
    where
        S: ToString,
    {
        Self {
            code,
            msg: msg.to_string(),
        }
    }
}

impl Error for FSRError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        &self.msg
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

impl Display for FSRError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
