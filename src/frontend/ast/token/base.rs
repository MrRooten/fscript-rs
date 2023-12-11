use crate::backend::base_type::function::FSRFunction;

use super::{name::FSRName, if_statement::FSRIf, constant::FSRConstant, assign::FSRAssign, expr::FSRExpr, call::FSRCall, hashtable::FSRHashtable};

pub enum FSRToken {
    FunctionDef(FSRFunction),
    Name(FSRName),
    IfExp(FSRIf),
    Constant(FSRConstant),
    Assign(FSRAssign),
    Expr(FSRExpr),
    Call(FSRCall),
    Hashtable(FSRHashtable)
}