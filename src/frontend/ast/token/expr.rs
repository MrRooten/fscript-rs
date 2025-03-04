#![allow(unused)]

use std::rc::Rc;
use std::{cmp::Ordering, fmt::Display};

use crate::frontend::ast::token;
use crate::frontend::ast::token::assign::FSRAssign;
use crate::frontend::ast::token::list::FSRListFrontEnd;
use crate::frontend::ast::token::slice::FSRSlice;
use crate::frontend::ast::{parse::ASTParser, token::constant::FSRConstant};
use crate::utils::error::{SyntaxErrType, SyntaxError};
use std::str;

use super::base::FSRPosition;
use super::{base::FSRToken, call::FSRCall, variable::FSRVariable};

#[derive(Debug, Clone)]
pub struct FSRExpr<'a> {
    pub(crate)single_op: Option<&'a str>,
    pub(crate)left: Box<FSRToken<'a>>,
    pub(crate)right: Box<FSRToken<'a>>,
    pub(crate)op: Option<&'a str>,
    pub(crate)len: usize,
    pub(crate)meta: FSRPosition,
}

impl<'a> FSRExpr<'a> {
    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }

    pub fn get_left(&self) -> &FSRToken<'a> {
        &self.left
    }
}

impl Display for FSRExpr<'_> {
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
    Comment
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
        return self.peek().eq(state);
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
    value: Option<FSRToken<'a>>,
    is_leaf: bool,
}

#[derive(Debug)]
struct TreeNode<'a> {
    node: Option<Node<'a>>,
    head: *mut Node<'a>,
    last: *mut Node<'a>,
}

