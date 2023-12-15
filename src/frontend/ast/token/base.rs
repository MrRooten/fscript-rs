use crate::backend::base_type::function::FSRFunction;

use super::{name::FSRName, if_statement::{FSRIf, FSRIfState}, constant::FSRConstant, assign::FSRAssign, expr::FSRBinOp, call::FSRCall, hashtable::FSRHashtable, function_def::FSRFunctionDef};

#[derive(Debug)]
pub enum FSRToken<'a> {
    FunctionDef(FSRFunctionDef<'a>),
    Name(FSRName),
    IfExp(FSRIf<'a>),
    Constant(FSRConstant<'a>),
    Assign(FSRAssign<'a>),
    Expr(FSRBinOp<'a>),
    Call(FSRCall<'a>),
    Hashtable(FSRHashtable),
}

pub enum FSRTokenState {
    If(FSRIfState),
}


