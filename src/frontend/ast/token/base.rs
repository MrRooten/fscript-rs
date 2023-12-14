use crate::backend::base_type::function::FSRFunction;

use super::{name::FSRName, if_statement::{FSRIf, FSRIfState}, constant::FSRConstant, assign::FSRAssign, expr::FSRExpr, call::FSRCall, hashtable::FSRHashtable, function_def::FSRFunctionDef, bin_op::FSRBinOp};

pub enum FSRToken<'a> {
    FunctionDef(FSRFunctionDef<'a>),
    Name(FSRName),
    IfExp(FSRIf<'a>),
    Constant(FSRConstant<'a>),
    Assign(FSRAssign<'a>),
    Expr(FSRExpr<'a>),
    Call(FSRCall<'a>),
    Hashtable(FSRHashtable),
    BinOp(FSRBinOp<'a>)
}

pub enum FSRTokenState {
    If(FSRIfState),
}


