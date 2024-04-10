use std::{
    collections::{HashMap, LinkedList},
    sync::atomic::{AtomicU64, Ordering},
};

use crate::frontend::ast::token::{
    assign::FSRAssign, base::FSRToken, block::FSRBlock, call::FSRCall, constant::{FSRConstant, FSRConstantType}, expr::FSRExpr, function_def::FSRFnDef, if_statement::FSRIf, variable::FSRVariable, while_statement::FSRWhile
};

#[derive(Debug, PartialEq)]
pub enum BytecodeOperator {
    Push,
    Pop,
    Load,
    LoadAttr,
    Assign,
    AssignArgs,
    BinaryAdd,
    BinarySub,
    BinaryMul,
    CompareTest,
    ReturnValue,
    Call,
    BinaryDot,
    InsertArg,
    IfBlockStart,
    IfTest,
    IfBlockEnd,
    WhileBlockStart,
    WhileTest,
    WhileBlockEnd,
    DefineFn,
    EndDefineFn
}

#[derive(Debug)]
pub enum ArgType {
    Variable(u64, String),
    ConstString(u64, String),
    ConstInteger(u64, i64),
    Attr(u64, String),
    IfTestNext(u64),
    WhileTest(u64), //i64 is return to test, u64 is skip the block,
    WhileEnd(i64),
    Compare(&'static str),
    FnLines(usize),
    CallArgsNumber(usize),
    DefineFnArgs(u64, u64),
    None,
}

#[derive(Debug)]
pub struct BytecodeArg {
    operator: BytecodeOperator,
    arg: ArgType,
}

impl BytecodeArg {
    pub fn new(op: BytecodeOperator, id: u64) {}

    pub fn get_operator(&self) -> &BytecodeOperator {
        return &self.operator
    }

    pub fn get_arg(&self) -> &ArgType {
        return &self.arg
    }
}

impl BytecodeOperator {

    pub fn get_static_op(op: &str) -> &'static str {
        // op reference my not life longer enough, so return static str
        if op.eq(">") {
            return ">"
        }
        else if op.eq("<") {
            return "<"
        }
        else if op.eq(">=") {
            return ">="
        }
        else if op.eq("<=") {
            return "<="
        }
        else if op.eq("==") {
            return "=="
        }
        else if op.eq(".") {
            return ".";
        }

        unimplemented!()
    }

    pub fn get_op(op: &str) -> BytecodeArg {
        if op.eq("+") {
            return BytecodeArg {
                operator: BytecodeOperator::BinaryAdd,
                arg: ArgType::None,
            };
        } else if op.eq("*") {
            return BytecodeArg {
                operator: BytecodeOperator::BinaryMul,
                arg: ArgType::None,
            };
        } else if op.eq(".") {
            return BytecodeArg {
                operator: BytecodeOperator::BinaryDot,
                arg: ArgType::None,
            };
        } else if op.eq("=") {
            return BytecodeArg {
                operator: BytecodeOperator::Assign,
                arg: ArgType::None,
            };
        } else if op.eq(">") || op.eq("<") || op.eq(">=") || op.eq("<=") {
            return BytecodeArg {
                operator: BytecodeOperator::CompareTest,
                arg: ArgType::Compare(Self::get_static_op(op))
            }
        }
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct VarMap<'a> {
    var_map: HashMap<&'a str, u64>,
    var_id: AtomicU64,
    attr_map: HashMap<&'a str, u64>,
    attr_id: AtomicU64,
    const_map: HashMap<&'a str, FSRConstant>,
    const_id: AtomicU64,
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
            const_map: HashMap::new(),
            const_id: AtomicU64::new(100),
        }
    }
}

pub struct ExprList {
    var_map     : VarMap<'static>,
    list        : Vec<LinkedList<BytecodeArg>>
}

#[derive(Debug)]
pub struct Bytecode {
    bytecode        : Vec<LinkedList<BytecodeArg>>,
}

