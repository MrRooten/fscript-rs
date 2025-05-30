#![allow(unused)]

use std::rc::Rc;
use std::{cmp::Ordering, fmt::Display};

use crate::frontend::ast::token;
use crate::frontend::ast::token::assign::FSRAssign;
use crate::frontend::ast::token::function_def::FSRFnDef;
use crate::frontend::ast::token::list::FSRListFrontEnd;
use crate::frontend::ast::token::slice::FSRGetter;
use crate::frontend::ast::{parse::ASTParser, token::constant::FSRConstant};
use crate::utils::error::{SyntaxErrType, SyntaxError};
use std::str;

use super::base::{FSRPosition, FSRType};
use super::ASTContext;
use super::{base::FSRToken, call::FSRCall, variable::FSRVariable};

static mut LAMBDA_NUMBER: i32 = 0;

#[derive(Debug, Clone)]
pub struct FSRExpr {
    pub(crate) single_op: Option<SingleOp>,
    pub(crate) left: Box<FSRToken>,
    pub(crate) right: Box<FSRToken>,
    pub(crate) op: Option<&'static str>,
    pub(crate) len: usize,
    pub(crate) meta: FSRPosition,
}

impl FSRExpr {
    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }

    pub fn get_left(&self) -> &FSRToken {
        &self.left
    }
}

impl Display for FSRExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(PartialEq, Copy, Clone)]
enum ExprState {
    ExprStart,
    EscapeNewline,
    EscapeChar,
    DoubleString,
    SingleString,
    Number,
    Float,
    EndToken,
    Function,
    Slice,
    Operator,
    Variable,
    Bracket,
    Square,
    WaitToken,
    SquareBracket,
    Comment,
}

struct ExprStates {
    states: Vec<ExprState>,
}

impl ExprStates {
    pub fn new() -> Self {
        Self { states: vec![] }
    }

    pub fn set_up_state(&mut self, new_state: ExprState) {
        self.states.pop();
        self.states.push(new_state);
    }

    pub fn push_state(&mut self, state: ExprState) {
        self.states.push(state);
    }

    pub fn pop_state(&mut self) {
        self.states.pop();
    }

    pub fn peek(&self) -> &ExprState {
        &self.states[self.states.len() - 1]
    }

    pub fn eq_peek(&self, state: &ExprState) -> bool {
        self.peek().eq(state)
    }

    pub fn is_string(&self) -> bool {
        self.peek().eq(&ExprState::DoubleString) || self.peek().eq(&ExprState::SingleString)
    }
}

enum Operator {
    Add,
    Sub,
    Div,
    Mul,
    LeftShift,
    RightShift,
    Xor,
    Or,
    And,
    Greater,
    Less,
}

#[derive(Debug)]
struct Node<'a> {
    op: &'a str,
    left: Option<Box<Node<'a>>>,
    right: Option<Box<Node<'a>>>,
    value: Option<FSRToken>,
    is_leaf: bool,
}

#[derive(Debug)]
struct TreeNode<'a> {
    node: Option<Node<'a>>,
    head: *mut Node<'a>,
    last: *mut Node<'a>,
}

