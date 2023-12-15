use std::fmt::Display;

use std::str;
use crate::frontend::ast::{parse::ASTParser, token::constant::{FSRConstant, FSRConstantType}};

use super::base::FSRToken;

#[derive(Debug)]
pub struct FSRBinOp<'a> {
    left        : Box<Option<FSRToken<'a>>>,
    right       : Box<Option<FSRToken<'a>>>,
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

impl<'a> FSRBinOp<'a> {
    pub fn is_op_one_char(op: char) -> bool {
        if op == '+' || op == '-' || op == '=' || op == '>' || op == '<' {
            return true;
        }

        return false;
    }

    pub fn parse(source: &'a [u8]) -> Result<Self, &str> {
        let mut pre_state = ExprState::ExprStart;
        let mut state = ExprState::WaitToken;
        let mut start = 0;
        let mut length = 0;
        let mut len = 0;
        let mut left: Option<FSRToken> = None;
        let mut right: Option<FSRToken> = None;
        let mut operator: Option<&str> = None;
        loop {
            if start + length >= source.len() {
                break;
            }
            let i = source[start];
            let c = i as char;
            let t_c = source[start+length];

            
            if i as char == '\\' {
                start += 1;
                pre_state = state;
                state = ExprState::EscapeNewline;
                continue;
            }

            if state != ExprState::EscapeNewline && i as char == '\n' {
                start += 1;
                len += 1;
                break;
            }



            if state == ExprState::EscapeNewline && i as char == '\n' {
                start += 1;
                state = pre_state;
                continue;
            }

            if state == ExprState::EscapeNewline && i as char != '\n' {
                return Err("not new line char");
            }

            if state == ExprState::WaitToken && ASTParser::is_blank_char(i) {
                start += 1;
                length = 0;
                continue;
            }

            if state == ExprState::WaitToken && i as char == '"' {
                state = ExprState::DoubleString;
                start += 1;
                length = 0;
                continue;
            }

            if state == ExprState::WaitToken && i as char == '\'' {
                state = ExprState::SingleString;
                start += 1;
                length = 0;
                continue;
            }

            if state == ExprState::DoubleString && source[start+length] as char != '"' {
                length += 1;
                continue;
            }

            if state == ExprState::SingleString && source[start+length] as char != '\'' {
                length += 1;
                continue;
            }

            if state == ExprState::DoubleString && source[start+length] as char == '"' {
                let string = str::from_utf8(&source[start..start+length]).unwrap();
                let s = FSRConstant::from_str(string);
                left = Some(FSRToken::Constant(s));
            }

            if state == ExprState::SingleString && source[start+length] as char == '\'' {
                let string = str::from_utf8(&source[start..start+length]).unwrap();
                let s = FSRConstant::from_str(string);
                left = Some(FSRToken::Constant(s));
            }


            if state == ExprState::WaitToken && c == '(' {
                state = ExprState::Bracket;
                start += 1;
                length = 0;
                continue;
            }

            if state == ExprState::Bracket && source[start+length] as char != ')' {
                length += 1;
            }

            if state == ExprState::Bracket && source[start+length] as char == ')' {
                let sub_expr = &source[start..start+length];
                let sub_expr = FSRBinOp::parse(sub_expr).unwrap();
                left = Some(FSRToken::Expr(sub_expr));
            }

            if state == ExprState::WaitToken && (t_c as char).is_digit(10) {
                state = ExprState::Number;
                length = 1;
                continue;
            }

            if state == ExprState::Number && (t_c as char).is_digit(10) {
                length += 1;
                continue;
            }


            if state == ExprState::Number && (t_c as char).is_digit(10) == false && (t_c as char) == '.' {
                state = ExprState::Float;
                length += 1;
                continue;
            }

            if state == ExprState::Number && (t_c as char).is_digit(10) == false && (t_c as char) != '.' {
                state = ExprState::WaitToken;
                let parse_int = str::from_utf8(&source[start..start+length]).unwrap();
                let i = parse_int.parse::<i64>().unwrap();
                let i = FSRConstant::from_int(i);
                left = Some(FSRToken::Constant(i));
                start = start + length;
                continue;
            }

            if state == ExprState::Float && (t_c as char).is_digit(10) == false {
                state = ExprState::WaitToken;
                let parse_int = str::from_utf8(&source[start..start+length]).unwrap();
                let f = parse_int.parse::<f64>().unwrap();
                let i = FSRConstant::from_float(f);
                left = Some(FSRToken::Constant(i));
                continue;
            }

            if state == ExprState::WaitToken && ASTParser::is_name_letter(i) {
                state = ExprState::Variable;
                length = 1;
                continue;
            }

            if state == ExprState::WaitToken && Self::is_op_one_char(i as char) {
                state = ExprState::Operator;
                start = start + length;
                length = 1;
                continue;
            }

            if state == ExprState::Operator && Self::is_op_one_char(t_c as char) {
                length += 1;
                continue;
            } else if state == ExprState::Operator {
                let op_s = str::from_utf8(&source[start..start+length]).unwrap();
                // let op = Operator::parse(op_s).unwrap();
                operator = Some(op_s);
                state = ExprState::WaitToken;
                start = start + length;
                length = 0;
                let expr = FSRBinOp::parse(&source[start..]).unwrap();
                right = Some(FSRToken::Expr(expr));
                break;
            }

            

        }
        

        return Ok(Self {
            left: Box::new(left),
            right: Box::new(right),
            op: operator,
            len: 0,
        });
    }

    pub fn parse_len(&self) -> usize {
        unimplemented!()
    }
}