#![allow(unused)]

use crate::{frontend::ast::{parse::ASTParser, token::expr::FSRExpr}, utils::error::SyntaxError};

use super::base::{FSRMeta, FSRToken};

#[derive(Debug, Clone)]
pub struct FSRAssign<'a> {
    pub(crate) expr        : Box<FSRToken<'a>>,
    pub(crate) name        : &'a str,
    pub(crate) len         : usize,
    pub(crate) meta        : FSRMeta
}

#[derive(PartialEq)]
enum FSRAssignState {
    Start,
    IdKeyLetStart,
    IdKeyLetEnd,
    NameStart,
    NameEnd,
    LeftValue,
    RightValue
}

impl<'a> FSRAssign<'a> {

    pub fn get_meta(&self) -> &FSRMeta {
        return &self.meta;
    }

    pub fn get_name(&self) -> &str {
        return &self.name;
    }

    pub fn get_assign_expr(&self) -> &Box<FSRToken<'a>> {
        return &self.expr;
    }

    pub fn get_len(&self) -> usize {
        return self.len;
    }
    pub fn parse(source: &'a [u8], meta: &FSRMeta) -> Result<FSRAssign<'a>, SyntaxError> {
        
        let mut start = 0;
        let mut length = 0;
        let mut state = FSRAssignState::Start;
        let mut name: Option<&[u8]> = None;
        let mut value: Option<Box<FSRToken>> = None;
        let mut len = 0;
        while start + length < source.len() {
            let c = source[start + length];
            len += 1;
            if ASTParser::is_blank_char(c) && state == FSRAssignState::Start {
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

            if (ASTParser::is_blank_char(c) || c as char == '=') && state == FSRAssignState::NameStart {
                state = FSRAssignState::NameEnd;
                name = Some(&source[start..start+length]);
            }

            if c as char == '=' && (state == FSRAssignState::NameStart || state == FSRAssignState::NameEnd) {
                start = start + length;
                length = 0;
                state = FSRAssignState::RightValue;
                continue;
            }

            if state == FSRAssignState::RightValue {
                let mut sub_meta = meta.clone();
                sub_meta.offset = start + meta.offset;
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