impl<'a> Node<'a> {
    pub fn from_value(value: FSRToken<'a>) -> Node<'a> {
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


    pub fn get_op_level(op: &str) -> i32 {
        if op.eq("-") || op.eq("+") {
            return 1;
        }

        if op.eq("*") || op.eq("/") {
            return 2;
        }

        if op.eq(">>") || op.eq("<<") {
            return 2;
        }

        if op.eq(".") {
            return 3;
        }

        if op.eq(">") || op.eq("<") || op.eq("==") || op.eq("!=") {
            return 0;
        }

        if op.eq("&&") || op.eq("||") {
            return -2;
        }

        if op.eq("and") || op.eq("or") {
            return -2
        }

        if op.eq("=") {
            return -3;
        }

        if op.eq(",") {
            return -4;
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
pub enum FSRBinOpResult<'a> {
    BinOp(FSRExpr<'a>),
    Constant(FSRConstant<'a>),
}

struct StmtContext<'a> {
    states: ExprStates,
    start: usize,
    length: usize,
    bracket_count: i32,
    candidates: Vec<FSRToken<'a>>,
    operators: Vec<(&'static str, usize)>,
    single_op: Option<&'static str>,
    last_loop: bool,
    is_high_prio_single_op: bool
}

impl<'a> StmtContext<'a> {
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
            is_high_prio_single_op: false,
        }
    }
}

impl<'a> FSRExpr<'a> {
    pub fn get_single_op(&self) -> Option<&'a str> {
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
        {
            return true;
        }

        false
    }

    #[inline]
    fn double_quote_loop(
        source: &'a [u8],
        ignore_nline: bool,
        meta: &FSRPosition,
        ctx: &mut StmtContext<'a>,
    ) -> Result<(), SyntaxError> {
        if let Some(s_op) = ctx.single_op {
            let mut sub_meta = meta.from_offset(ctx.start);
            return Err(SyntaxError::new(
                &sub_meta,
                format!("{} can not follow string", s_op),
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
        source: &'a [u8],
        ignore_nline: bool,
        meta: &FSRPosition,
        ctx: &mut StmtContext<'a>,
    ) -> Result<(), SyntaxError> {
        if let Some(s_op) = ctx.single_op {
            let mut sub_meta = meta.from_offset(ctx.start);
            return Err(SyntaxError::new(
                &sub_meta,
                format!("{} can not follow string", s_op),
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

    #[inline]
    fn end_of_char(
        source: &'a [u8],
        ignore_nline: bool,
        meta: &FSRPosition,
        ctx: &mut StmtContext<'a>,
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
            ctx.single_op = Some(op);
            ctx.states.pop_state();
            ctx.start += ctx.length;
            ctx.length = 0;
            return Ok(());
        }

        if Self::is_single_op(op) && !op.eq("-") {
            if ctx.single_op.is_some() && (ctx.single_op.unwrap().eq("not") || ctx.single_op.unwrap().eq("!")) {
                ctx.single_op = None;
            } else {
                ctx.single_op = Some(op);
            }
            
            ctx.states.pop_state();
            ctx.start += ctx.length;
            ctx.length = 0;
        } else {
            ctx.operators.push((op, ctx.start));
            ctx.states.pop_state();
            ctx.start += ctx.length;
            ctx.length = 0;
        }
        Ok(())
    }

    #[inline]
    fn end_of_bracket(
        source: &'a [u8],
        ignore_nline: bool,
        meta: &FSRPosition,
        ctx: &mut StmtContext<'a>,
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
            let mut sub_expr = FSRExpr::parse(_ps, true, sub_meta)?.0;
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

    #[inline]
    fn variable_process(
        source: &'a [u8],
        ignore_nline: bool,
        meta: &FSRPosition,
        ctx: &mut StmtContext<'a>,
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
            || (source[ctx.start + ctx.length] != b'('
                && source[ctx.start + ctx.length] != b'[')
        {
            let name = str::from_utf8(&source[ctx.start..ctx.start + ctx.length]).unwrap();
            if name.eq("and") || name.eq("or") || name.eq("not") {
                if name.eq("not") {
                    ctx.is_high_prio_single_op = true;
                }
                Self::end_of_char(source, ignore_nline, meta, ctx)?;
                return Ok(())
                //ctx.states.pop_state();
            }
            let mut sub_meta = meta.from_offset(ctx.start);
            let mut variable = FSRVariable::parse(name, sub_meta).unwrap();
            variable.single_op = ctx.single_op;
            ctx.single_op = None;
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
        source: &'a [u8],
        ignore_nline: bool,
        meta: &FSRPosition,
        ctx: &mut StmtContext<'a>,
    ) -> Result<(), SyntaxError> {

        let mut is_float = false;
        loop {
            if ctx.start + ctx.length >= source.len() {
                break;
            }
            let c = source[ctx.start + ctx.length] as char;

            if c.eq(&'.') {
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
            FSRConstant::from_float(f as f64, sub_meta, ps)
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
        source: &'a [u8],
        ignore_nline: bool,
        meta: &FSRPosition,
        ctx: &mut StmtContext<'a>,
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

            if ((t_c == '#' && !(ctx.states.eq_peek(&ExprState::SingleString) || ctx.states.eq_peek(&ExprState::DoubleString)))) {
                if ctx.length != 0 {
                    let sub_meta = meta.from_offset(meta.offset);
                    return Err(SyntaxError::new_with_type(&FSRPosition::from_offset(&sub_meta, ctx.start + ctx.length), "error # place", SyntaxErrType::CommentError))
                }

                while ctx.start + ctx.length < source.len() && source[ctx.start + ctx.length] != '\n' as u8 {
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

            if ctx.states.eq_peek(&ExprState::WaitToken) && Self::is_op_one_char(c) {
                ctx.states.push_state(ExprState::Operator);
                continue;
            }

            if ctx.states.eq_peek(&ExprState::Operator) && Self::is_op_one_char(t_c) {
                ctx.length += 1;
                continue;
            }

            if ctx.states.eq_peek(&ExprState::Operator) && !Self::is_op_one_char(t_c) {
                Self::end_of_char(source, ignore_nline, meta, ctx)?;
                continue;
            }


            if ctx.is_high_prio_single_op && ctx.states.eq_peek(&ExprState::WaitToken) {
                let mut sub_expr = FSRExpr::parse(&source[ctx.start..], false, meta.from_offset(ctx.start))?;
                ctx.start += sub_expr.1;
                if let FSRToken::Expr(e) = &mut sub_expr.0 {
                    e.single_op = ctx.single_op;
                }
                if let FSRToken::Call(c) = &mut sub_expr.0 {
                    c.single_op = ctx.single_op;
                }
    
                if let FSRToken::Variable(v) = &mut sub_expr.0 {
                    v.single_op = ctx.single_op;
                }

                if let FSRToken::Constant(v) = &mut sub_expr.0 {
                    v.single_op = ctx.single_op;
                }

                ctx.is_high_prio_single_op = false;
                ctx.candidates.push(sub_expr.0);
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
                Self::end_of_bracket(source, ignore_nline, meta, ctx)?;
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

                let list = FSRListFrontEnd::parse(&source[ctx.start..ctx.start + len], sub_meta)?;
                ctx.candidates.push(FSRToken::List(list));
                ctx.start += len;
                ctx.length = 0;
                continue;
            }

            if ctx.states.eq_peek(&ExprState::WaitToken) && ASTParser::is_name_letter_first(ord) {
                Self::variable_process(source, ignore_nline, meta, ctx)?;

                continue;
            }

            if ctx.states.eq_peek(&ExprState::Variable) && t_c == '(' {
                let mut sub_meta = meta.from_offset(ctx.start);
                let len =
                    ASTParser::read_valid_bracket(&source[ctx.start + ctx.length..], sub_meta)?;
                ctx.length += len;
                let mut sub_meta = meta.from_offset(ctx.start);
                let mut call =
                    FSRCall::parse(&source[ctx.start..ctx.start + ctx.length], sub_meta)?;
                call.single_op = ctx.single_op;
                ctx.single_op = None;
                ctx.candidates.push(FSRToken::Call(call));
                ctx.start += ctx.length;
                ctx.length = 0;
                ctx.states.pop_state();
                continue;
            }

            if ctx.states.eq_peek(&ExprState::Variable) && t_c == '[' {
                let mut sub_meta = meta.from_offset(ctx.start);
                let len =
                    ASTParser::read_valid_bracket(&source[ctx.start + ctx.length..], sub_meta)?;
                ctx.length += len;
                let slice =
                    FSRSlice::parse(&source[ctx.start..ctx.start + ctx.length + 1]).unwrap();
                ctx.start += ctx.length;
                ctx.length = 0;
                continue;
            }

            if (ctx.states.eq_peek(&ExprState::Variable) && !ASTParser::is_name_letter(t_i))
                || ctx.last_loop
            {
                let name = str::from_utf8(&source[ctx.start..ctx.start + ctx.length]).unwrap();

                if name.eq("and") || name.eq("or") || name.eq("not") {
                    if name.eq("not") {
                        ctx.is_high_prio_single_op = true;
                    }
                    Self::end_of_char(source, ignore_nline, meta, ctx)?;
                    continue;
                }

                let mut sub_meta = meta.from_offset(ctx.start);
                let mut variable = FSRVariable::parse(name, sub_meta).unwrap();
                variable.single_op = ctx.single_op;
                ctx.single_op = None;
                ctx.candidates.push(FSRToken::Variable(variable));
                ctx.start += ctx.length;
                ctx.length = 0;
                ctx.states.pop_state();
                continue;
            }

            if ctx.states.eq_peek(&ExprState::Slice) && !FSRSlice::is_valid_char(t_c as u8) {
                unimplemented!()
            }
        }

        Ok(())
    }

    pub fn parse(
        source: &'a [u8],
        ignore_nline: bool,
        meta: FSRPosition,
    ) -> Result<(FSRToken<'a>, usize), SyntaxError> {
        let mut ctx = StmtContext::new();
        Self::stmt_loop(source, ignore_nline, &meta, &mut ctx)?;

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
            let left = ctx.candidates.remove(0);
            let right = ctx.candidates.remove(0);
            let n_left = left.clone();
            let op = ctx.operators.remove(0).0;
            if op.eq("=") {
                if let FSRToken::Variable(name) = left {
                    return Ok((
                        FSRToken::Assign(FSRAssign {
                            left: Rc::new(n_left),
                            name: name.get_name(),
                            expr: Rc::new(right),
                            len: ctx.start + ctx.length,
                            meta,
                        }),
                        ctx.start + ctx.length,
                    ));
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

        let operator = ctx.operators[0];
        let split_offset = operator.1;

        let mut sub_meta = meta.from_offset(0);
        let left = FSRExpr::parse(&source[0..split_offset], false, sub_meta)?.0;

        let mut sub_meta = meta.from_offset(0);
        let right = FSRExpr::parse(&source[split_offset + operator.0.len()..], false, sub_meta.clone())?.0;
        let n_left = left.clone();

        if operator.0.eq("=") {
            if let FSRToken::Variable(name) = left {
                return Ok((
                    FSRToken::Assign(FSRAssign {
                        left: Rc::new(n_left),
                        name: name.get_name(),
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
                        name: "",
                        expr: Rc::new(right),
                        len: ctx.start + ctx.length,
                        meta,
                    }),
                    ctx.start + ctx.length,
                ));
            }
        }
        return Ok((
            FSRToken::Expr(Self {
                single_op: ctx.single_op,
                left: Box::new(left),
                right: Box::new(right),
                op: Some(operator.0),
                len: ctx.start + ctx.length,
                meta,
            }),
            ctx.start + ctx.length,
        ));
    }

    pub fn get_op(&self) -> &str {
        self.op.unwrap()
    }
}

mod test {
    use crate::frontend::ast::token::base::FSRPosition;

    use super::FSRExpr;

    #[test]
    fn test_binary_str() {
        let v = "v and c";
        let p = FSRExpr::parse(v.as_bytes(), true, FSRPosition::new()).unwrap();
        println!("{:#?}", p.0);
    }

    #[test]
    fn test_binary_str2() {
        let v = "not c and d";
        let p = FSRExpr::parse(v.as_bytes(), true, FSRPosition::new()).unwrap();
        println!("{:#?}", p.0);
    }

    #[test]
    fn test_binary_str3() {
        let v = "not false";
        let p = FSRExpr::parse(v.as_bytes(), true, FSRPosition::new()).unwrap();
        println!("{:#?}", p.0);
    }


    #[test]
    fn test_binary_str4() {
        let v = "(not 1 == 1) or 1 == 1";
        let p = FSRExpr::parse(v.as_bytes(), true, FSRPosition::new()).unwrap();
        println!("{:#?}", p.0);
    }

    #[test]
    fn test_float() {
        let v = "1.0 + 1.0";
        let p = FSRExpr::parse(v.as_bytes(), true, FSRPosition::new()).unwrap();
        println!("{:#?}", p.0);
    }


    #[test]
    fn test_float_div() {
        let v = "1.0 / 1.0";
        let p = FSRExpr::parse(v.as_bytes(), true, FSRPosition::new()).unwrap();
        println!("{:#?}", p.0);
    }
}