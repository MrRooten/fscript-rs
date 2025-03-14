#![allow(unused)]

use std::rc::Rc;

use crate::{
    frontend::ast::{parse::ASTParser, token::expr::FSRExpr},
    utils::error::SyntaxError,
};

use super::base::{FSRPosition, FSRToken};

#[derive(Debug, Clone)]
pub struct FSRAssign<'a> {
    pub(crate) expr: Rc<FSRToken<'a>>,
    pub(crate) left: Rc<FSRToken<'a>>,
    pub(crate) name: &'a str,
    pub(crate) len: usize,
    pub(crate) meta: FSRPosition,
}

#[derive(PartialEq)]
enum FSRAssignState {
    Start,
    IdKeyLetStart,
    IdKeyLetEnd,
    NameStart,
    NameEnd,
    LeftValue,
    RightValue,
}

impl<'a> FSRAssign<'a> {
    pub fn get_left(&self) -> &Rc<FSRToken<'a>> {
        &self.left
    }

    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }

    pub fn get_name(&self) -> &str {
        self.name
    }

    pub fn get_assign_expr(&self) -> &Rc<FSRToken<'a>> {
        &self.expr
    }

    pub fn get_len(&self) -> usize {
        self.len
    }
    pub fn parse(source: &'a [u8], meta: &FSRPosition) -> Result<FSRAssign<'a>, SyntaxError> {
        let mut start = 0;
        let mut length = 0;
        let mut state = FSRAssignState::Start;
        let mut name: Option<&[u8]> = None;
        let mut value: Option<Box<FSRToken>> = None;
        let mut len = 0;
        while start + length < source.len() {
            let c = source[start + length];
            len += 1;
            if ASTParser::is_blank_char_with_new_line(c) && state == FSRAssignState::Start {
                start += 1;
                length = 0;
                continue;
            }

            if ASTParser::is_name_letter_first(c) && state == FSRAssignState::Start {
                state = FSRAssignState::NameStart;
                length += 1;
                continue;
            }

            if ASTParser::is_name_letter_first(c) && state == FSRAssignState::NameStart {
                length += 1;
                continue;
            }

            if (ASTParser::is_blank_char_with_new_line(c) || c as char == '=')
                && state == FSRAssignState::NameStart
            {
                state = FSRAssignState::NameEnd;
                name = Some(&source[start..start + length]);
            }

            if c as char == '='
                && (state == FSRAssignState::NameStart || state == FSRAssignState::NameEnd)
            {
                start += length;
                length = 0;
                state = FSRAssignState::RightValue;
                continue;
            }

            if state == FSRAssignState::RightValue {
                let mut sub_meta = meta.from_offset(start);
                let expr = FSRExpr::parse(&source[start..], false, sub_meta)?;
                if let FSRToken::Expr(e) = &expr.0 {
                    len += expr.1;
                }
                value = Some(Box::new(expr.0));

                break;
            }

            start += 1;
        }
        unimplemented!()
    }
}
