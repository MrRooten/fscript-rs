use std::fmt::Display;
use std::rc::Rc;

use crate::frontend::ast::token::module::FSRModuleFrontEnd;
use crate::{frontend::ast::token::block::FSRBlock, utils::error::SyntaxError};

use super::expr::SingleOp;
use super::try_expr::FSRTryBlock;
use super::ASTContext;
use super::{
    assign::FSRAssign, call::FSRCall, class::FSRClassFrontEnd, constant::FSRConstant,
    expr::FSRExpr, for_statement::FSRFor, function_def::FSRFnDef, if_statement::FSRIf,
    import::FSRImport, list::FSRListFrontEnd, return_def::FSRReturn, slice::FSRGetter,
    variable::FSRVariable, while_statement::FSRWhile,
};

#[derive(Debug, Clone)]
pub enum FSRToken {
    FunctionDef(Rc<FSRFnDef>),
    IfExp(FSRIf),
    Constant(FSRConstant),
    Assign(FSRAssign),
    Break(FSRPosition),
    Continue(FSRPosition),
    Expr(FSRExpr),
    // Case like a[0][1]
    StackExpr((Option<SingleOp>, Vec<FSRToken>)),
    ForBlock(FSRFor),
    Call(FSRCall),
    Variable(FSRVariable),
    Return(FSRReturn),
    Block(FSRBlock),
    WhileExp(FSRWhile),
    Module(FSRModuleFrontEnd),
    Import(FSRImport),
    List(FSRListFrontEnd),
    Class(FSRClassFrontEnd),
    Getter(FSRGetter),
    TryBlock(FSRTryBlock),
    EmptyExpr(FSRPosition),
    None,
}

impl FSRToken {
    pub fn set_single_op(&mut self, op: SingleOp) {
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

    pub fn as_variable(&self) -> &FSRVariable {
        match self {
            FSRToken::Variable(e) => e,
            _ => panic!("Not a variable"),
        }
    }

    pub fn as_mut_variable(&mut self) -> &mut FSRVariable {
        match self {
            FSRToken::Variable(e) => e,
            _ => panic!("Not a variable"),
        }
    }

    pub fn deduction_type(&self, context: &ASTContext) -> Option<FSRType> {
        match self {
            FSRToken::Variable(e) => {
                let name = e.get_name();
                if let FSRToken::Variable(v) = context.get_token(name)? {
                    return v.var_type.clone()
                }

                None
            },
            FSRToken::Call(c) => {
                let state = context.get_token(c.get_name())?;
                if let FSRToken::FunctionDef(c) = &state {
                    return c.ret_type.clone()
                }

                None
            }
            FSRToken::Constant(c) => {
                Some(c.deduction())
            }
            _ => None,
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
            FSRToken::EmptyExpr(e) => e,
            FSRToken::None => todo!(),
            FSRToken::List(e) => e.get_meta(),
            FSRToken::Class(e) => e.get_meta(),
            FSRToken::Break(e) => e,
            FSRToken::Continue(e) => e,
            FSRToken::ForBlock(b) => b.get_meta(),
            FSRToken::Getter(fsrgetter) => fsrgetter.get_meta(),
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
        matches!(self, FSRToken::EmptyExpr(_))
    }

    pub fn is_check_exception(&self) -> bool {
        match self {
            _ => false,
        }
    }

    pub fn try_push_stack_expr(&mut self, value: FSRToken) -> Result<(), SyntaxError> {
        if let FSRToken::StackExpr(e) = self {
            e.1.push(value);
            return Ok(());
        }
        Err(SyntaxError::new(value.get_meta(), "Empty stack expression"))
    }

    pub fn flatten_comma(&self) -> Vec<FSRToken> {
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
            _ => {
                
            },
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

    pub fn from_offset(offset: usize) -> Self {
        Self { offset }
    }

    pub fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
    }

    pub fn get_offset(&self) -> usize {
        self.offset
    }

    pub fn new_offset(&self, offset: usize) -> Self {
        Self {
            offset: self.offset + offset,
        }
    }
}

impl Display for FSRPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FSRPosition(offset: {})", self.offset)
    }
}


#[derive(Debug, Clone)]
pub struct FSRType {
    pub(crate) name: String,
    pub(crate) subtype: Option<Vec<Box<FSRType>>>,
}

impl FSRType {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), subtype: None }
    }
}