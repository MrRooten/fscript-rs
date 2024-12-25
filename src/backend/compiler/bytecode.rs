use std::{
    borrow::Cow,
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{
    backend::{
        types::{base::ObjId, integer::FSRInteger, string::FSRString},
        vm::runtime::FSRVM,
    },
    frontend::ast::token::{
        assign::FSRAssign,
        base::{FSRPosition, FSRToken},
        block::FSRBlock,
        call::FSRCall,
        class::FSRClassFrontEnd,
        constant::{FSRConstant, FSRConstantType, FSROrinStr, FSROrinStr2},
        expr::FSRExpr,
        for_statement::FSRFor,
        function_def::FSRFnDef,
        if_statement::FSRIf,
        import::FSRImport,
        list::FSRListFrontEnd,
        module::FSRModuleFrontEnd,
        return_def::FSRReturn,
        variable::FSRVariable,
        while_statement::FSRWhile,
    },
};

#[derive(Debug, Clone, Copy)]
pub enum BinaryOffset {
    Add = 0,
    Sub = 1,
    Mul = 2,
    Greater = 3,
    GreatEqual = 4,
    Less = 5,
    LessEqual = 6,
    Equal = 7,
    NotEqual = 8,
    NextObject,
    GetItem,
    SetItem,
}

impl BinaryOffset {
    #[inline(always)]
    pub fn alias_name(&self) -> &str {
        match self {
            BinaryOffset::Add => "__add__",
            BinaryOffset::Sub => "__sub__",
            BinaryOffset::Mul => "__mul__",
            BinaryOffset::Greater => "__gt__",
            BinaryOffset::GreatEqual => "__gte__",
            BinaryOffset::Less => "__lt__",
            BinaryOffset::LessEqual => "__lte__",
            BinaryOffset::Equal => "__eq__",
            BinaryOffset::NotEqual => "__neq__",
            BinaryOffset::NextObject => "__next__",
            BinaryOffset::GetItem => "__get__",
            BinaryOffset::SetItem => "__set__",
        }
    }
}

impl From<BinaryOffset> for usize {
    fn from(val: BinaryOffset) -> Self {
        val as usize
    }
}

#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy)]
pub enum BytecodeOperator {
    Assign = 0,
    BinaryAdd = 1,
    BinaryDot = 2,
    BinaryMul = 3,
    Call = 4,
    IfTest = 5,
    WhileTest = 6,
    DefineFn = 7,
    EndDefineFn = 8,
    CompareTest = 9,
    ReturnValue = 10,
    WhileBlockEnd = 11,
    AssignArgs = 12,
    ClassDef = 13,
    EndDefineClass = 14,
    LoadList = 15,
    Else = 16,
    ElseIf = 17,
    ElseIfTest = 18,
    IfBlockEnd = 19,
    Break = 20,
    Continue = 21,
    LoadForIter = 22,
    PushForNext = 24, // call iter_obj.__next__()
    ForBlockEnd = 23,
    SpecialLoadFor = 25,
    AndJump = 26,
    OrJump = 27,
    Empty = 28,
    // BinarySub,
    BinaryRShift = 29,
    BinaryLShift = 30,
    StoreFast = 31,
    BinarySub = 32,
    Import = 33,
    Load = 1000,
}

#[derive(Debug)]
pub enum ArgType {
    Variable(u64, String),
    ImportModule(u64, Vec<String>),
    VariableList(Vec<(u64, String)>),
    ConstString(u64, String),
    ConstInteger(u64, i64),
    Attr(u64, String),
    BinaryOperator(BinaryOffset),
    IfTestNext((u64, u64)), // first u64 for if line, second for count else if /else
    WhileTest(u64),         //i64 is return to test, u64 is skip the block,
    WhileEnd(i64),
    Compare(&'static str),
    FnLines(usize),
    CallArgsNumber(usize),
    DefineFnArgs(u64, u64),
    DefineClassLine(u64),
    LoadListNumber(usize),
    ForEnd(i64),
    AddOffset(usize),
    ForLine(u64),
    StoreFastVar(u64, String),
    Import(Vec<String>),
    None,
}

#[derive(Debug)]
pub struct BytecodeArg {
    operator: BytecodeOperator,
    arg: ArgType,
}

impl BytecodeArg {
    #[inline]
    pub fn get_operator(&self) -> &BytecodeOperator {
        &self.operator
    }

    #[inline]
    pub fn get_arg(&self) -> &ArgType {
        &self.arg
    }
}

impl BytecodeOperator {
    pub fn get_static_op(op: &str) -> &'static str {
        // op reference my not life longer enough, so return static str
        if op.eq(">") {
            return ">";
        } else if op.eq("<") {
            return "<";
        } else if op.eq(">=") {
            return ">=";
        } else if op.eq("<=") {
            return "<=";
        } else if op.eq("==") {
            return "==";
        } else if op.eq(".") {
            return ".";
        } else if op.eq("!=") {
            return "!=";
        }

        unimplemented!()
    }

    pub fn get_op(op: &str) -> Option<BytecodeArg> {
        if op.eq("+") {
            return Some(BytecodeArg {
                operator: BytecodeOperator::BinaryAdd,
                arg: ArgType::None,
            });
        } else if op.eq("*") {
            return Some(BytecodeArg {
                operator: BytecodeOperator::BinaryMul,
                arg: ArgType::None,
            });
        } else if op.eq(".") {
            return Some(BytecodeArg {
                operator: BytecodeOperator::BinaryDot,
                arg: ArgType::None,
            });
        } else if op.eq("=") {
            return Some(BytecodeArg {
                operator: BytecodeOperator::Assign,
                arg: ArgType::None,
            });
        } else if op.eq(">")
            || op.eq("<")
            || op.eq(">=")
            || op.eq("<=")
            || op.eq("==")
            || op.eq("!=")
        {
            return Some(BytecodeArg {
                operator: BytecodeOperator::CompareTest,
                arg: ArgType::Compare(Self::get_static_op(op)),
            });
        } else if op.eq("<<") {
            return Some(BytecodeArg {
                operator: BytecodeOperator::BinaryLShift,
                arg: ArgType::None,
            });
        } else if op.eq(">>") {
            return Some(BytecodeArg {
                operator: BytecodeOperator::BinaryRShift,
                arg: ArgType::None,
            });
        } else if op.eq("-") {
            return Some(BytecodeArg {
                operator: BytecodeOperator::BinarySub,
                arg: ArgType::None,
            });
        }
        None
    }
}

#[derive(Debug)]
pub struct ConstTable {
    pub(crate) const_map: HashMap<FSROrinStr2, u64>,
    pub(crate) table: Vec<ObjId>,
}

#[allow(clippy::new_without_default)]
impl ConstTable {
    pub fn new() -> Self {
        Self {
            const_map: HashMap::new(),
            table: vec![0],
        }
    }

    pub fn insert(&mut self, c_id: usize, obj_id: ObjId) {
        if c_id + 1 > self.table.len() {
            self.table.resize(c_id + 1, 0);
        }

        self.table[c_id] = obj_id;
    }
}

#[derive(Debug)]
pub struct VarMap<'a> {
    var_map: HashMap<&'a str, u64>,
    var_id: AtomicU64,
    attr_map: HashMap<&'a str, u64>,
    attr_id: AtomicU64,
    #[allow(unused)]
    const_map: HashMap<FSROrinStr<'a>, u64>,
    #[allow(unused)]
    const_id: AtomicU64,
}

