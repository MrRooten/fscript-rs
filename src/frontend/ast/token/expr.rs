use std::fmt::Error;

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
    EndToken,
    Function
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

}

struct Node<'a> {
    op      : Operator,
    left    : *mut Node<'a>,
    right   : *mut Node<'a>,
    value   : Option<FSRToken<'a>>,
    is_leaf : bool
}

impl FSRExpr<'_> {
    pub fn parse(source: &[u8]) -> Result<Self, &str> {
        let mut pre_state = ExprState::ExprStart;
        let mut state = ExprState::ExprStart;
        let mut start = 0;
        let mut length = 0;
        let mut len = 0;
        loop {
            let c = source[start + length];
            len += 1;

            if c as char == '\\' {
                pre_state = state;
                state = ExprState::EscapeNewline;
                continue;
            }

            if state != ExprState::EscapeNewline && c as char == '\n' {
                len += 1;
                break;
            }



            if state == ExprState::EscapeNewline && c as char == '\n' {
                state = pre_state;
                continue;
            }

            if state == ExprState::EscapeNewline && c as char != '\n' {
                return Err("not new line char");
            }
        }
        unimplemented!()
    }

    pub fn parse_len(&self) -> usize {
        unimplemented!()
    }
}