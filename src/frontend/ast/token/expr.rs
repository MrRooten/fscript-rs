use std::fmt::Error;
use std::str;
use crate::frontend::ast::{parse::ASTParser, token::constant::{FSRConstant, FSRConstantType}};

use super::base::FSRToken;

pub struct FSRExpr<'a> {
    value       : Box<FSRToken<'a>>,
    len         : usize
}

#[derive(PartialEq, Copy, Clone)]
enum ExprState {
    ExprStart,
    EscapeNewline,
    String,
    Number,
    Float,
    EndToken,
    Function,
    Operator,
    WaitToken
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

impl Operator {
    pub fn parse(s: &str) -> Result<Operator, &str> {
        unimplemented!()
    }
}

struct Node<'a> {
    op      : Operator,
    left    : *mut Node<'a>,
    right   : *mut Node<'a>,
    value   : Option<FSRToken<'a>>,
    is_leaf : bool
}

type FSROpreatorTree<'a> = Node<'a>;

impl FSROpreatorTree<'_> {
    
}

impl FSRExpr<'_> {
    pub fn is_op_one_char(op: char) -> bool {
        unimplemented!()
    }

    pub fn parse(source: &[u8]) -> Result<Self, &str> {
        let mut pre_state = ExprState::ExprStart;
        let mut state = ExprState::ExprStart;
        let mut start = 0;
        let mut length = 0;
        let mut len = 0;
        let mut t = vec![];
        let mut operators = vec![];
        loop {
            let c = source[start + length];
            len += 1;

            if c as char == '\\' {
                start += 1;
                pre_state = state;
                state = ExprState::EscapeNewline;
                continue;
            }

            if state != ExprState::EscapeNewline && c as char == '\n' {
                start += 1;
                len += 1;
                break;
            }



            if state == ExprState::EscapeNewline && c as char == '\n' {
                start += 1;
                state = pre_state;
                continue;
            }

            if state == ExprState::EscapeNewline && c as char != '\n' {
                return Err("not new line char");
            }

            if state == ExprState::WaitToken && ASTParser::is_blank_char(c) {
                start += 1;
                continue;
            }

            if state == ExprState::WaitToken && c as char == '"' {
                state = ExprState::String;
                length += 1;
                continue;
            }

            if state == ExprState::String && c as char != '"' {
                length += 1;
                continue;
            }

            if state == ExprState::String && c as char == '"' {
                let string = str::from_utf8(&source[start..start+length]).unwrap();
                let s = FSRConstant::from_str(string);
                t.push(s);
            }

            if state == ExprState::WaitToken && (c as char).is_digit(10) {
                state = ExprState::Number;
                length += 1;
                continue;
            }

            if state == ExprState::Number && (c as char).is_digit(10) {
                length += 1;
                continue;
            }

            if state == ExprState::Number && (c as char).is_digit(10) == false && (c as char) == '.' {
                state = ExprState::Float;
                length += 1;
                continue;
            }

            if state == ExprState::Number && (c as char).is_digit(10) == false && (c as char) != '.' {
                state = ExprState::WaitToken;
                let parse_int = str::from_utf8(&source[start..start+length]).unwrap();
                let i = parse_int.parse::<i64>().unwrap();
                let i = FSRConstant::from_int(i);
                t.push(i);
                continue;
            }

            if state == ExprState::Float && (c as char).is_digit(10) == false {
                state = ExprState::WaitToken;
                let parse_int = str::from_utf8(&source[start..start+length]).unwrap();
                let f = parse_int.parse::<f64>().unwrap();
                let i = FSRConstant::from_float(f);
                t.push(i);
                continue;
            }

            if state == ExprState::WaitToken && Self::is_op_one_char(c as char) {
                state = ExprState::Operator;
                length += 1;
                continue;
            }

            if state == ExprState::Operator && Self::is_op_one_char(c as char) {
                length += 1;
                continue;
            } else if state == ExprState::Operator {
                let op_s = str::from_utf8(&source[start..start+length]).unwrap();
                let op = Operator::parse(op_s).unwrap();
                operators.push(op);
            }

        }
        unimplemented!()
    }

    pub fn parse_len(&self) -> usize {
        unimplemented!()
    }
}