impl<'a> Node<'a> {
    pub fn from_value(value: FSRToken) -> Node<'a> {
        Self {
            op: "",
            left: None,
            right: None,
            value: Some(value),
            is_leaf: true,
        }
    }

    pub fn from_op(op: &'a str) -> Node<'a> {
        Self {
            op,
            left: None,
            right: None,
            value: None,
            is_leaf: false,
        }
    }

    pub fn get_single_op_level(op: &SingleOp) -> i32 {
        if op.eq(&SingleOp::Not) {
            return 1;
        }

        if op.eq(&SingleOp::Minus) {
            return 2;
        }

        // if op.eq("~") {
        //     return 2;
        // }

        -1
    }

    pub fn get_op_level(op: &str) -> i32 {
        if op.eq("..") {
            return 0;
        }

        if op.eq("-") || op.eq("+") {
            return 1;
        }

        if op.eq("*") || op.eq("/") || op.eq("%") {
            return 2;
        }

        if op.eq(">>") || op.eq("<<") {
            return 2;
        }

        if op.eq(".") || op.eq("::") {
            return 3;
        }

        if op.eq(">") || op.eq("<") || op.eq("==") || op.eq("!=") {
            return 0;
        }

        if op.eq("&&") || op.eq("||") {
            return -2;
        }

        if op.eq("and") || op.eq("or") {
            return -2;
        }

        if op.eq("=") {
            return -3;
        }

        if op.eq(":") {
            return 4;
        }

        if op.eq(",") {
            return -5;
        }

        -1
    }

    pub fn is_higher_priority(op1: &str, op2: &str) -> Ordering {
        let op1 = Node::get_op_level(op1);
        let op2 = Node::get_op_level(op2);

        op1.cmp(&op2)
    }

    pub fn two_nodes_add(node1: Node<'a>, node2: Node<'a>, op: &'a str) -> Box<Node<'a>> {
        unimplemented!()
    }
}

type FSROpreatorTree<'a> = Node<'a>;

impl FSROpreatorTree<'_> {}

#[derive(Debug)]
pub enum FSRBinOpResult {
    BinOp(FSRExpr),
    Constant(FSRConstant),
}

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
pub enum SingleOp {
    Not,
    Minus,
}


struct StmtContext {
    states: ExprStates,
    start: usize,
    length: usize,
    bracket_count: i32,
    candidates: Vec<FSRToken>,
    operators: Vec<(&'static str, usize)>,
    single_op: Option<SingleOp>,
    last_loop: bool,
    single_op_level: Option<i32>,
}

impl StmtContext {
    pub fn new() -> Self {
        let mut states = ExprStates::new();
        states.push_state(ExprState::WaitToken);
        Self {
            states,
            start: 0,
            length: 0,
            bracket_count: 0,
            candidates: vec![],
            operators: vec![],
            single_op: None,
            last_loop: false,
            single_op_level: None,
        }
    }
}

impl FSRExpr {
    pub fn get_single_op(&self) -> Option<SingleOp> {
        self.single_op
    }

    pub fn get_len(&self) -> usize {
        self.len
    }

    pub fn is_single_op(ps: &str) -> bool {
        if ps.eq("!") {
            return true;
        }

        if ps.eq("-") {
            return true;
        }

        if ps.eq("not") {
            return true;
        }

        false
    }

    pub fn expr_split_by_comma(&self) -> Vec<FSRToken> {
        unimplemented!()
    }

    pub fn get_right(&self) -> &FSRToken {
        &self.right
    }

    pub fn is_op_one_char(op: char) -> bool {
        if op == '+'
            || op == '-'
            || op == '='
            || op == '>'
            || op == '<'
            || op == '*'
            || op == '.'
            || op == ','
            || op == '&'
            || op == '|'
            || op == '!'
            || op == '/'
            || op == ':'
            || op == '%'
        {
            return true;
        }

        false
    }

    #[inline]
    fn double_quote_loop(
        source: &[u8],
        ignore_nline: bool,
        meta: &FSRPosition,
        ctx: &mut StmtContext,
    ) -> Result<(), SyntaxError> {
        if let Some(s_op) = ctx.single_op {
            let mut sub_meta = meta.from_offset(ctx.start);
            return Err(SyntaxError::new(
                &sub_meta,
                format!("{:?} can not follow string", s_op),
            ));
        }
        ctx.start += 1;
        loop {
            if ctx.start + ctx.length >= source.len() {
                let mut sub_meta = meta.from_offset(ctx.start);
                let err = SyntaxError::new_with_type(
                    &sub_meta,
                    "Not Close for Double Quote",
                    SyntaxErrType::QuoteNotClose,
                );
                return Err(err);
            }
            let c = source[ctx.start + ctx.length] as char;
            if ctx.states.eq_peek(&ExprState::EscapeChar) {
                ctx.states.pop_state();
                ctx.length += 1;
                continue;
            }

            if c == '\"' {
                break;
            }

            if c == '\\' {
                ctx.states.push_state(ExprState::EscapeChar);
            }

            ctx.length += 1;
        }

        let s = &source[ctx.start..ctx.start + ctx.length];
        let mut sub_meta = meta.from_offset(ctx.start);
        let constant = FSRConstant::from_str(s, sub_meta);
        ctx.candidates.push(FSRToken::Constant(constant));
        ctx.length += 1;
        ctx.start += ctx.length;
        ctx.length = 0;
        Ok(())
    }

    #[inline]
    fn single_quote_loop(
        source: &[u8],
        ignore_nline: bool,
        meta: &FSRPosition,
        ctx: &mut StmtContext,
    ) -> Result<(), SyntaxError> {
        if let Some(s_op) = ctx.single_op {
            let mut sub_meta = meta.from_offset(ctx.start);
            return Err(SyntaxError::new(
                &sub_meta,
                format!("{:?} can not follow string", s_op),
            ));
        }
        ctx.start += 1;

        loop {
            if ctx.start + ctx.length >= source.len() {
                let mut sub_meta = meta.from_offset(ctx.start);
                let err = SyntaxError::new_with_type(
                    &sub_meta,
                    "Not Close for Single Quote",
                    SyntaxErrType::QuoteNotClose,
                );
                return Err(err);
            }
            let c = source[ctx.start + ctx.length] as char;
            if ctx.states.eq_peek(&ExprState::EscapeChar) {
                ctx.states.pop_state();
                ctx.length += 1;
                continue;
            }

            if c == '\'' {
                break;
            }

            if c == '\\' {
                ctx.states.push_state(ExprState::EscapeChar);
            }

            ctx.length += 1;
        }

        let s = &source[ctx.start..ctx.start + ctx.length];
        let mut sub_meta = meta.from_offset(ctx.start);
        let constant = FSRConstant::from_str(s, sub_meta);
        ctx.candidates.push(FSRToken::Constant(constant));
        ctx.length += 1;
        ctx.start += ctx.length;
        ctx.length = 0;

        Ok(())
    }

    fn end_of_operator(
        source: &[u8],
        ignore_nline: bool,
        meta: &FSRPosition,
        ctx: &mut StmtContext,
    ) -> Result<(), SyntaxError> {
        let op = str::from_utf8(&source[ctx.start..ctx.start + ctx.length]).unwrap();
        let op = ASTParser::get_static_op(op);
        if ctx.start + ctx.length >= source.len() {
            let mut sub_meta = meta.from_offset(ctx.start);
            return Err(SyntaxError::new(
                &sub_meta,
                format!("{} must follow a expr or variable", op),
            ));
        }

        if op.eq("-") && (source[ctx.start + ctx.length] as char).is_ascii_digit() {
            ctx.single_op = Some(SingleOp::Minus);
            ctx.states.pop_state();
            ctx.start += ctx.length;
            ctx.length = 0;
            return Ok(());
        }

        if Self::is_single_op(op) && !op.eq("-") {
            if ctx.single_op.is_some()
                && (ctx.single_op.unwrap().eq(&SingleOp::Not))
            {
                ctx.single_op = None;
            } else {
                ctx.single_op = Some(SingleOp::Not);
            }

            ctx.states.pop_state();
            ctx.start += ctx.length;
            ctx.length = 0;
        } else {
            if ctx.single_op.is_some()
                && Node::get_single_op_level(ctx.single_op.as_ref().unwrap())
                    > Node::get_op_level(op)
            {
                ctx.candidates[0].set_single_op(ctx.single_op.unwrap());
                ctx.single_op = None;
            }
            ctx.operators.push((op, ctx.start));
            ctx.states.pop_state();
            ctx.start += ctx.length;
            ctx.length = 0;
        }
        Ok(())
    }

    #[inline]
    fn process_operator(
        source: &[u8],
        ignore_nline: bool,
        meta: &FSRPosition,
        ctx: &mut StmtContext,
    ) -> Result<(), SyntaxError> {
        let op = str::from_utf8(&source[ctx.start..ctx.start + ctx.length]).unwrap();
        let op = ASTParser::get_static_op(op);
        if ctx.start + ctx.length >= source.len() {
            let mut sub_meta = meta.from_offset(ctx.start);
            return Err(SyntaxError::new(
                &sub_meta,
                format!("{} must follow a expr or variable", op),
            ));
        }

        if op.eq("-") && (source[ctx.start + ctx.length] as char).is_ascii_digit() {
            ctx.single_op = Some(SingleOp::Minus);
            ctx.states.pop_state();
            ctx.start += ctx.length;
            ctx.length = 0;
            return Ok(());
        }

        if Self::is_single_op(op) && !op.eq("-") {
            if ctx.single_op.is_some()
                && ctx.single_op.unwrap().eq(&SingleOp::Not)
            {
                ctx.single_op = None;
            } else {
                ctx.single_op = Some(SingleOp::Not);
            }

            ctx.states.pop_state();
            ctx.start += ctx.length;
            ctx.length = 0;
        } else {
            if ctx.single_op.is_some()
                && Node::get_single_op_level(ctx.single_op.as_ref().unwrap())
                    < Node::get_op_level(op)
            {
                println!("sdfsdf");
            }
            ctx.operators.push((op, ctx.start));
            ctx.states.pop_state();
            ctx.start += ctx.length;
            ctx.length = 0;
        }
        Ok(())
    }

    #[inline]
    fn end_of_bracket(
        source: &[u8],
        ignore_nline: bool,
        meta: &FSRPosition,
        ctx: &mut StmtContext,
        context: &mut ASTContext,
    ) -> Result<(), SyntaxError> {
        ctx.states.pop_state();
        ctx.bracket_count -= 1;

        if ctx.bracket_count > 0 {
            ctx.length += 1;
            return Ok(());
        } else {
            let _ps = &source[ctx.start..ctx.start + ctx.length];
            let ps = str::from_utf8(_ps).unwrap();

            ctx.start += ctx.length;
            ctx.length = 0;
            let sub_meta = meta.from_offset(0);
            let mut sub_expr = FSRExpr::parse(_ps, true, sub_meta, context)?.0;
            if let FSRToken::Expr(e) = &mut sub_expr {
                e.single_op = ctx.single_op;
            }
            if let FSRToken::Call(c) = &mut sub_expr {
                c.single_op = ctx.single_op;
            }

            if let FSRToken::Variable(v) = &mut sub_expr {
                v.single_op = ctx.single_op;
            }

            ctx.single_op = None;
            ctx.start += 1;
            ctx.candidates.push(sub_expr);
        }
        Ok(())
    }

    fn end_of_square_bracket(
        source: &[u8],
        ignore_nline: bool,
        meta: &FSRPosition,
        ctx: &mut StmtContext,
        context: &mut ASTContext,
    ) -> Result<(), SyntaxError> {
        ctx.states.pop_state();
        ctx.bracket_count -= 1;

        if ctx.bracket_count > 0 {
            ctx.length += 1;
            return Ok(());
        } else {
            let _ps = &source[ctx.start..ctx.start + ctx.length];
            let ps = str::from_utf8(_ps).unwrap();

            ctx.start += ctx.length;
            ctx.length = 0;
            let sub_meta = meta.from_offset(0);
            let mut sub_expr = FSRExpr::parse(_ps, true, sub_meta, context)?.0;
            if let FSRToken::Expr(e) = &mut sub_expr {
                e.single_op = ctx.single_op;
            }
            if let FSRToken::Call(c) = &mut sub_expr {
                if context.is_variable_defined_in_curr(c.get_name()) {
                    c.is_defined = true;
                } else {
                    context.ref_variable(c.get_name());
                }
                c.single_op = ctx.single_op;
            }

            if let FSRToken::Variable(v) = &mut sub_expr {
                if context.is_variable_defined_in_curr(v.get_name()) {
                    v.is_defined = true;
                } else {
                    context.ref_variable(v.get_name());
                }
                v.single_op = ctx.single_op;
            }

            ctx.single_op = None;
            ctx.start += 1;
            ctx.candidates.push(sub_expr);
        }
        Ok(())
    }

    #[inline]
    fn variable_process(
        source: &[u8],
        ignore_nline: bool,
        meta: &FSRPosition,
        ctx: &mut StmtContext,
        context: &mut ASTContext,
    ) -> Result<(), SyntaxError> {
        ctx.states.push_state(ExprState::Variable);
        loop {
            if ctx.start + ctx.length >= source.len() {
                break;
            }
            let c = source[ctx.start + ctx.length] as char;
            if !ASTParser::is_name_letter(c as u8) {
                break;
            }

            ctx.length += 1;
        }

        if ctx.start + ctx.length >= source.len()
            || (source[ctx.start + ctx.length] != b'(' && source[ctx.start + ctx.length] != b'[')
        {
            let name = str::from_utf8(&source[ctx.start..ctx.start + ctx.length]).unwrap().to_string();
            if name.eq("and") || name.eq("or") || name.eq("not") {
                if name.eq("not") {
                    ctx.single_op_level = Some(Node::get_single_op_level(&SingleOp::Not));
                    ctx.single_op = Some(SingleOp::Not);
                    ctx.start += ctx.length;
                    ctx.length = 0;
                    ctx.states.pop_state();
                    return Ok(());
                }
                Self::end_of_operator(source, ignore_nline, meta, ctx)?;
                return Ok(());
                //ctx.states.pop_state();
            }
            let mut sub_meta = meta.from_offset(ctx.start);
            let fsr_type = context.get_token_var_type(&name, &context);
            let mut variable = FSRVariable::parse(&name, sub_meta, fsr_type).unwrap();
            if context.is_variable_defined_in_curr(variable.get_name()) {
                variable.is_defined = true;
            } else {
                context.ref_variable(variable.get_name());
            }
            // variable.single_op = ctx.single_op;
            // ctx.single_op = None;
            ctx.candidates.push(FSRToken::Variable(variable));
            ctx.start += ctx.length;
            ctx.length = 0;
            ctx.states.pop_state();
            return Ok(());
        }

        Ok(())
    }

    #[inline]
    fn number_process(
        source: &[u8],
        ignore_nline: bool,
        meta: &FSRPosition,
        ctx: &mut StmtContext,
    ) -> Result<(), SyntaxError> {
        let mut is_float = false;
        loop {
            if ctx.start + ctx.length >= source.len() {
                break;
            }
            let c = source[ctx.start + ctx.length] as char;

            if c.eq(&'.') {
                if ctx.start + ctx.length + 1 < source.len()
                    && source[ctx.start + ctx.length + 1] == b'.'
                {
                    // like range 0..4
                    break;
                }
                is_float = true;
                ctx.length += 1;
                continue;
            }

            if !c.is_ascii_digit() {
                break;
            }

            ctx.length += 1;
        }
        let ps = str::from_utf8(&source[ctx.start..(ctx.start + ctx.length)]).unwrap();
        let mut sub_meta = meta.from_offset(ctx.start);
        let c = if is_float {
            let f = ps.parse::<f64>().unwrap();
            FSRConstant::from_float(f as f64, sub_meta, ps, ctx.single_op)
        } else {
            let i = ps.parse::<i64>().unwrap();
            FSRConstant::from_int(i, sub_meta, ps, ctx.single_op)
        };

        //c.single_op = ctx.single_op;
        ctx.single_op = None;
        ctx.candidates.push(FSRToken::Constant(c));
        ctx.start += ctx.length;
        ctx.length = 0;

        Ok(())
    }

    #[inline]
    fn stmt_loop(
        source: &[u8],
        ignore_nline: bool,
        meta: &FSRPosition,
        ctx: &mut StmtContext,
        context: &mut ASTContext,
    ) -> Result<(), SyntaxError> {
        loop {
            if ctx.last_loop {
                break;
            }
            if ctx.start + ctx.length >= source.len() {
                break;
            }

            let ord = source[ctx.start];
            let c = ord as char;
            let t_i = source[ctx.start + ctx.length];
            let t_c = t_i as char;

            // Process comment
            if (t_c == '#'
                && !(ctx.states.eq_peek(&ExprState::SingleString)
                    || ctx.states.eq_peek(&ExprState::DoubleString)))
            {
                if ctx.length != 0 {
                    let sub_meta = meta.from_offset(meta.offset);
                    return Err(SyntaxError::new_with_type(
                        &FSRPosition::from_offset(&sub_meta, ctx.start + ctx.length),
                        "error # place",
                        SyntaxErrType::CommentError,
                    ));
                }

                while ctx.start + ctx.length < source.len()
                    && source[ctx.start + ctx.length] != b'\n'
                {
                    ctx.start += 1;
                }

                continue;
            }

            if ((t_c == '\n' && !ignore_nline) || t_c == ';' || t_c == '}')
                && !ctx.states.eq_peek(&ExprState::EscapeNewline)
            {
                if ctx.states.eq_peek(&ExprState::WaitToken) {
                    break;
                }
                ctx.last_loop = true;
            }

            if ctx.states.eq_peek(&ExprState::WaitToken) && c == '|' {
                // Process lambda, like |a, b| {
                //
                // }
                if !ctx.operators.is_empty() || ctx.candidates.is_empty() {
                    let fn_def = FSRFnDef::parse_lambda(
                        &source[ctx.start..],
                        meta.from_offset(ctx.start),
                        &format!("___lambda_zXjiTkDs_{}", unsafe { LAMBDA_NUMBER }),
                        context,
                    )?;
                    unsafe {
                        LAMBDA_NUMBER += 1;
                    }
                    ctx.start += fn_def.get_len();
                    ctx.candidates.push(FSRToken::FunctionDef(Rc::new(fn_def)));
                    continue;
                }
            }

            if ctx.states.eq_peek(&ExprState::WaitToken) && Self::is_op_one_char(c) {
                ctx.states.push_state(ExprState::Operator);
                continue;
            }

            if ctx.states.eq_peek(&ExprState::Operator) && Self::is_op_one_char(t_c) {
                ctx.length += 1;
                continue;
            }

            if ctx.states.eq_peek(&ExprState::Operator) && !Self::is_op_one_char(t_c) {
                Self::end_of_operator(source, ignore_nline, meta, ctx)?;
                continue;
            }

            if t_i as char == '('
                && (ctx.states.eq_peek(&ExprState::Bracket)
                    || ctx.states.eq_peek(&ExprState::WaitToken))
            {
                if ctx.bracket_count == 0 {
                    ctx.start += 1;
                    ctx.states.push_state(ExprState::Bracket);
                    ctx.bracket_count += 1;
                } else {
                    ctx.length += 1;
                    ctx.states.push_state(ExprState::Bracket);
                    ctx.bracket_count += 1;
                }

                continue;
            }

            if t_i as char != ')'
                && (!ctx.states.eq_peek(&ExprState::SingleString)
                    && !ctx.states.eq_peek(&ExprState::DoubleString))
                && ctx.states.eq_peek(&ExprState::Bracket)
            {
                ctx.length += 1;
                continue;
            }

            if t_i as char == ')'
                && (!ctx.states.eq_peek(&ExprState::SingleString)
                    && !ctx.states.eq_peek(&ExprState::DoubleString))
                && ctx.states.eq_peek(&ExprState::Bracket)
                || ctx.last_loop
            {
                Self::end_of_bracket(source, ignore_nline, meta, ctx, context)?;
                continue;
            }

            if ctx.states.eq_peek(&ExprState::WaitToken)
                && ASTParser::is_blank_char_with_new_line(ord)
            {
                ctx.start += 1;
                continue;
            }

            if ctx.states.eq_peek(&ExprState::WaitToken) && c == '\'' {
                Self::single_quote_loop(source, ignore_nline, meta, ctx)?;
                continue;
            }

            if ctx.states.eq_peek(&ExprState::WaitToken) && c == '\"' {
                Self::double_quote_loop(source, ignore_nline, meta, ctx)?;
                continue;
            }

            if ctx.states.eq_peek(&ExprState::WaitToken) && t_c.is_ascii_digit() {
                Self::number_process(source, ignore_nline, meta, ctx)?;

                continue;
            }

            if ctx.states.eq_peek(&ExprState::WaitToken) && t_c == '[' {
                let mut sub_meta = meta.from_offset(ctx.start);
                let len = ASTParser::read_valid_bracket(&source[ctx.start..], sub_meta.clone())?;
                assert!(len >= 2);

                let list =
                    FSRListFrontEnd::parse(&source[ctx.start..ctx.start + len], sub_meta, context)?;
                ctx.candidates.push(FSRToken::List(list));
                ctx.start += len;
                ctx.length = 0;
                continue;
            }

            if ctx.states.eq_peek(&ExprState::WaitToken) && ASTParser::is_name_letter_first(ord) {
                Self::variable_process(source, ignore_nline, meta, ctx, context)?;
                continue;
            }

            if ctx.states.eq_peek(&ExprState::Variable) && t_c == '(' {
                let mut sub_meta = meta.from_offset(ctx.start);
                let len =
                    ASTParser::read_valid_bracket(&source[ctx.start + ctx.length..], sub_meta)?;
                ctx.length += len;
                let mut sub_meta = meta.from_offset(ctx.start);
                let mut call = FSRCall::parse(
                    &source[ctx.start..ctx.start + ctx.length],
                    sub_meta,
                    context,
                    false,
                )?;

                // if reference to defined variable, will set is_defined to true
                if context.is_variable_defined_in_curr(call.get_name()) {
                    call.is_defined = true;
                } else {
                    context.ref_variable(call.get_name());
                }
                if ctx.operators.is_empty() && !ctx.candidates.is_empty() {
                    let mut stack_expr = vec![];
                    let mut right = ctx.candidates.pop().unwrap();
                    if right.is_stack_expr() {
                        right.try_push_stack_expr(FSRToken::Call(call));
                        ctx.candidates.push(right);
                    } else {
                        stack_expr.push(right);
                        stack_expr.push(FSRToken::Call(call));
                        ctx.candidates
                            .push(FSRToken::StackExpr((ctx.single_op.take(), stack_expr)));
                    }
                } else {
                    call.single_op = ctx.single_op;
                    ctx.candidates.push(FSRToken::Call(call));
                    ctx.single_op = None;
                }

                // escape blank char, case like a[1] (1 + 2)
                while ctx.start < source.len() && ASTParser::is_blank_char(source[ctx.start]) {
                    ctx.start += 1;
                }

                ctx.start += ctx.length;
                ctx.length = 0;
                //ctx.states.pop_state();
                continue;
            }

            if ctx.states.eq_peek(&ExprState::Variable) && t_c == '[' {
                let mut sub_meta = meta.from_offset(ctx.start);
                let len =
                    ASTParser::read_valid_bracket(&source[ctx.start + ctx.length..], sub_meta)?;
                ctx.length += len;
                let mut sub_meta = meta.from_offset(ctx.start);
                let mut getter = FSRGetter::parse(
                    &source[ctx.start..ctx.start + ctx.length],
                    sub_meta,
                    context,
                )
                .unwrap();

                if context.is_variable_defined_in_curr(getter.get_name()) {
                    getter.is_defined = true;
                } else {
                    context.ref_variable(getter.get_name());
                }
                if ctx.operators.is_empty() && !ctx.candidates.is_empty() {
                    let mut stack_expr = vec![];
                    let mut right = ctx.candidates.pop().unwrap();
                    if right.is_stack_expr() {
                        right.try_push_stack_expr(FSRToken::Getter(getter));
                        ctx.candidates.push(right);
                    } else {
                        stack_expr.push(right);
                        stack_expr.push(FSRToken::Getter(getter));
                        ctx.candidates
                            .push(FSRToken::StackExpr((ctx.single_op.take(), stack_expr)));
                    }
                } else {
                    ctx.candidates.push(FSRToken::Getter(getter));
                }

                ctx.start += ctx.length;
                ctx.length = 0;

                while ctx.start < source.len() && ASTParser::is_blank_char(source[ctx.start]) {
                    ctx.start += 1;
                }
                // ctx.states.pop_state();
                continue;
            }

            if (ctx.states.eq_peek(&ExprState::Variable) && !ASTParser::is_name_letter(t_i))
                || ctx.last_loop
            {
                if !ctx.candidates.is_empty() {
                    let c = ctx.candidates.first().unwrap();
                    // case like a[1][1] + 2
                    if c.is_stack_expr() || c.is_call() || c.is_getter() {
                        ctx.states.pop_state();
                        continue;
                    }
                }
                let name = str::from_utf8(&source[ctx.start..ctx.start + ctx.length]).unwrap();

                if name.eq("and") || name.eq("or") || name.eq("not") {
                    if name.eq("not") {
                        ctx.single_op_level = Some(Node::get_single_op_level(&SingleOp::Not));
                    }
                    Self::end_of_operator(source, ignore_nline, meta, ctx)?;
                    continue;
                }

                let mut sub_meta = meta.from_offset(ctx.start);
                let mut variable = FSRVariable::parse(name, sub_meta, None).unwrap();
                if context.is_variable_defined_in_curr(variable.get_name()) {
                    variable.is_defined = true;
                } else {
                    context.ref_variable(variable.get_name());
                }
                variable.single_op = ctx.single_op;
                ctx.single_op = None;

                ctx.candidates.push(FSRToken::Variable(variable));
                ctx.start += ctx.length;
                ctx.length = 0;
                ctx.states.pop_state();
                continue;
            }

            if ctx.states.eq_peek(&ExprState::Slice) && !FSRGetter::is_valid_char(t_c as u8) {
                unimplemented!()
            }
        }

        if let Some(s_op) = ctx.single_op {
            ctx.candidates.last_mut().unwrap().set_single_op(s_op);
            ctx.single_op = None;
        }

        if ctx.states.eq_peek(&ExprState::Operator) {
            return Err(SyntaxError::new_with_type(
                &meta.from_offset(ctx.start),
                "Must have a expr after operator",
                SyntaxErrType::OperatorError,
            ));
        }

        Ok(())
    }

    pub fn parse(
        source: &[u8],
        ignore_nline: bool,
        meta: FSRPosition,
        context: &mut ASTContext,
    ) -> Result<(FSRToken, usize), SyntaxError> {
        let mut ctx = StmtContext::new();
        Self::stmt_loop(source, ignore_nline, &meta, &mut ctx, context)?;

        if ctx.candidates.is_empty() {
            return Ok((FSRToken::EmptyExpr, ctx.start + ctx.length));
        }

        ctx.operators.sort_by(|a, b| -> Ordering {
            if a.0 != b.0 {
                Node::is_higher_priority(a.0, b.0)
            } else if a.1 < b.1 {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        });

        if ctx.candidates.len() == 2 {
            let mut left = ctx.candidates.remove(0);
            let mut right = ctx.candidates.remove(0);

            if right.is_empty() {
                return Ok((left, ctx.start + ctx.length));
            }
            let mut n_left = left.clone();
            if ctx.operators.is_empty() {
                let mut stack_expr = vec![left, right];

                return Ok((
                    FSRToken::StackExpr((ctx.single_op.take(), stack_expr)),
                    ctx.start + ctx.length,
                ));
            }
            let op = ctx.operators.remove(0).0;
            if op.eq("=") {
                if let FSRToken::Variable(mut name) = left {
                    let type_hint = right.deduction_type(&context);
                    n_left.as_mut_variable().set_type_hint(type_hint);
                    context.add_variable(name.get_name(), Some(n_left.clone()));
                    return Ok((
                        FSRToken::Assign(FSRAssign {
                            left: Rc::new(n_left),
                            name: name.get_name().to_string(),
                            expr: Rc::new(right),
                            len: ctx.start + ctx.length,
                            meta,
                        }),
                        ctx.start + ctx.length,
                    ));
                } else if let FSRToken::Getter(getter) = left {
                    let name = getter.get_name();
                    return Ok((
                        FSRToken::Assign(FSRAssign {
                            left: Rc::new(n_left),
                            name: getter.get_name().to_string(),
                            expr: Rc::new(right),
                            len: ctx.start + ctx.length,
                            meta,
                        }),
                        ctx.start + ctx.length,
                    ));
                }
            }

            if op.eq(":") {
                if let FSRToken::Variable(name) = &left {
                    let name = name.get_name();
                    if let FSRToken::Variable(type_name) = &right {
                        let mut var = FSRVariable::parse(
                            name,
                            left.get_meta().clone(),
                            Some(FSRType::new(type_name.get_name())),
                        )
                        .unwrap();
                        var.force_type = true;
                        return Ok((FSRToken::Variable(var), ctx.start + ctx.length));
                    } else {
                        panic!("Type name must be a string")
                    }
                } else {
                    unimplemented!()
                }
            }

            return Ok((
                FSRToken::Expr(Self {
                    single_op: ctx.single_op,
                    left: Box::new(left),
                    right: Box::new(right),
                    op: Some(op),
                    len: ctx.start + ctx.length,
                    meta,
                }),
                ctx.start + ctx.length,
            ));
        }

        if ctx.candidates.len().eq(&1) {
            if !ctx.operators.is_empty() {
                let mut sub_meta = meta.from_offset(ctx.operators[0].1);
                let err = SyntaxError::new_with_type(
                    &sub_meta,
                    format!(
                        "Must have second ctx.candidates with {}",
                        ctx.operators[0].0
                    ),
                    SyntaxErrType::OperatorError,
                );
                return Err(err);
            }
            let mut c = ctx.candidates.remove(0);
            if let FSRToken::Constant(s) = &mut c {
                s.single_op = ctx.single_op;
            }

            return Ok((c, ctx.start + ctx.length));
        }

        if ctx.operators.is_empty() {
            println!("first candidates: {:#?}", ctx.candidates);
            unimplemented!()
        }

        let operator = ctx.operators[0];

        let split_offset = operator.1;

        let mut sub_meta = meta.from_offset(0);
        let left = FSRExpr::parse(&source[0..split_offset], false, sub_meta, context)?.0;

        let mut sub_meta = meta.from_offset(0);
        let right = FSRExpr::parse(
            &source[split_offset + operator.0.len()..],
            false,
            sub_meta.clone(),
            context,
        )?
        .0;
        let mut n_left = left.clone();

        if operator.0.eq("=") {
            if let FSRToken::Variable(name) = left {
                
                let type_hint = right.deduction_type(&context);
                // context.set_variable_type(name.get_name(), type_hint.clone());
                n_left.as_mut_variable().set_type_hint(type_hint);
                context.add_variable(name.get_name(), Some(n_left.clone()));
                return Ok((
                    FSRToken::Assign(FSRAssign {
                        left: Rc::new(n_left),
                        name: name.get_name().to_string(),
                        expr: Rc::new(right),
                        len: ctx.start + ctx.length,
                        meta,
                    }),
                    ctx.start + ctx.length,
                ));
            } else {
                return Ok((
                    FSRToken::Assign(FSRAssign {
                        left: Rc::new(n_left),
                        name: "".to_string(),
                        expr: Rc::new(right),
                        len: ctx.start + ctx.length,
                        meta,
                    }),
                    ctx.start + ctx.length,
                ));
            }
        }

        if operator.0.eq(":") {
            if let FSRToken::Variable(name) = &left {
                let name = name.get_name();
                if let FSRToken::Variable(type_name) = &right {
                    let mut var = FSRVariable::parse(
                        name,
                        left.get_meta().clone(),
                        Some(FSRType::new(type_name.get_name())),
                    )
                    .unwrap();
                    var.force_type = true;
                    context.set_variable_token(name, Some(FSRToken::Variable(var.clone())));
                    return Ok((FSRToken::Variable(var), ctx.start + ctx.length));
                } else {
                    panic!("Type name must be a string")
                }
            } else {
                unimplemented!()
            }
        }
        Ok((
            FSRToken::Expr(Self {
                single_op: ctx.single_op,
                left: Box::new(left),
                right: Box::new(right),
                op: Some(operator.0),
                len: ctx.start + ctx.length,
                meta,
            }),
            ctx.start + ctx.length,
        ))
    }

    pub fn get_op(&self) -> &str {
        self.op.unwrap()
    }
}