impl<'a> Default for VarMap<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> VarMap<'a> {
    pub fn has_var(&self, var: &str) -> bool {
        self.var_map.contains_key(var)
    }

    pub fn get_var(&self, var: &str) -> Option<&u64> {
        return self.var_map.get(var);
    }

    pub fn insert_var(&mut self, var: &'a str) {
        if self.var_map.contains_key(var) {
            return;
        }
        let v = self.var_id.fetch_add(1, Ordering::Acquire);
        self.var_map.insert(var, v);
    }

    pub fn has_const(&self, c: &FSROrinStr) -> bool {
        self.const_map.contains_key(c)
    }

    pub fn get_const(&self, c: &FSROrinStr) -> Option<u64> {
        self.const_map.get(c).copied()
    }

    pub fn insert_const(&mut self, c: &FSROrinStr<'a>) {
        if self.has_const(c) {
            return;
        }
        let v = self.const_id.fetch_add(1, Ordering::Acquire);
        self.const_map.insert(*c, v);
    }

    pub fn insert_attr(&mut self, attr: &'a str) {
        let v = self.attr_id.fetch_add(1, Ordering::Acquire);
        self.attr_map.insert(attr, v);
    }

    pub fn has_attr(&self, attr: &str) -> bool {
        self.attr_map.contains_key(attr)
    }

    pub fn get_attr(&self, attr: &str) -> Option<&u64> {
        return self.attr_map.get(attr);
    }

