use std::fmt::Display;

use std::str;
use crate::frontend::ast::{parse::ASTParser, token::constant::{FSRConstant, FSRConstantType}};

use super::{base::FSRToken, variable::{FSRVariable, self}, call::FSRCall};

#[derive(Debug)]
pub struct FSRBinOp<'a> {
    left        : Option<Box<FSRToken<'a>>>,
    right       : Option<Box<FSRToken<'a>>>,
    op          : Option<&'a str>,
    len         : usize
}

impl Display for FSRBinOp<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(PartialEq, Copy, Clone)]
enum ExprState {
    ExprStart,
    EscapeNewline,
    DoubleString,
    SingleString,
    Number,
    Float,
    EndToken,
    Function,
    Operator,
    WaitToken,
    Variable,
    Bracket
}

struct ExprStates {
    states      : Vec<ExprState>
}

impl ExprStates {
    pub fn new() -> Self {
        return Self { states: vec![] }
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
        &self.states[self.states.len()]
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

#[derive(Debug)]
pub enum FSRBinOpResult<'a> {
    BinOp(FSRBinOp<'a>),
    Constant(FSRConstant<'a>),
}



impl<'a> FSRBinOp<'a> {
    pub fn is_op_one_char(op: char) -> bool {
        if op == '+' || op == '-' || op == '=' || op == '>' || op == '<' {
            return true;
        }

        return false;
    }

    

    pub fn parse(source: &'a [u8]) -> Result<(Box<FSRToken>, usize), &str> {
        let s = str::from_utf8(source).unwrap();
        let mut pre_state = ExprState::ExprStart;
        let mut states = ExprStates::new();
        let mut start = 0;
        let mut length = 0;
        let mut len = 0;
        let mut left: Option<Box<FSRToken>> = None;
        let mut right: Option<Box<FSRToken>> = None;
        let mut operator: Option<&str> = None;

        loop {
            if start + length >= source.len() {
                break;
            }
            let i = source[start];
            let c = i as char;
            let t_i = source[start+length];
            let t_c = t_i as char;

            
            if i as char == '\\' {
                start += 1;
                states.push_state(ExprState::EscapeNewline);
                len += 1;
                continue;
            }

            if states.peek().eq(&ExprState::EscapeNewline) && i as char == '\n' {
                start += 1;
                len += 1;
                break;
            }



            if states.peek().eq(&ExprState::EscapeNewline) && i as char == '\n' {
                start += 1;
                states.pop_state();
                len += 1;
                continue;
            }

            if states.peek().eq(&ExprState::EscapeNewline) && i as char != '\n' {
                return Err("not new line char");
            }

            if states.peek().eq(&ExprState::WaitToken) && ASTParser::is_blank_char(i) {
                start += 1;
                length = 0;
                len += 1;
                continue;
            }

            if states.peek().eq(&ExprState::WaitToken) && i as char == '"' {
                states.set_up_state(ExprState::DoubleString);
                start += 1;
                length = 0;
                len += 1;
                continue;
            }

            if states.peek().eq(&ExprState::WaitToken) && i as char == '\'' {
                states.push_state(ExprState::SingleString);
                start += 1;
                length = 0;
                len += 1;
                continue;
            }

            if states.peek().eq(&ExprState::DoubleString) && source[start+length] as char != '"' {
                length += 1;
                len += 1;
                continue;
            }

            if states.peek().eq(&ExprState::SingleString) && source[start+length] as char != '\'' {
                length += 1;
                len += 1;
                continue;
            }

            if states.peek().eq(&ExprState::DoubleString) && source[start+length] as char == '"' {
                let string = str::from_utf8(&source[start..start+length]).unwrap();
                let s = FSRConstant::from_str(string);
                left = Some(Box::new(FSRToken::Constant(s)));
                start = start + length + 1;
                len += 1;
                length = 0;
                states.pop_state();
                continue;
            }

            if states.peek().eq(&ExprState::SingleString) && source[start+length] as char == '\'' {
                let string = str::from_utf8(&source[start..start+length]).unwrap();
                let s = FSRConstant::from_str(string);
                left = Some(Box::new(FSRToken::Constant(s)));
                start = start + length + 1;
                len += 1;
                length = 0;
                states.pop_state();
                continue;
            }

            if states.peek().eq(&ExprState::Variable) && t_i as char == '(' {
                states.set_up_state(ExprState::Function);
                let call = FSRCall::parse(&source[start..]).unwrap();
                length += call.parse_len();
                len += call.parse_len();
                left = Some(Box::new(FSRToken::Call(call)));
                states.set_up_state(ExprState::WaitToken);
                continue;
            }

            if states.peek().eq(&ExprState::WaitToken) && c == '(' {
                states.set_up_state(ExprState::Bracket);
                start += 1;
                length = 0;
                len += 1;
                continue;
            }

            if state == ExprState::Bracket && source[start+length] as char != ')' {
                length += 1;
                continue;
            }

            if state == ExprState::Bracket && source[start+length] as char == ')' {
                let sub_expr = &source[start..start+length];
                let sub_expr = FSRBinOp::parse(sub_expr).unwrap();
                len += sub_expr.1;
                left = Some(sub_expr.0);
                start = start + length + 1;
                length = 0;
                len += 1;
                state = ExprState::WaitToken;
                continue;
            }

            if state == ExprState::WaitToken && (t_i as char).is_digit(10) {
                state = ExprState::Number;
                length = 0;
                continue;
            }

            if state == ExprState::Number && (t_i as char).is_digit(10) == false && (t_i as char) == '.' {
                state = ExprState::Float;
                length += 1;
                len += 1;
                continue;
            }

            if state == ExprState::Number && (t_i as char).is_digit(10) {
                length += 1;
                len += 1;
                if start + length >= source.len() {
                    let parse_int = str::from_utf8(&source[start..start+length]).unwrap();
                    let i = parse_int.parse::<i64>().unwrap();
                    let i = FSRConstant::from_int(i);
                    left = Some(Box::new(FSRToken::Constant(i)));
                    start = start + length;
                    continue;
                }
                continue;
            }

            if state == ExprState::Number && ((t_i as char).is_digit(10) == false && (t_i as char) != '.') {
                state = ExprState::WaitToken;
                let parse_int = str::from_utf8(&source[start..start+length]).unwrap();
                let i = parse_int.parse::<i64>().unwrap();
                let i = FSRConstant::from_int(i);
                left = Some(Box::new(FSRToken::Constant(i)));
                start = start + length;
                length = 0;
                continue;
            }

            if state == ExprState::Float && (t_i as char).is_digit(10) == false {
                state = ExprState::WaitToken;
                let parse_int = str::from_utf8(&source[start..start+length]).unwrap();
                let f = parse_int.parse::<f64>().unwrap();
                let i = FSRConstant::from_float(f);
                left = Some(Box::new(FSRToken::Constant(i)));
                continue;
            }

            if state == ExprState::WaitToken && ASTParser::is_name_letter(i) {
                state = ExprState::Variable;
                length = 0;
                continue;
            }

            if state == ExprState::Variable && ASTParser::is_name_letter(t_i) {
                length += 1;
                len += 1;
                if start + length >= source.len() {
                    let name = str::from_utf8(&source[start..start+length]).unwrap();
                    let variable = FSRVariable::parse(name).unwrap();
                    left = Some(Box::new(FSRToken::Variable(variable)));
                    start = start + length;
                    continue;
                }
            }

            if state == ExprState::Variable && ASTParser::is_name_letter(t_i) == false {
                state = ExprState::WaitToken;
                let name = str::from_utf8(&source[start..start+length]).unwrap();
                let variable = FSRVariable::parse(name).unwrap();
                left = Some(Box::new(FSRToken::Variable(variable)));
                start = start + length;
                length = 0;
                continue;
            }



            if state == ExprState::WaitToken && Self::is_op_one_char(i as char) {
                state = ExprState::Operator;
                start = start + length;
                length = 0;
                continue;
            }

            if state == ExprState::Operator && Self::is_op_one_char(t_i as char) {
                length += 1;
                len += 1;
                continue;
            } else if state == ExprState::Operator {
                let op_s = str::from_utf8(&source[start..start+length]).unwrap();
                // let op = Operator::parse(op_s).unwrap();
                operator = Some(op_s);
                state = ExprState::WaitToken;
                start = start + length;
                length = 0;
                let expr = FSRBinOp::parse(&source[start..]).unwrap();
                len += expr.1;
                right = Some(expr.0);
                break;
            }

            

        }
        
        if right.is_none() {
            match left {
                Some(s) => return Ok((s, len)),
                None => {
                    return Err("error");
                }
            }
        } else {
            return Ok((Box::new(FSRToken::Expr(Self {
                left: left,
                right: right,
                op: operator,
                len: len,
            })), len));
        }

    }

    pub fn parse_len(&self) -> usize {
        return self.len;
    }
}