use crate::backend::base_type::function::FSRFunction;

use super::{name::FSRName, if_statement::{FSRIf, FSRIfState}, constant::FSRConstant, assign::FSRAssign, expr::FSRExpr, call::FSRCall, hashtable::FSRHashtable, function_def::FSRFunctionDef};

pub enum FSRToken<'a> {
    FunctionDef(FSRFunctionDef<'a>),
    Name(FSRName),
    IfExp(FSRIf<'a>),
    Constant(FSRConstant),
    Assign(FSRAssign<'a>),
    Expr(FSRExpr<'a>),
    Call(FSRCall<'a>),
    Hashtable(FSRHashtable)
}

pub enum FSRTokenState {
    If(FSRIfState),
}