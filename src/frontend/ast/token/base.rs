use crate::backend::base_type::function::FSRFunction;

use super::{name::FSRName, if_statement::FSRIf, constant::FSRConstant, assign::FSRAssign, expr::FSRExpr, call::FSRCall, hashtable::FSRHashtable};

pub enum FSRToken<'a> {
    FunctionDef(FSRFunction),
    Name(FSRName),
    IfExp(FSRIf<'a>),
    Constant(FSRConstant),
    Assign(FSRAssign<'a>),
    Expr(FSRExpr<'a>),
    Call(FSRCall<'a>),
    Hashtable(FSRHashtable)
}