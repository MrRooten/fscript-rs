use std::{collections::{HashMap, LinkedList}, sync::atomic::{AtomicU64, Ordering}};

use crate::frontend::ast::token::{base::FSRToken, call::FSRCall, expr::FSRExpr, variable::FSRVariable};

#[derive(Debug)]
pub enum BytecodeOperator {
    Push,
    Pop,
    Load,
    LoadAttr,
    Assign,
    BinaryAdd,
    BinaryMul,
    ReturnValue,
    Call,
    BinaryDot,
    InsertArg
}

#[derive(Debug)]
pub enum ArgType {
    Variable(u64, String),
    Attr(u64, String),
    BinaryOperator,
    CallOperator,
    InsertArg
}

#[derive(Debug)]
pub struct BytecodeArg {
    operator        : BytecodeOperator,
    arg             : ArgType
}

impl BytecodeArg {
    pub fn new(op: BytecodeOperator, id: u64) {
        
    }
}

impl BytecodeOperator {
    pub fn get_op(op: &str) -> BytecodeArg {
        if op.eq("+") {
            return BytecodeArg {
                operator: BytecodeOperator::BinaryAdd,
                arg: ArgType::BinaryOperator
            }
        } else if op.eq("*") {
            return BytecodeArg {
                operator: BytecodeOperator::BinaryMul,
                arg: ArgType::BinaryOperator
            }
        } else if op.eq(".") {
            return BytecodeArg {
                operator: BytecodeOperator::BinaryDot,
                arg: ArgType::BinaryOperator
            }
        }
        unimplemented!()
    }
}


#[derive(Debug)]
pub struct VarMap<'a> {
    var_map     : HashMap<&'a str, u64>,
    var_id      : AtomicU64,
    attr_map    : HashMap<&'a str, u64>,
    attr_id     : AtomicU64,
}

impl<'a> VarMap<'a> {
    pub fn has_var(&self, var: &str) -> bool {
        return self.var_map.get(var).is_some();
    }

    pub fn get_var(&self, var: &str) -> Option<&u64> {
        return self.var_map.get(var);
    }

    pub fn insert_var(&mut self, var: &'a str) {
        let v = self.var_id.fetch_add(1, Ordering::Relaxed);
        self.var_map.insert(var, v);
    }


    pub fn insert_attr(&mut self, attr: &'a str) {
        let v = self.attr_id.fetch_add(1, Ordering::Relaxed);
        self.attr_map.insert(attr, v);
    }

    pub fn has_attr(&self, attr: &str) -> bool {
        return self.attr_map.get(attr).is_some();
    }

    pub fn get_attr(&self, attr: &str) -> Option<&u64> {
        return self.attr_map.get(attr);
    }

    pub fn new() -> Self {
        Self {
            var_map: HashMap::new(),
            var_id: AtomicU64::new(100),
            attr_map: HashMap::new(),
            attr_id: AtomicU64::new(100),
        }
    }
}

pub struct Bytecode {

}

impl<'a> Bytecode {
    fn load_call(call: &'a FSRCall<'a>, var_map: &'a mut VarMap<'a>, is_attr: bool) -> (LinkedList<BytecodeArg>, &'a mut VarMap<'a>) {
        let mut result = LinkedList::new();

        for arg in call.get_args() {
            let mut v = Self::load_token(arg);
            result.append(&mut v);
            result.push_back(BytecodeArg {
                operator: BytecodeOperator::InsertArg,
                arg: ArgType::InsertArg
            });
        }

        let name = call.get_name();
        if is_attr {
            if var_map.has_attr(name) == false {
                let v = name;
                var_map.insert_attr(v);
            }
            let id = var_map.get_attr(name).unwrap();
            result.push_back(BytecodeArg {
                operator: BytecodeOperator::Load,
                arg: ArgType::Variable(id.clone(), name.to_string())
            });
        } else {
            if var_map.has_var(name) == false {
                let v = name;
                var_map.insert_var(v);
            }
            let id = var_map.get_var(name).unwrap();
            result.push_back(BytecodeArg {
                operator: BytecodeOperator::Load,
                arg: ArgType::Variable(id.clone(), name.to_string())
            });
        }

        result.push_back(BytecodeArg {
            operator: BytecodeOperator::Call,
            arg: ArgType::CallOperator
        });


        return (result, var_map);
    }

    fn load_variable(var: &'a FSRVariable<'a>, var_map: &'a mut VarMap<'a>) -> (LinkedList<BytecodeArg>, &'a mut VarMap<'a>) {
        if var_map.has_var(&var.get_name()) == false {
            let v = var.get_name();
            var_map.insert_var(v);
        }

        let arg_id = var_map.get_var(&var.get_name()).unwrap();

        let op_arg = BytecodeArg {
            operator: BytecodeOperator::Load,
            arg: ArgType::Variable(arg_id.clone(), var.get_name().to_string())
        };
        let mut ans = LinkedList::new();
        ans.push_back(op_arg);

        return (ans, var_map);
    }

    fn load_expr(expr: &'a FSRExpr<'a>, var_map: &'a mut VarMap<'a>) -> (LinkedList<BytecodeArg>, &'a mut VarMap<'a>) {
        let mut op_code = LinkedList::new();
        let mut var_map_ref = Some(var_map);
        if let FSRToken::Expr(sub_expr) = &**expr.get_left() {
            let mut v = Self::load_expr(sub_expr, var_map_ref.unwrap());
            op_code.append(&mut v.0);
            var_map_ref = Some(v.1);

        } else if let FSRToken::Variable(v) = &**expr.get_left() {
            let mut v = Self::load_variable(v, var_map_ref.unwrap());
            op_code.append(&mut v.0);
            var_map_ref = Some(v.1);
        } else if let FSRToken::Call(c) = &**expr.get_left() {
            let mut v = Self::load_call(c, var_map_ref.unwrap(), false);
            op_code.append(&mut v.0);
            var_map_ref = Some(v.1);
        }
        else {
            unimplemented!()
        }

        if let FSRToken::Expr(sub_expr) = &**expr.get_right() {
            let mut v = Self::load_expr(sub_expr, var_map_ref.unwrap());
            op_code.append(&mut v.0);
            var_map_ref = Some(v.1);
        } else if let FSRToken::Variable(v) = &**expr.get_right() {
            let mut v = Self::load_variable(v, var_map_ref.unwrap());
            op_code.append(&mut v.0);
            var_map_ref = Some(v.1);
        }

        op_code.push_back(BytecodeOperator::get_op(expr.get_op()));

        return (op_code, var_map_ref.unwrap());
    }

    fn load_token(token: &FSRToken<'a>) -> LinkedList<BytecodeArg> {
        let mut var_map = VarMap::new();
        if let FSRToken::Expr(expr) = token {
            let v = Self::load_expr(&expr, &mut var_map);
            return v.0;
        }
        else if let FSRToken::Variable(v) = token {
            let v = Self::load_variable(v, &mut var_map);
            return v.0;
        }

        unimplemented!()
    }

    pub fn load_ast(token: FSRToken<'a>) -> LinkedList<BytecodeArg> {
        let v = Self::load_token(&token);
        return v;
    }
}