impl<'a> Bytecode {
    pub fn get(&self, index: usize) -> Option<&LinkedList<BytecodeArg>> {
        return self.bytecode.get(index);
    }

    fn load_call(
        call: &'a FSRCall<'a>,
        var_map: &'a mut VarMap<'a>,
        is_attr: bool,
    ) -> (LinkedList<BytecodeArg>, &'a mut VarMap<'a>) {
        let mut result = LinkedList::new();
        let mut var_map_ref = var_map;
        for arg in call.get_args() {
            let mut v = Self::load_token_with_map(arg, var_map_ref);
            var_map_ref = v.1;
            result.append(&mut v.0[0]);
            // result.push_back(BytecodeArg {
            //     operator: BytecodeOperator::InsertArg,
            //     arg: ArgType::None,
            // });
        }

        let name = call.get_name();
        if is_attr {
            if var_map_ref.has_attr(name) == false {
                let v = name;
                var_map_ref.insert_attr(v);
            }
            let id = var_map_ref.get_attr(name).unwrap();
            result.push_back(BytecodeArg {
                operator: BytecodeOperator::Load,
                arg: ArgType::Attr(id.clone(), name.to_string()),
            });
            result.push_back(BytecodeArg {
                operator: BytecodeOperator::BinaryDot,
                arg: ArgType::None,
            });
        } else {
            if var_map_ref.has_var(name) == false {
                let v = name;
                var_map_ref.insert_var(v);
            }
            let id = var_map_ref.get_var(name).unwrap();
            result.push_back(BytecodeArg {
                operator: BytecodeOperator::Load,
                arg: ArgType::Variable(id.clone(), name.to_string()),
            });
        }

        result.push_back(BytecodeArg {
            operator: BytecodeOperator::Call,
            arg: ArgType::CallArgsNumber(call.get_args().len()),
        });

        return (result, var_map_ref);
    }

    fn load_variable(
        var: &'a FSRVariable<'a>,
        var_map: &'a mut VarMap<'a>,
        is_attr: bool
    ) -> (LinkedList<BytecodeArg>, &'a mut VarMap<'a>) {
        if var_map.has_var(&var.get_name()) == false {
            let v = var.get_name();
            var_map.insert_var(v);
        }

        let arg_id = var_map.get_var(&var.get_name()).unwrap();
        let op_arg;
        if is_attr {
            op_arg = BytecodeArg {
                operator: BytecodeOperator::Load,
                arg: ArgType::Attr(arg_id.clone(), var.get_name().to_string()),
            };
        } else {
            op_arg = BytecodeArg {
                operator: BytecodeOperator::Load,
                arg: ArgType::Variable(arg_id.clone(), var.get_name().to_string()),
            };
        }
        
        let mut ans = LinkedList::new();
        ans.push_back(op_arg);
        
        return (ans, var_map);
    }

    fn load_assign_arg(
        var: &'a FSRVariable<'a>,
        var_map: &'a mut VarMap<'a>,
    ) -> (LinkedList<BytecodeArg>, &'a mut VarMap<'a>) {
        if var_map.has_var(&var.get_name()) == false {
            let v = var.get_name();
            var_map.insert_var(v);
        }

        let arg_id = var_map.get_var(&var.get_name()).unwrap();
        let op_arg;

        op_arg = BytecodeArg {
            operator: BytecodeOperator::AssignArgs,
            arg: ArgType::Variable(arg_id.clone(), var.get_name().to_string()),
        };
        
        
        let mut ans = LinkedList::new();
        ans.push_back(op_arg);
        
        return (ans, var_map);
    }

    fn load_expr(
        expr: &'a FSRExpr<'a>,
        var_map: &'a mut VarMap<'a>,
    ) -> (LinkedList<BytecodeArg>, &'a mut VarMap<'a>) {
        let mut op_code = LinkedList::new();
        let mut var_map_ref = Some(var_map);
        if let FSRToken::Expr(sub_expr) = &**expr.get_left() {
            let mut v = Self::load_expr(sub_expr, var_map_ref.unwrap());
            op_code.append(&mut v.0);
            var_map_ref = Some(v.1);
        } else if let FSRToken::Variable(v) = &**expr.get_left() {
            
            let mut v = Self::load_variable(v, var_map_ref.unwrap(), false);
            op_code.append(&mut v.0);
            var_map_ref = Some(v.1);
        } else if let FSRToken::Call(c) = &**expr.get_left() {
            let mut v = Self::load_call(c, var_map_ref.unwrap(), false);
            op_code.append(&mut v.0);
            var_map_ref = Some(v.1);
        } else if let FSRToken::Constant(c) = &**expr.get_left() {
            let mut v = Self::load_constant(c, var_map_ref.unwrap());
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
            let mut is_attr = false;
            if expr.get_op().eq(".") {
                is_attr = true;
            }
            let mut v = Self::load_variable(v, var_map_ref.unwrap(), is_attr);
            op_code.append(&mut v.0);
            var_map_ref = Some(v.1);
        } else if let FSRToken::Call(c) = &**expr.get_right() {
            let mut is_attr = false;
            if expr.get_op().eq(".") {
                is_attr = true;
            }
            let mut v = Self::load_call(c, var_map_ref.unwrap(), is_attr);
            op_code.append(&mut v.0);
            var_map_ref = Some(v.1);
            //call special process
            return (op_code, var_map_ref.unwrap());
        } else if let FSRToken::Constant(c) = &**expr.get_right() {
            let mut v = Self::load_constant(c, var_map_ref.unwrap());
            op_code.append(&mut v.0);
            var_map_ref = Some(v.1);
        }

        

        op_code.push_back(BytecodeOperator::get_op(expr.get_op()));

        return (op_code, var_map_ref.unwrap());
    }

    fn load_block(
        block: &'a FSRBlock<'a>,
        var_map: &'a mut VarMap<'a>,
    ) -> (Vec<LinkedList<BytecodeArg>>, &'a mut VarMap<'a>) {
        let mut vs = vec![];
        let mut ref_self = var_map;
        for token in block.get_tokens() {
            let lines = Self::load_token_with_map(token, ref_self);
            ref_self = lines.1;
            let lines = lines.0;
            for line in lines {
                vs.push(line);
            }
        }

        return (vs, ref_self);
    }

    fn load_if_def(
        if_def: &'a FSRIf<'a>,
        var_map: &'a mut VarMap<'a>,
    ) -> (Vec<LinkedList<BytecodeArg>>, &'a mut VarMap<'a>) {
        let test_exp = if_def.get_test();
        let mut vs = vec![];
        let mut v = Self::load_token_with_map(&test_exp, var_map);
        let mut test_list = LinkedList::new();
        let mut t = v.0.remove(0);
        test_list.append(&mut t);

        let block_items = Self::load_block(&if_def.get_block(), v.1);
        test_list.push_back(BytecodeArg {
            operator: BytecodeOperator::IfTest,
            arg: ArgType::IfTestNext(block_items.0.len() as u64),
        });
        vs.push(test_list);
        vs.extend(block_items.0);

        return (vs, block_items.1);
    }

    fn load_while_def(
        while_def: &'a FSRWhile<'a>,
        var_map: &'a mut VarMap<'a>,
    ) -> (Vec<LinkedList<BytecodeArg>>, &'a mut VarMap<'a>) {
        let test_exp = while_def.get_test();
        let mut vs = vec![];
        let mut v = Self::load_token_with_map(&test_exp, var_map);
        let mut test_list = LinkedList::new();
        let mut t = v.0.remove(0);
        test_list.append(&mut t);

        let mut block_items = Self::load_block(&while_def.get_block(), v.1);
        test_list.push_back(BytecodeArg {
            operator: BytecodeOperator::WhileTest,
            arg: ArgType::WhileTest(block_items.0.len() as u64),
        });
        vs.push(test_list);
        let len = block_items.0.len();
        let l = block_items.0.get_mut(len - 1).unwrap();
        l.push_back(BytecodeArg {
            operator: BytecodeOperator::WhileBlockEnd,
            arg: ArgType::WhileEnd(len as i64),
        });
        vs.extend(block_items.0);

        return (vs, block_items.1);
    }

    fn load_token_with_map(
        token: &'a FSRToken<'a>,
        var_map: &'a mut VarMap<'a>,
    ) -> (Vec<LinkedList<BytecodeArg>>, &'a mut VarMap<'a>) {
        if let FSRToken::Expr(expr) = token {
            let v = Self::load_expr(&expr, var_map);
            let r = v.1;
            return (vec![v.0], r);
        } else if let FSRToken::Variable(v) = token {
            let v = Self::load_variable(v, var_map, false);
            let r = v.1;
            return (vec![v.0], r);
        } else if let FSRToken::Module(m) = token {
            let mut vs = vec![];
            let mut ref_self = var_map;
            for token in &m.tokens {
                
                let lines = Self::load_token_with_map(token, ref_self);
                ref_self = lines.1;
                let lines = lines.0;
                for line in lines {
                    vs.push(line);
                }
            }

            return (vs, ref_self);
        } else if let FSRToken::IfExp(if_def) = token {
            let v = Self::load_if_def(if_def, var_map);

            return (v.0, v.1);
        } else if let FSRToken::Assign(assign) = token {
            let v = Self::load_assign(assign, var_map);
            return (vec![v.0], v.1);
        } else if let FSRToken::WhileExp(while_def) = token {
            let v = Self::load_while_def(while_def, var_map);
            return (v.0, v.1);
        } else if let FSRToken::Block(block) = token {
            let v = Self::load_block(block, var_map);
            return (v.0, v.1);
        }  else if let FSRToken::Call(call) = token {
            let v = Self::load_call(call, var_map, false);
            return (vec![v.0], v.1)
        }  else if let FSRToken::Constant(c) = token {
            let v = Self::load_constant(c, var_map);
            return (vec![v.0], v.1);
        }  else if let FSRToken::FunctionDef(fn_def) = token {
            let v = Self::load_function(fn_def, var_map);
            return (v.0, v.1)
        }

        unimplemented!()
    }

    fn load_assign(
        token: &'a FSRAssign<'a>,
        var_map: &'a mut VarMap<'a>,
    ) -> (LinkedList<BytecodeArg>, &'a mut VarMap<'a>) {
        let mut result_list = LinkedList::new();
        let mut left = Self::load_token_with_map(token.get_left(), var_map);
        let mut right = Self::load_token_with_map(token.get_assign_expr(), left.1);
        result_list.append(&mut right.0[0]);
        result_list.append(&mut left.0[0]);
        result_list.push_back(BytecodeArg {
            operator: BytecodeOperator::Assign,
            arg: ArgType::None,
        });
        return (result_list, right.1);
    }

    fn load_constant(
        token: &'a FSRConstant,
        var_map: &'a mut VarMap<'a>
    ) -> (LinkedList<BytecodeArg>, &'a mut VarMap<'a>) {
        let mut result_list = LinkedList::new();
        if let FSRConstantType::Integer(i) = token.get_constant() {
            result_list.push_back(BytecodeArg {
                operator: BytecodeOperator::Load,
                arg: ArgType::ConstInteger(0, i.clone()),
            });
        }
        else if let FSRConstantType::String(s) = token.get_constant() {
            result_list.push_back(BytecodeArg {
                operator: BytecodeOperator::Load,
                arg: ArgType::ConstString(0, String::from_utf8_lossy(s).to_string()),
            });
        }
        
        return (result_list, var_map)
    }

    fn load_function(fn_def: &'a FSRFnDef<'a>, var_map: &'a mut VarMap<'a>) -> (Vec<LinkedList<BytecodeArg>>, &'a mut VarMap<'a>) {
        let mut result = vec![];
        let name = fn_def.get_name();
        //let mut define_fn = LinkedList::new();
        if var_map.has_var(name) == false {
            var_map.insert_var(name);
        }

        let mut fn_var_map = VarMap::new();
        let mut fn_var_map_ref = &mut fn_var_map;
        let args = fn_def.get_args();
        let mut var_map = var_map;
        let mut args_load = LinkedList::new();
        let mut arg_len = 0;
        for arg in args {
            if let FSRToken::Variable(v) = arg {
                let mut a = Self::load_variable(v, fn_var_map_ref, false);
                fn_var_map_ref = a.1;
                args_load.append(&mut a.0);
                arg_len += 1;
            }
        }
        

        

        let arg_id = var_map.get_var(name).unwrap();

        let op_arg = BytecodeArg {
            operator: BytecodeOperator::Load,
            arg: ArgType::Variable(arg_id.clone(), name.to_string()),
        };
        
        

        
        
        let body = fn_def.get_body();
        let v = Self::load_block(body, fn_var_map_ref);
        fn_var_map_ref = v.1;
        let fn_body = v.0;
        // let mut end_list = LinkedList::new();
        // end_list.push_back(BytecodeArg {
        //     operator: BytecodeOperator::EndDefineFn,
        //     arg: ArgType::None,
        // });
        //define_fn.push_back(op_arg);
        let mut load_args = LinkedList::new();
        for arg in args {
            if let FSRToken::Variable(v) = arg {
                let mut a = Self::load_assign_arg(v, fn_var_map_ref);
                fn_var_map_ref = a.1;
                load_args.append(&mut a.0);
            }
        }

        args_load.push_back(op_arg);
        args_load.push_back(BytecodeArg {
            operator: BytecodeOperator::DefineFn,
            arg: ArgType::DefineFnArgs(fn_body.len() as u64 + 1, arg_len),
        });
        // define_fn.push_back(BytecodeArg {
        //     operator: BytecodeOperator::DefineFn,
        //     arg: ArgType::DefineFnArgs(fn_body.len() as u64 + 1, arg_len),
        // });

        result.push(args_load);
        result.push(load_args);
        //result.push(define_fn);
        result.extend(fn_body);

        let mut end_of_fn = LinkedList::new();
        end_of_fn.push_back(BytecodeArg {
            operator: BytecodeOperator::EndDefineFn,
            arg: ArgType::None,
        });
        result.push(end_of_fn);
        // result.push(end_list);
        return (result, var_map);
    }

    fn load_isolate_block(token: &FSRToken<'a>) -> Vec<LinkedList<BytecodeArg>> {
        let mut var_map = VarMap::new();
        let v = Self::load_token_with_map(token, &mut var_map);
        return v.0;
    }

    pub fn load_ast(token: FSRToken<'a>) -> Bytecode {
        let v = Self::load_isolate_block(&token);
        return Self {
            bytecode: v
        }
    }
}
