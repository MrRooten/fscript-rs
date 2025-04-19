use std::fmt::Display;

use crate::frontend::ast::token::module::FSRModuleFrontEnd;
use crate::{frontend::ast::token::block::FSRBlock, utils::error::SyntaxError};

use super::try_expr::FSRTryBlock;
use super::{
    assign::FSRAssign, call::FSRCall, class::FSRClassFrontEnd, constant::FSRConstant,
    expr::FSRExpr, for_statement::FSRFor, function_def::FSRFnDef, if_statement::FSRIf,
    import::FSRImport, list::FSRListFrontEnd, return_def::FSRReturn, slice::FSRGetter,
    variable::FSRVariable, while_statement::FSRWhile,
};

#[derive(Debug, Clone)]
pub enum FSRToken<'a> {
    FunctionDef(FSRFnDef<'a>),
    IfExp(FSRIf<'a>),
    Constant(FSRConstant<'a>),
    Assign(FSRAssign<'a>),
    Break(FSRPosition),
    Continue(FSRPosition),
    Expr(FSRExpr<'a>),
    StackExpr((Option<&'a str>, Vec<FSRToken<'a>>)),
    ForBlock(FSRFor<'a>),
    Call(FSRCall<'a>),
    Variable(FSRVariable<'a>),
    Return(FSRReturn<'a>),
    Block(FSRBlock<'a>),
    WhileExp(FSRWhile<'a>),
    Module(FSRModuleFrontEnd<'a>),
    Import(FSRImport<'a>),
    List(FSRListFrontEnd<'a>),
    Class(FSRClassFrontEnd<'a>),
    Getter(FSRGetter<'a>),
    TryBlock(FSRTryBlock<'a>),
    EmptyExpr,
    None,
}

impl<'a> FSRToken<'a> {
    pub fn set_single_op(&mut self, op: &'a str) {
        match self {
            FSRToken::Expr(e) => e.single_op = Some(op),
            FSRToken::StackExpr(e) => e.0 = Some(op),
            FSRToken::Call(e) => e.single_op = Some(op),
            FSRToken::Getter(e) => e.single_op = Some(op),
            FSRToken::Variable(e) => e.single_op = Some(op),
            FSRToken::Constant(e) => e.single_op = Some(op),
            _ => panic!("Not an expression"),
        }

    }

    pub fn get_meta(&self) -> &FSRPosition {
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
            FSRToken::Break(e) => e,
            FSRToken::Continue(e) => e,
            FSRToken::ForBlock(b) => b.get_meta(),
            FSRToken::Getter(fsrslice) => fsrslice.get_meta(),
            FSRToken::StackExpr(fsrexprs) => fsrexprs.1.first().unwrap().get_meta(),
            FSRToken::TryBlock(fsrtry_block) => fsrtry_block.get_meta(),
        }
    }

    pub fn is_stack_expr(&self) -> bool {
        matches!(self, FSRToken::StackExpr(_))
    }

    pub fn is_call(&self) -> bool {
        matches!(self, FSRToken::Call(_))
    }

    pub fn is_getter(&self) -> bool {
        matches!(self, FSRToken::Getter(_))
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, FSRToken::EmptyExpr)
    }

    pub fn try_push_stack_expr(&mut self, value: FSRToken<'a>) -> Result<(), SyntaxError> {
        if let FSRToken::StackExpr(e) = self {
            e.1.push(value);
            return Ok(());
        }
        Err(SyntaxError::new(value.get_meta(), "Empty stack expression"))
    }

    pub fn flatten_comma(&'a self) -> Vec<FSRToken<'a>> {
        let mut v = vec![];
        match self {
            FSRToken::Expr(e) => {
                if e.get_op() == "," {
                    let left = e.get_left();
                    let right = e.get_right();
                    let mut tmp = left.flatten_comma();
                    tmp.extend(right.flatten_comma());
                    v.extend(tmp);
                    return v;
                }
            }
            _ => {},
        }

        v.push(self.clone());
        v
    }
}

pub trait FSRTokenMatcher {
    fn match_token() -> bool;
}

#[derive(Clone, Debug)]
pub struct FSRPosition {
    pub(crate) offset: usize,
}

impl Default for FSRPosition {
    fn default() -> Self {
        Self::new()
    }
}

impl FSRPosition {
    pub fn new() -> Self {
        Self { offset: 0 }
    }

    #[inline]
    pub fn from_offset(&self, offset: usize) -> FSRPosition {
        Self {
            offset: self.offset + offset,
        }
    }
}

impl Display for FSRPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.offset)
    }
}
