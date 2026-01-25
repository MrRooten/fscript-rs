#![allow(unused)]
use std::error::Error;
use std::fmt::Display;
use std::io;

use frontend::ast::token::base::FSRPosition;

use crate::backend::types::base::ObjId;
use crate::backend::vm::thread::CallFrame;

#[derive(Debug)]
pub enum SyntaxErrType {
    BracketNotMatch,
    OperatorError,
    QuoteNotClose,
    CommentError,
    NotMatchAttribute,
    None,
}

#[derive(Debug)]
pub struct SyntaxError {
    meta: FSRPosition,
    msg: String,
    err_type: SyntaxErrType,
}

impl SyntaxError {
    pub fn new<S>(meta: &FSRPosition, msg: S) -> Self
    where
        S: ToString,
    {
        Self {
            meta: meta.clone(),
            msg: msg.to_string(),
            err_type: SyntaxErrType::None,
        }
    }

    pub fn new_with_type<S>(meta: &FSRPosition, msg: S, t: SyntaxErrType) -> Self
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
        write!(f, "(offset: {}) {}", self.meta, self.msg)
    }
}

impl Error for SyntaxError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        self.msg.as_str()
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

pub struct RuntimeBaseError {}

#[derive(Debug, PartialEq)]
pub enum FSRErrCode {
    EmptyExpStack,
    NoSuchMethod,
    NoSuchObject,
    OutOfRange,
    NotValidArgs,
    NotSupportOperator,
    IndexOutOfRange,
    RuntimeError,
    FileError
}

#[derive(Debug)]
pub struct ErrorStruct {
    pub(crate) code: FSRErrCode,
    pub(crate) exception: Option<ObjId>,
    msg: String,
}

#[derive(Debug)]
pub struct FSRError {
    pub(crate) inner: Box<ErrorStruct>,
}

impl FSRError {
    pub fn new(msg: impl Into<String>, code: FSRErrCode) -> Self {
        Self {
            inner: Box::new(ErrorStruct {
                code,
                exception: None,
                msg: msg.into(),
            }),
        }
    }

    pub fn new_runtime_error(exception: ObjId) -> Self {
        Self {
            inner: Box::new(ErrorStruct {
                code: FSRErrCode::RuntimeError,
                exception: Some(exception),
                msg: "Runtime Error".to_string(),
            }),
        }
    }
}

impl Error for FSRError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        &self.inner.msg
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

impl Display for FSRError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(exception) = &self.inner.exception {
            write!(f, "FSR Error: {} (code: {:?}, exception: {})", self.inner.msg, self.inner.code, exception)
        } else {
            write!(f, "FSR Error: {} (code: {:?})", self.inner.msg, self.inner.code)
        }
    }
}

#[repr(C)]
pub enum FSRResult {
    Ok,
    Err
}

#[repr(C)]
pub struct FSRCResult {
    r_type: FSRResult,
    ok_value: ObjId,
    err_value: FSRError
}

impl From<io::Error> for FSRError {
    fn from(value: io::Error) -> Self {
        Self {
            inner: Box::new(ErrorStruct {
                code: FSRErrCode::FileError,
                exception: None,
                msg: value.to_string(),
            }),
        }
    }
}

impl From<anyhow::Error> for FSRError {
    fn from(value: anyhow::Error) -> Self {
        Self {
            inner: Box::new(ErrorStruct {
                code: FSRErrCode::RuntimeError,
                exception: None,
                msg: value.to_string(),
            }),
        }
    }
}