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

use super::base::FSRMeta;
use super::{base::FSRToken, call::FSRCall, variable::FSRVariable};

#[derive(Debug, Clone)]
pub struct FSRExpr<'a> {
    single_op: Option<&'a str>,
    left: Box<FSRToken<'a>>,
    right: Box<FSRToken<'a>>,
    op: Option<&'a str>,
    len: usize,
    meta: FSRMeta,
}

impl<'a> FSRExpr<'a> {
    pub fn get_meta(&self) -> &FSRMeta {
        return &self.meta;
    }

    pub fn get_left(&self) -> &Box<FSRToken<'a>> {
        return &self.left;
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
}

struct ExprStates {
    states: Vec<ExprState>,
}

impl ExprStates {
    pub fn new() -> Self {
        return Self { states: vec![] };
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

    pub fn add_higher_priority(&mut self, node: Node) {
        unimplemented!()
    }

    pub fn add_lower_priority(&mut self, node: Node) {
        unimplemented!()
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

        if op.eq(">") || op.eq("<") || op.ends_with("=") {
            return 0;
        }

        if op.eq("=") {
            return -2;
        }

        if op.eq(",") {
            return -3;
        }
        return -1;
    }

    pub fn is_higher_priority(op1: &str, op2: &str) -> Ordering {
        let op1 = Node::get_op_level(op1);
        let op2 = Node::get_op_level(op2);

        if op1 > op2 {
            return Ordering::Greater;
        } else if op1 < op2 {
            return Ordering::Less;
        } else {
            return Ordering::Equal;
        }
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

impl<'a> FSRExpr<'a> {
    pub fn get_len(&self) -> usize {
        return self.len;
    }

    pub fn is_single_op(ps: &str) -> bool {
        if ps.eq("!") {
            return true;
        }

        if ps.eq("-") {
            return true;
        }

        return false;
    }

    pub fn expr_split_by_comma(&self) -> Vec<FSRToken> {
        unimplemented!()
    }

    pub fn get_right(&self) -> &Box<FSRToken> {
        return &self.right;
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
        {
            return true;
        }

        return false;
    }

    pub fn parse(
        source: &'a [u8],
        ignore_nline: bool,
        meta: FSRMeta,
    ) -> Result<(FSRToken<'a>, usize), SyntaxError> {
        let mut states = ExprStates::new();
        states.push_state(ExprState::WaitToken);
        let mut start = 0;
        let mut length = 0;
        let mut bracket_count = 0;
        let mut candidates: Vec<FSRToken> = vec![];
        let mut operators: Vec<(&str, usize)> = vec![];
        let mut single_op: Option<&str> = None;
        let mut last_loop = false;
        loop {
            if last_loop {
                break;
            }
            if start + length >= source.len() {
                break;
            }

            let ord = source[start];
            let c = ord as char;
            let t_i = source[start + length];
            let t_c = t_i as char;

            if ((t_c == '\n' && ignore_nline == false) || t_c == ';' || t_c == '}')
                && states.eq_peek(&ExprState::EscapeNewline) == false
            {
                if states.eq_peek(&ExprState::WaitToken) {
                    break;
                }
                last_loop = true;
            }

            if states.eq_peek(&ExprState::WaitToken) && Self::is_op_one_char(c) {
                states.push_state(ExprState::Operator);
                continue;
            }

            if states.eq_peek(&ExprState::Operator) && Self::is_op_one_char(t_c) {
                length += 1;
                continue;
            }

            if states.eq_peek(&ExprState::Operator) && Self::is_op_one_char(t_c) == false {
                let op = str::from_utf8(&source[start..start + length]).unwrap();
                if start + length >= source.len() {
                    let mut sub_meta = meta.clone();
                    sub_meta.offset += start;
                    return Err(SyntaxError::new(
                        &sub_meta,
                        format!("{} must follow a expr or variable", op),
                    ));
                }

                if op.eq("-") && (source[start + length] as char).is_digit(10) {
                    single_op = Some(op);
                    states.pop_state();
                    start = start + length;
                    length = 0;
                    continue;
                }

                if Self::is_single_op(op) == true && op.eq("-") == false {
                    single_op = Some(op);
                    states.pop_state();
                    start = start + length;
                    length = 0;
                } else {
                    operators.push((op, start));
                    states.pop_state();
                    start = start + length;
                    length = 0;
                }
                continue;
            }

            if t_i as char == '('
                && (states.eq_peek(&ExprState::Bracket) || states.eq_peek(&ExprState::WaitToken))
            {
                if bracket_count == 0 {
                    start += 1;
                    states.push_state(ExprState::Bracket);
                    bracket_count += 1;
                } else {
                    length += 1;
                    states.push_state(ExprState::Bracket);
                    bracket_count += 1;
                }

                continue;
            }

            if t_i as char != ')'
                && (states.eq_peek(&ExprState::SingleString) == false
                    && states.eq_peek(&ExprState::DoubleString) == false)
                && states.eq_peek(&ExprState::Bracket)
            {
                length += 1;
                continue;
            }

            if t_i as char == ')'
                && (states.eq_peek(&ExprState::SingleString) == false
                    && states.eq_peek(&ExprState::DoubleString) == false)
                && states.eq_peek(&ExprState::Bracket)
                || last_loop
            {
                states.pop_state();
                bracket_count -= 1;

                if bracket_count > 0 {
                    length += 1;
                    continue;
                } else {
                    let _ps = &source[start..start + length];
                    let ps = str::from_utf8(_ps).unwrap();

                    start = start + length;
                    length = 0;
                    let sub_meta = meta.clone();
                    let mut sub_expr = FSRExpr::parse(_ps, true, sub_meta)?.0;
                    if let FSRToken::Expr(e) = &mut sub_expr {
                        e.single_op = single_op;
                    }
                    if let FSRToken::Call(c) = &mut sub_expr {
                        c.single_op = single_op;
                    }

                    if let FSRToken::Variable(v) = &mut sub_expr {
                        v.single_op = single_op;
                    }

                    single_op = None;
                    start += 1;
                    candidates.push(sub_expr);
                }
                continue;
            }

            if states.eq_peek(&ExprState::WaitToken) && ASTParser::is_blank_char_with_new_line(ord) {
                start += 1;
                continue;
            }

            // if states.eq_peek(&ExprState::WaitToken) && c.is_digit(10) {
            //     states.push_state(ExprState::Number);
            //     continue;
            // }

            if states.eq_peek(&ExprState::WaitToken) && c == '\'' {
                if let Some(s_op) = single_op {
                    let mut sub_meta = meta.clone();
                    sub_meta.offset += start;
                    return Err(SyntaxError::new(
                        &sub_meta,
                        format!("{} can not follow string", s_op),
                    ));
                }
                start += 1;
                loop {
                    if start + length >= source.len() {
                        let mut sub_meta = meta.clone();
                        sub_meta.offset = meta.offset + start;
                        let err = SyntaxError::new_with_type(
                            &sub_meta,
                            "Not Close for Single Quote",
                            SyntaxErrType::QuoteNotClose,
                        );
                        return Err(err);
                    }
                    let c = source[start + length] as char;
                    if states.eq_peek(&ExprState::EscapeChar) {
                        states.pop_state();
                        length += 1;
                        continue;
                    }

                    if c == '\'' {
                        break;
                    }

                    if c == '\\' {
                        states.push_state(ExprState::EscapeChar);
                    }

                    length += 1;
                }

                let s = &source[start..start + length];
                let mut sub_meta = meta.clone();
                sub_meta.offset = meta.offset + start;
                let constant = FSRConstant::from_str(s, sub_meta);
                candidates.push(FSRToken::Constant(constant));
                length += 1;
                start = start + length;
                length = 0;
                continue;
            }

            if states.eq_peek(&ExprState::WaitToken) && c == '\"' {
                if let Some(s_op) = single_op {
                    let mut sub_meta = meta.clone();
                    sub_meta.offset += start;
                    return Err(SyntaxError::new(
                        &sub_meta,
                        format!("{} can not follow string", s_op),
                    ));
                }
                start += 1;
                loop {
                    if start + length >= source.len() {
                        let mut sub_meta = meta.clone();
                        sub_meta.offset = meta.offset + start;
                        let err = SyntaxError::new_with_type(
                            &sub_meta,
                            "Not Close for Single Quote",
                            SyntaxErrType::QuoteNotClose,
                        );
                        return Err(err);
                    }
                    let c = source[start + length] as char;
                    if states.eq_peek(&ExprState::EscapeChar) {
                        states.pop_state();
                        length += 1;
                        continue;
                    }

                    if c == '\"' {
                        break;
                    }

                    if c == '\\' {
                        states.push_state(ExprState::EscapeChar);
                    }

                    length += 1;
                }

                let s = &source[start..start + length];
                let mut sub_meta = meta.clone();
                sub_meta.offset = meta.offset + start;
                let constant = FSRConstant::from_str(s, sub_meta);
                candidates.push(FSRToken::Constant(constant));
                length += 1;
                start = start + length;
                length = 0;
                continue;
            }

            if states.eq_peek(&ExprState::WaitToken) && t_c.is_digit(10) {
                loop {
                    if start + length >= source.len() {
                        break;
                    }
                    let c = source[start + length] as char;
                    if c.is_digit(10) == false {
                        break;
                    }

                    length += 1;
                }

                let ps = str::from_utf8(&source[start..(start + length)]).unwrap();
                let i = ps.parse::<i64>().unwrap();
                let mut sub_meta = meta.clone();
                sub_meta.offset = meta.offset + start;

                let mut c = FSRConstant::from_int(i, sub_meta);
                c.single_op = single_op;
                single_op = None;
                candidates.push(FSRToken::Constant(c));
                start = start + length;
                length = 0;
                continue;
            }
            if states.eq_peek(&ExprState::WaitToken) && t_c == '[' {
                let mut sub_meta = meta.clone();
                sub_meta.offset += start;
                let len = ASTParser::read_valid_bracket(&source[start..], sub_meta.clone())?;
                assert!(len >= 2);

                let list = FSRListFrontEnd::parse(&source[start..start+len], sub_meta)?;
                candidates.push(FSRToken::List(list));
                start += len;
                length = 0;
                continue;
            }
            if states.eq_peek(&ExprState::WaitToken) && ASTParser::is_name_letter_first(ord) {
                states.push_state(ExprState::Variable);
                loop {
                    if start + length >= source.len() {
                        break;
                    }
                    let c = source[start + length] as char;
                    if ASTParser::is_name_letter(c as u8) == false {
                        break;
                    }

                    length += 1;
                }

                if start + length >= source.len()
                    || (source[start + length] != '(' as u8 && source[start + length] != '[' as u8)
                {
                    let name = str::from_utf8(&source[start..start + length]).unwrap();
                    let mut sub_meta = meta.clone();
                    sub_meta.offset += start;
                    let mut variable = FSRVariable::parse(name, sub_meta).unwrap();
                    variable.single_op = single_op;
                    single_op = None;
                    candidates.push(FSRToken::Variable(variable));
                    start = start + length;
                    length = 0;
                    states.pop_state();
                    continue;
                }

                continue;
            }

            if states.eq_peek(&ExprState::Variable) && t_c == '(' {
                let mut sub_meta = meta.clone();
                sub_meta.offset = meta.offset + start + length;
                let len = ASTParser::read_valid_bracket(&source[start + length..], sub_meta)?;
                length += len;
                let mut sub_meta = meta.clone();
                sub_meta.offset = start + meta.offset;
                let mut call = FSRCall::parse(&source[start..start + length], sub_meta)?;
                call.single_op = single_op;
                single_op = None;
                candidates.push(FSRToken::Call(call));
                start = start + length;
                length = 0;
                states.pop_state();
                continue;
            }

            if states.eq_peek(&ExprState::Variable) && t_c == '[' {
                let mut sub_meta = meta.clone();
                sub_meta.offset = meta.offset + start + length;
                let len = ASTParser::read_valid_bracket(&source[start + length..], sub_meta)?;
                length += len;
                let slice = FSRSlice::parse(&source[start..start + length + 1]).unwrap();
                start = start + length;
                length = 0;
                continue;
            }

            if (states.eq_peek(&ExprState::Variable) && ASTParser::is_name_letter(t_i) == false)
                || last_loop
            {
                let name = str::from_utf8(&source[start..start + length]).unwrap();
                let mut sub_meta = meta.clone();
                sub_meta.offset += start;
                let mut variable = FSRVariable::parse(name, sub_meta).unwrap();
                variable.single_op = single_op;
                single_op = None;
                candidates.push(FSRToken::Variable(variable));
                start = start + length;
                length = 0;
                states.pop_state();
                continue;
            }

            if states.eq_peek(&ExprState::Slice) && FSRSlice::is_valid_char(t_c as u8) == false {
                unimplemented!()
            }
        }

        if candidates.len() == 0 {
            return Ok((FSRToken::EmptyExpr, start + length));
        }

        operators.sort_by(|a, b| -> Ordering { Node::is_higher_priority(a.0, b.0) });

        if candidates.len() == 2 {
            let left = candidates.remove(0);
            let right = candidates.remove(0);
            let n_left = left.clone();
            let op = operators.remove(0).0;
            if op.eq("=") {
                if let FSRToken::Variable(name) = left {
                    return Ok((
                        FSRToken::Assign(FSRAssign {
                            left: Rc::new(n_left),
                            name: name.get_name(),
                            expr: Rc::new(right),
                            len: start + length,
                            meta,
                        }),
                        start + length,
                    ));
                }
            }
            return Ok((
                FSRToken::Expr(Self {
                    single_op: None,
                    left: Box::new(left),
                    right: Box::new(right),
                    op: Some(op),
                    len: start + length,
                    meta,
                }),
                start + length,
            ));
        }

        if (&candidates).len().eq(&1) {
            if operators.len() != 0 {
                let mut sub_meta = meta.clone();
                sub_meta.offset = meta.offset + operators[0].1;
                let err = SyntaxError::new_with_type(
                    &sub_meta,
                    format!("Must have second candidates with {}", operators[0].0),
                    SyntaxErrType::OperatorError,
                );
                return Err(err);
            }
            let c = candidates.remove(0);
            return Ok((c, start + length));
        }

        for operator in operators {
            let split_offset = operator.1;
            let mut sub_meta = meta.clone();
            sub_meta.offset = 0 + meta.offset;
            let left = FSRExpr::parse(&source[0..split_offset], false, sub_meta)?.0;
            let mut sub_meta = meta.clone();
            sub_meta.offset = 0 + meta.offset;
            let right = FSRExpr::parse(&source[split_offset + 1..], false, sub_meta.clone())?.0;
            let n_left = left.clone();
            if operator.0.eq("=") {
                if let FSRToken::Variable(name) = left {
                    return Ok((
                        FSRToken::Assign(FSRAssign {
                            left: Rc::new(n_left),
                            name: name.get_name(),
                            expr: Rc::new(right),
                            len: start + length,
                            meta,
                        }),
                        start + length,
                    ));
                } else {
                    return Ok((
                        FSRToken::Assign(FSRAssign {
                            left: Rc::new(n_left),
                            name: "",
                            expr: Rc::new(right),
                            len: start + length,
                            meta,
                        }),
                        start + length,
                    ));
                }
            }
            return Ok((
                FSRToken::Expr(Self {
                    single_op: None,
                    left: Box::new(left),
                    right: Box::new(right),
                    op: Some(operator.0),
                    len: start + length,
                    meta,
                }),
                start + length,
            ));
        }

        return Err(SyntaxError::new(&meta, "".to_string()));
    }

    pub fn get_op(&self) -> &str {
        return self.op.unwrap();
    }
}
