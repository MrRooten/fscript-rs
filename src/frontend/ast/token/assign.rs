#![allow(unused)]

use std::rc::Rc;

use crate::{
    frontend::ast::{parse::ASTParser, token::expr::FSRExpr},
    utils::error::SyntaxError,
};

use super::{base::{FSRPosition, FSRToken}, ASTContext};

#[derive(Debug, Clone)]
pub struct FSRAssign {
    pub(crate) expr: Rc<FSRToken>,
    pub(crate) left: Rc<FSRToken>,
    pub(crate) name: String,
    pub(crate) len: usize,
    pub(crate) meta: FSRPosition,
    pub(crate) op_assign: String,
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

impl FSRAssign {
    pub fn get_left(&self) -> &Rc<FSRToken> {
        &self.left
    }

    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_assign_expr(&self) -> &Rc<FSRToken> {
        &self.expr
    }

    pub fn get_len(&self) -> usize {
        self.len
    }
    pub fn parse(source: &[u8], meta: &FSRPosition, context: &mut ASTContext) -> Result<FSRAssign, SyntaxError> {
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
                let mut sub_meta = meta.new_offset(start);
                let expr = FSRExpr::parse(&source[start..], false, sub_meta, context)?;
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