    pub fn new() -> Self {
        Self {
            var_map: HashMap::new(),
            var_id: AtomicU64::new(1),
            attr_map: HashMap::new(),
            attr_id: AtomicU64::new(1),
            const_map: HashMap::new(),
            const_id: AtomicU64::new(1),
        }
    }
}

#[derive(Debug)]
pub struct Bytecode {
    #[allow(unused)]
    name: String,
    pub(crate) const_table: ConstTable,
    bytecode: Vec<Vec<BytecodeArg>>,
}

impl<'a> Bytecode {
    pub fn get(&self, index: &(usize, usize)) -> Option<&Vec<BytecodeArg>> {
        if let Some(s) = self.bytecode.get(index.0) {
            return Some(s);
        }

        None
    }

    fn load_call(
        call: &'a FSRCall<'a>,
        var_map: &'a mut VarMap<'a>,
        is_attr: bool,
        const_map: &mut ConstTable,
    ) -> (Vec<BytecodeArg>, &'a mut VarMap<'a>) {
        let mut result = Vec::new();
        let mut var_map_ref = var_map;

        let name = call.get_name();
        if is_attr {
            if !var_map_ref.has_attr(name) {
                let v = name;
                var_map_ref.insert_attr(v);
            }
            let id = var_map_ref.get_attr(name).unwrap();
            result.push(BytecodeArg {
                operator: BytecodeOperator::Load,
                arg: ArgType::Attr(*id, name.to_string()),
            });
            result.push(BytecodeArg {
                operator: BytecodeOperator::BinaryDot,
                arg: ArgType::None,
            });
        } else {
            if !var_map_ref.has_var(name) {
                let v = name;
                var_map_ref.insert_var(v);
            }
            let id = var_map_ref.get_var(name).unwrap();
            result.push(BytecodeArg {
                operator: BytecodeOperator::Load,
                arg: ArgType::Variable(*id, name.to_string()),
            });
        }

        for arg in call.get_args() {
            let mut v = Self::load_token_with_map(arg, var_map_ref, const_map);
            var_map_ref = v.1;
            result.append(&mut v.0[0]);
            // result.push(BytecodeArg {
            //     operator: BytecodeOperator::InsertArg,
            //     arg: ArgType::None,
            // });
        }

        result.push(BytecodeArg {
            operator: BytecodeOperator::Call,
            arg: ArgType::CallArgsNumber(call.get_args().len()),
        });

        (result, var_map_ref)
    }

    fn load_variable(
        var: &'a FSRVariable<'a>,
        var_map: &'a mut VarMap<'a>,
        is_attr: bool,
    ) -> (Vec<BytecodeArg>, &'a mut VarMap<'a>) {
        if !var_map.has_var(var.get_name()) {
            let v = var.get_name();
            var_map.insert_var(v);
        }

        let arg_id = var_map.get_var(var.get_name()).unwrap();
        let op_arg = match is_attr {
            true => BytecodeArg {
                operator: BytecodeOperator::Load,
                arg: ArgType::Attr(*arg_id, var.get_name().to_string()),
            },
            false => BytecodeArg {
                operator: BytecodeOperator::Load,
                arg: ArgType::Variable(*arg_id, var.get_name().to_string()),
            },
        };

        let ans = vec![op_arg];

        (ans, var_map)
    }

    fn load_assign_arg(
        var: &'a FSRVariable<'a>,
        var_map: &'a mut VarMap<'a>,
    ) -> (Vec<BytecodeArg>, &'a mut VarMap<'a>) {
        if !var_map.has_var(var.get_name()) {
            let v = var.get_name();
            var_map.insert_var(v);
        }

        let arg_id = var_map.get_var(var.get_name()).unwrap();

        let op_arg = BytecodeArg {
            operator: BytecodeOperator::AssignArgs,
            arg: ArgType::Variable(*arg_id, var.get_name().to_string()),
        };

        let ans = vec![op_arg];

        (ans, var_map)
    }

    fn load_expr(
        expr: &'a FSRExpr<'a>,
        var_map: &'a mut VarMap<'a>,
        const_map: &mut ConstTable,
    ) -> (Vec<BytecodeArg>, &'a mut VarMap<'a>) {
        let mut op_code = Vec::new();
        let mut var_map_ref = Some(var_map);
        if let FSRToken::Expr(sub_expr) = expr.get_left() {
            let mut v = Self::load_expr(sub_expr, var_map_ref.unwrap(), const_map);
            op_code.append(&mut v.0);
            var_map_ref = Some(v.1);
        } else if let FSRToken::Variable(v) = expr.get_left() {
            let mut v = Self::load_variable(v, var_map_ref.unwrap(), false);
            op_code.append(&mut v.0);
            var_map_ref = Some(v.1);
        } else if let FSRToken::Call(c) = expr.get_left() {
            let mut v = Self::load_call(c, var_map_ref.unwrap(), false, const_map);
            op_code.append(&mut v.0);
            var_map_ref = Some(v.1);
        } else if let FSRToken::Constant(c) = expr.get_left() {
            let mut v = Self::load_constant(c, var_map_ref.unwrap(), const_map);
            op_code.append(&mut v.0);
            var_map_ref = Some(v.1);
        } else {
            println!("{:#?}", expr.get_left());
            unimplemented!()
        }

        let mut second = Vec::new();
        if let FSRToken::Expr(sub_expr) = expr.get_right() {
            let mut v = Self::load_expr(sub_expr, var_map_ref.unwrap(), const_map);
            second.append(&mut v.0);
            var_map_ref = Some(v.1);
        } else if let FSRToken::Variable(v) = expr.get_right() {
            let mut is_attr = false;
            if expr.get_op().eq(".") {
                is_attr = true;
            }
            let mut v = Self::load_variable(v, var_map_ref.unwrap(), is_attr);
            second.append(&mut v.0);
            var_map_ref = Some(v.1);
        } else if let FSRToken::Call(c) = expr.get_right() {
            let mut is_attr = false;
            if expr.get_op().eq(".") {
                is_attr = true;
            }
            let mut v = Self::load_call(c, var_map_ref.unwrap(), is_attr, const_map);
            second.append(&mut v.0);
            op_code.append(&mut second);
            var_map_ref = Some(v.1);
            //call special process
            if expr.get_op().eq(".") {
                return (op_code, var_map_ref.unwrap());
            }
        } else if let FSRToken::Constant(c) = expr.get_right() {
            let mut v = Self::load_constant(c, var_map_ref.unwrap(), const_map);
            second.append(&mut v.0);
            var_map_ref = Some(v.1);
        }
        if expr.get_op().eq("&&") {
            op_code.push(BytecodeArg {
                operator: BytecodeOperator::AndJump,
                arg: ArgType::AddOffset(second.len()),
            });
        } else if expr.get_op().eq("||") {
            op_code.push(BytecodeArg {
                operator: BytecodeOperator::OrJump,
                arg: ArgType::AddOffset(second.len()),
            });
        }

        op_code.append(&mut second);
        if let Some(s) = BytecodeOperator::get_op(expr.get_op()) {
            op_code.push(s);
        } else {
            unimplemented!()
        }

        (op_code, var_map_ref.unwrap())
    }

    fn load_block(
        block: &'a FSRBlock<'a>,
        var_map: &'a mut VarMap<'a>,
        const_map: &mut ConstTable,
    ) -> (Vec<Vec<BytecodeArg>>, &'a mut VarMap<'a>) {
        let mut vs = vec![];
        let mut ref_self = var_map;
        for token in block.get_tokens() {
            let lines = Self::load_token_with_map(token, ref_self, const_map);
            ref_self = lines.1;
            let lines = lines.0;
            for line in lines {
                vs.push(line);
            }
        }

        if vs.is_empty() {
            vs.push(vec![BytecodeArg {
                operator: BytecodeOperator::Empty,
                arg: ArgType::None,
            }]);
        }

        (vs, ref_self)
    }

    fn load_if_def(
        if_def: &'a FSRIf<'a>,
        var_map: &'a mut VarMap<'a>,
        const_map: &mut ConstTable,
    ) -> (Vec<Vec<BytecodeArg>>, &'a mut VarMap<'a>) {
        let mut var_ref = var_map;
        let test_exp = if_def.get_test();
        let mut vs = vec![];
        let mut v = Self::load_token_with_map(test_exp, var_ref, const_map);
        var_ref = v.1;
        let mut test_list = Vec::new();
        let mut t = v.0.remove(0);
        test_list.append(&mut t);

        let block_items = Self::load_block(if_def.get_block(), var_ref, const_map);
        var_ref = block_items.1;
        let mut count_elses = 0;
        if let Some(s) = if_def.get_elses() {
            count_elses = s.get_elses().len();
        }
        test_list.push(BytecodeArg {
            operator: BytecodeOperator::IfTest,
            arg: ArgType::IfTestNext((block_items.0.len() as u64, count_elses as u64)),
        });
        vs.push(test_list);
        vs.extend(block_items.0);
        if let Some(s) = if_def.get_elses() {
            for e in s.get_elses() {
                let test_exp = e.get_test();

                let mut test_list = Vec::new();
                if let Some(t) = test_exp {
                    let block = e.get_block();
                    let block_items = Self::load_block(block, var_ref, const_map);
                    var_ref = block_items.1;
                    test_list.push(BytecodeArg {
                        operator: BytecodeOperator::ElseIf,
                        arg: ArgType::IfTestNext((block_items.0.len() as u64, 0)),
                    });
                    let mut v = Self::load_token_with_map(t, var_ref, const_map);
                    var_ref = v.1;
                    let mut t = v.0.remove(0);
                    test_list.append(&mut t);
                    test_list.push(BytecodeArg {
                        operator: BytecodeOperator::ElseIfTest,
                        arg: ArgType::IfTestNext((block_items.0.len() as u64, 0)),
                    });
                    vs.push(test_list);
                    vs.extend(block_items.0);
                } else {
                    let block = e.get_block();
                    let block_items = Self::load_block(block, var_ref, const_map);
                    test_list.push(BytecodeArg {
                        operator: BytecodeOperator::Else,
                        arg: ArgType::IfTestNext((block_items.0.len() as u64, 0)),
                    });

                    var_ref = block_items.1;

                    vs.push(test_list);
                    vs.extend(block_items.0);
                }
            }
        }

        let end_if = vec![BytecodeArg {
            operator: BytecodeOperator::IfBlockEnd,
            arg: ArgType::None,
        }];
        vs.push(end_if);
        (vs, var_ref)
    }

    #[allow(unused)]
    fn load_for_def(
        for_def: &'a FSRFor<'a>,
        var_map: &'a mut VarMap<'a>,
        const_map: &mut ConstTable,
    ) -> (Vec<Vec<BytecodeArg>>, &'a mut VarMap<'a>) {
        let mut result = vec![];

        let mut var_self = var_map;
        let v = Self::load_token_with_map(for_def.get_expr(), var_self, const_map);
        let mut expr = v.0;
        var_self = v.1;
        let mut t = expr.remove(0);
        if !var_self.has_attr("__iter__") {
            var_self.insert_attr("__iter__");
        }
        let id = var_self.get_attr("__iter__").unwrap();
        t.push(BytecodeArg {
            operator: BytecodeOperator::Load,
            arg: ArgType::Attr(*id, "__iter__".to_string()),
        });
        t.push(BytecodeArg {
            operator: BytecodeOperator::BinaryDot,
            arg: ArgType::None,
        });
        t.push(BytecodeArg {
            operator: BytecodeOperator::Call,
            arg: ArgType::CallArgsNumber(0),
        });
        let mut block_items = Self::load_block(for_def.get_block(), var_self, const_map);
        var_self = block_items.1;
        t.push(BytecodeArg {
            operator: BytecodeOperator::LoadForIter,
            arg: ArgType::ForLine(block_items.0.len() as u64 + 3),
        });
        result.push(t);

        let mut load_next = Vec::new();
        if !var_self.has_attr("__next__") {
            var_self.insert_attr("__next__");
        }
        let id = var_self.get_attr("__next__").unwrap();
        load_next.push(BytecodeArg {
            operator: BytecodeOperator::SpecialLoadFor,
            arg: ArgType::None,
        });
        load_next.push(BytecodeArg {
            operator: BytecodeOperator::Load,
            arg: ArgType::Attr(*id, "__next__".to_string()),
        });
        load_next.push(BytecodeArg {
            operator: BytecodeOperator::BinaryDot,
            arg: ArgType::None,
        });
        load_next.push(BytecodeArg {
            operator: BytecodeOperator::Call,
            arg: ArgType::CallArgsNumber(0),
        });
        load_next.push(BytecodeArg {
            operator: BytecodeOperator::PushForNext,
            arg: ArgType::None,
        });
        if !var_self.has_var(for_def.get_var_name()) {
            var_self.insert_var(for_def.get_var_name());
        }

        let arg_id = var_self.get_var(for_def.get_var_name()).unwrap();
        load_next.push(BytecodeArg {
            operator: BytecodeOperator::Load,
            arg: ArgType::Variable(*arg_id, for_def.get_var_name().to_string()),
        });
        load_next.push(BytecodeArg {
            operator: BytecodeOperator::Assign,
            arg: ArgType::None,
        });

        result.push(load_next);
        result.append(&mut block_items.0);
        let end = vec![BytecodeArg {
            operator: BytecodeOperator::ForBlockEnd,
            arg: ArgType::ForEnd(result.len() as i64 - 1),
        }];

        result.push(end);
        (result, var_self)
    }

    fn load_while_def(
        while_def: &'a FSRWhile<'a>,
        var_map: &'a mut VarMap<'a>,
        const_map: &mut ConstTable,
    ) -> (Vec<Vec<BytecodeArg>>, &'a mut VarMap<'a>) {
        let test_exp = while_def.get_test();
        let mut vs = vec![];
        let mut v = Self::load_token_with_map(test_exp, var_map, const_map);
        let mut test_list = Vec::new();
        let mut t = v.0.remove(0);
        test_list.append(&mut t);

        let block_items = Self::load_block(while_def.get_block(), v.1, const_map);
        test_list.push(BytecodeArg {
            operator: BytecodeOperator::WhileTest,
            arg: ArgType::WhileTest(block_items.0.len() as u64 + 1),
        });
        vs.push(test_list);
        let len = block_items.0.len();
        //let l = block_items.0.get_mut(len - 1).unwrap();
        let end = BytecodeArg {
            operator: BytecodeOperator::WhileBlockEnd,
            arg: ArgType::WhileEnd(len as i64 + 1),
        };
        vs.extend(block_items.0);
        vs.push(vec![end]);
        (vs, block_items.1)
    }

    fn load_break() -> Vec<BytecodeArg> {
        let break_list = vec![BytecodeArg {
            operator: BytecodeOperator::Break,
            arg: ArgType::None,
        }];
        break_list
    }

    fn load_continue() -> Vec<BytecodeArg> {
        let continue_list = vec![BytecodeArg {
            operator: BytecodeOperator::Continue,
            arg: ArgType::None,
        }];
        continue_list
    }

    fn load_import(import: &'a FSRImport, var_map: &'a mut VarMap<'a>) -> (Vec<Vec<BytecodeArg>>, &'a mut VarMap<'a>) {
        let name = import.module_name.last().unwrap();
        if !var_map.has_var(name) {
            let v = name;
            var_map.insert_var(name);
        }

        let id = var_map.get_var(name).unwrap();
        let import_list = vec![
            BytecodeArg {
                operator: BytecodeOperator::Import,
                arg: ArgType::ImportModule(*id, import.module_name.iter().map(|x| x.to_string()).collect())
            },
        ];

        (vec![import_list], var_map)
    }

    fn load_token_with_map(
        token: &'a FSRToken<'a>,
        var_map: &'a mut VarMap<'a>,
        const_map: &mut ConstTable,
    ) -> (Vec<Vec<BytecodeArg>>, &'a mut VarMap<'a>) {
        if let FSRToken::Expr(expr) = token {
            let v = Self::load_expr(expr, var_map, const_map);
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
                let lines = Self::load_token_with_map(token, ref_self, const_map);
                ref_self = lines.1;
                let lines = lines.0;
                for line in lines {
                    vs.push(line);
                }
            }

            return (vs, ref_self);
        } else if let FSRToken::IfExp(if_def) = token {
            let v = Self::load_if_def(if_def, var_map, const_map);

            return (v.0, v.1);
        } else if let FSRToken::Assign(assign) = token {
            let v = Self::load_assign(assign, var_map, const_map);
            return (vec![v.0], v.1);
        } else if let FSRToken::WhileExp(while_def) = token {
            let v = Self::load_while_def(while_def, var_map, const_map);
            return (v.0, v.1);
        } else if let FSRToken::Block(block) = token {
            let v = Self::load_block(block, var_map, const_map);
            return (v.0, v.1);
        } else if let FSRToken::Call(call) = token {
            let v = Self::load_call(call, var_map, false, const_map);
            return (vec![v.0], v.1);
        } else if let FSRToken::Constant(c) = token {
            let v = Self::load_constant(c, var_map, const_map);
            return (vec![v.0], v.1);
        } else if let FSRToken::FunctionDef(fn_def) = token {
            let v = Self::load_function(fn_def, var_map, const_map);
            return (v.0, v.1);
        } else if let FSRToken::Class(cls) = token {
            let v = Self::load_class(cls, var_map, const_map);
            return (v.0, v.1);
        } else if let FSRToken::Return(ret) = token {
            let v = Self::load_ret(ret, var_map, const_map);
            return (vec![v.0], v.1);
        } else if let FSRToken::List(list) = token {
            let v = Self::load_list(list, var_map, const_map);
            return (vec![v.0], v.1);
        } else if let FSRToken::Break(_) = token {
            let v = Self::load_break();
            return (vec![v], var_map);
        } else if let FSRToken::Continue(_) = token {
            let v = Self::load_continue();
            return (vec![v], var_map);
        } else if let FSRToken::ForBlock(b) = token {
            let v = Self::load_for_def(b, var_map, const_map);
            return (v.0, v.1);
        } else if let FSRToken::Import(import) = token {
            let v = Self::load_import(import, var_map);
            return (v.0, v.1);
        }

        unimplemented!()
    }

    fn load_assign(
        token: &'a FSRAssign<'a>,
        var_map: &'a mut VarMap<'a>,
        const_map: &mut ConstTable,
    ) -> (Vec<BytecodeArg>, &'a mut VarMap<'a>) {
        let mut result_list = Vec::new();
        if let FSRToken::Variable(v) = &**token.get_left() {
            let mut right = Self::load_token_with_map(token.get_assign_expr(), var_map, const_map);
            result_list.append(&mut right.0[0]);
            right.1.insert_var(v.get_name());
            let id = right.1.get_var(v.get_name()).unwrap();
            result_list.push(BytecodeArg {
                operator: BytecodeOperator::Assign,
                arg: ArgType::Variable(*id, v.get_name().to_string()),
            });
            (result_list, right.1)
        } else {
            let mut left = Self::load_token_with_map(token.get_left(), var_map, const_map);
            let mut right = Self::load_token_with_map(token.get_assign_expr(), left.1, const_map);
            result_list.append(&mut right.0[0]);
            result_list.append(&mut left.0[0]);
            result_list.push(BytecodeArg {
                operator: BytecodeOperator::Assign,
                arg: ArgType::None,
            });
            (result_list, right.1)
        }
    }

    fn load_constant(
        token: &'a FSRConstant,
        var_map: &'a mut VarMap<'a>,
        const_map: &mut ConstTable,
    ) -> (Vec<BytecodeArg>, &'a mut VarMap<'a>) {
        let c = token.get_const_str();
        if !const_map.const_map.contains_key(&c.to_2()) {
            let r = if const_map.const_map.is_empty() {
                1
            } else {
                *const_map.const_map.values().max().unwrap() + 1
            };
            const_map.const_map.insert(c.to_2(), r);
        }
        let id = *const_map.const_map.get(&c.to_2()).unwrap();

        let mut result_list = Vec::new();
        if let FSRConstantType::Integer(i) = token.get_constant() {
            let obj = FSRInteger::new_inst(*i);
            obj.set_not_delete();
            let ptr = FSRVM::leak_object(Box::new(obj));
            const_map.insert(id as usize, ptr);
            result_list.push(BytecodeArg {
                operator: BytecodeOperator::Load,
                arg: ArgType::ConstInteger(id, *i),
            });
        } else if let FSRConstantType::String(s) = token.get_constant() {
            let obj = FSRString::new_inst(Cow::Owned(String::from_utf8_lossy(s).to_string()));
            obj.set_not_delete();
            let ptr = FSRVM::leak_object(Box::new(obj));
            const_map.insert(id as usize, ptr);
            result_list.push(BytecodeArg {
                operator: BytecodeOperator::Load,
                arg: ArgType::ConstString(id, String::from_utf8_lossy(s).to_string()),
            });
        }

        (result_list, var_map)
    }

    fn load_list(
        token: &'a FSRListFrontEnd,
        var_map: &'a mut VarMap<'a>,
        const_map: &mut ConstTable,
    ) -> (Vec<BytecodeArg>, &'a mut VarMap<'a>) {
        let mut result_list = Vec::new();
        let mut self_var = var_map;
        for sub_t in token.get_items().iter().rev() {
            let v = Bytecode::load_token_with_map(sub_t, self_var, const_map);
            let mut expr = v.0;
            self_var = v.1;
            result_list.append(&mut expr[0]);
            let load = BytecodeArg {
                operator: BytecodeOperator::Load,
                arg: ArgType::None,
            };
            result_list.push(load);
        }

        let load_list = BytecodeArg {
            operator: BytecodeOperator::LoadList,
            arg: ArgType::LoadListNumber(token.get_items().len()),
        };
        result_list.push(load_list);
        (result_list, self_var)
    }

    fn load_ret(
        ret: &'a FSRReturn,
        var_map: &'a mut VarMap<'a>,
        const_map: &mut ConstTable,
    ) -> (Vec<BytecodeArg>, &'a mut VarMap<'a>) {
        let v = Self::load_token_with_map(ret.get_return_expr(), var_map, const_map);
        let mut ret_expr = Vec::new();
        let mut r = v.0;
        if !r.is_empty() {
            ret_expr.append(&mut r[0]);
        }
        ret_expr.push(BytecodeArg {
            operator: BytecodeOperator::ReturnValue,
            arg: ArgType::None,
        });

        (ret_expr, v.1)
    }

    fn load_function(
        fn_def: &'a FSRFnDef<'a>,
        var_map: &'a mut VarMap<'a>,
        const_map: &mut ConstTable,
    ) -> (Vec<Vec<BytecodeArg>>, &'a mut VarMap<'a>) {
        let mut result = vec![];
        let name = fn_def.get_name();
        //let mut define_fn = Vec::new();
        if !var_map.has_var(name) {
            var_map.insert_var(name);
        }

        let mut fn_var_map = VarMap::new();
        let mut fn_var_map_ref = &mut fn_var_map;
        let args = fn_def.get_args();
        let mut args_load = Vec::new();
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
            arg: ArgType::Variable(*arg_id, name.to_string()),
        };

        let body = fn_def.get_body();
        let v = Self::load_block(body, fn_var_map_ref, const_map);
        fn_var_map_ref = v.1;
        let fn_body = v.0;
        // let mut end_list = Vec::new();
        // end_list.push(BytecodeArg {
        //     operator: BytecodeOperator::EndDefineFn,
        //     arg: ArgType::None,
        // });
        //define_fn.push(op_arg);
        let mut load_args = Vec::new();
        for arg in args {
            if let FSRToken::Variable(v) = arg {
                let mut a = Self::load_assign_arg(v, fn_var_map_ref);
                fn_var_map_ref = a.1;
                load_args.append(&mut a.0);
            }
        }

        args_load.push(op_arg);
        args_load.push(BytecodeArg {
            operator: BytecodeOperator::DefineFn,
            arg: ArgType::DefineFnArgs(fn_body.len() as u64 + 1, arg_len),
        });
        // define_fn.push(BytecodeArg {
        //     operator: BytecodeOperator::DefineFn,
        //     arg: ArgType::DefineFnArgs(fn_body.len() as u64 + 1, arg_len),
        // });

        result.push(args_load);
        result.push(load_args);
        //result.push(define_fn);
        if !fn_body.is_empty() {
            result.extend(fn_body);
        }

        let end_of_fn = vec![BytecodeArg {
            operator: BytecodeOperator::EndDefineFn,
            arg: ArgType::None,
        }];
        result.push(end_of_fn);
        // result.push(end_list);
        (result, var_map)
    }

    fn load_class(
        class_def: &'a FSRClassFrontEnd<'a>,
        var_map: &'a mut VarMap<'a>,
        const_map: &mut ConstTable,
    ) -> (Vec<Vec<BytecodeArg>>, &'a mut VarMap<'a>) {
        let mut result = vec![];
        let name = class_def.get_name();
        if !var_map.has_var(name) {
            let v = name;
            var_map.insert_var(v);
        }
        let arg_id = var_map.get_var(name).unwrap();

        let op_arg = BytecodeArg {
            operator: BytecodeOperator::Load,
            arg: ArgType::Variable(*arg_id, name.to_string()),
        };

        let mut class_var_map = VarMap::new();
        let v = Self::load_block(class_def.get_block(), &mut class_var_map, const_map);

        let ans = vec![
            op_arg,
            BytecodeArg {
                operator: BytecodeOperator::ClassDef,
                arg: ArgType::DefineClassLine(v.0.len() as u64),
            },
        ];

        result.push(ans);
        result.extend(v.0);
        let end_of_cls = vec![BytecodeArg {
            operator: BytecodeOperator::EndDefineClass,
            arg: ArgType::None,
        }];
        result.push(end_of_cls);
        (result, var_map)
    }

    fn load_isolate_block(
        token: &FSRToken<'a>,
        const_map: &mut ConstTable,
    ) -> Vec<Vec<BytecodeArg>> {
        let mut var_map = VarMap::new();
        let v = Self::load_token_with_map(token, &mut var_map, const_map);
        v.0
    }

    pub fn load_ast(name: &str, token: FSRToken<'a>) -> Bytecode {
        let mut const_table = ConstTable::new();
        let vs = Self::load_isolate_block(&token, &mut const_table);
        let mut result = vec![];
        for v in vs {
            let single_line = Vec::from_iter(v);
            result.push(single_line);
        }
        Self {
            bytecode: result,
            name: name.to_string(),
            const_table,
        }
    }

    pub fn compile(name: &str, code: &str) -> Bytecode {
        let meta = FSRPosition::new();
        let token = FSRModuleFrontEnd::parse(code.as_bytes(), meta).unwrap();
        return Self::load_ast(name, FSRToken::Module(token));
    }
}
