use std::{cell::Ref, fmt::Error, rc::Rc};

use crate::{frontend::ast::parse::ASTParser, utils::error::SyntaxError};
use std::str;
use super::{base::{FSRMeta, FSRToken}, expr::FSRExpr};

#[derive(Debug, Clone)]
pub struct FSRCall<'a> {
    name        : &'a str,
    args        : Vec<FSRToken<'a>>,
    pub(crate) len         : usize,
    pub(crate) single_op   : Option<&'a str>,
    meta                    : FSRMeta
}

#[derive(PartialEq)]
enum CallState {
    Name,
    Start,
    Args,
    WaitToken
}

impl<'a> FSRCall<'a> {
    pub fn get_meta(&self) -> &FSRMeta {
        return &self.meta;
    }

    pub fn get_args(&self) -> &Vec<FSRToken<'a>> {
        return &self.args;
    }

    pub fn get_name(&self) -> &'a str {
        return self.name;
    }

    pub fn parse(source: &'a [u8], meta: FSRMeta) -> Result<Self, SyntaxError> {
        let mut state = CallState::Start;
        let mut start = 0;
        let mut length = 0;
        let mut name = "";
        let mut fn_args = vec![];
        loop {
            let i = source[start];
            let t_i = source[start + length];
            if state == CallState::Start && ASTParser::is_blank_char(i) {
                start += 1;
                continue;
            }

            if ASTParser::is_name_letter(i) && state == CallState::Start {
                state = CallState::Name;
                continue;
            }

            if state == CallState::Name && ASTParser::is_name_letter(t_i) {
                length += 1;
                continue;
            }

            if state == CallState::Name && t_i as char == '(' {
                name = str::from_utf8(&source[start..start+length]).unwrap();
                state = CallState::Args;
                start = start + length;
                start += 1;
                length = 0;
                break;
            }
        }


        let s = str::from_utf8(source).unwrap();
        let first = s.find('(').unwrap();
        let last = s.rfind(')').unwrap();
        let args = &source[first+1..last];
        let mut sub_meta = meta.clone();
        sub_meta.offset += start;
        let exprs = ASTParser::split_by_comma(args, sub_meta)?;
        for s in exprs {
            let mut sub_meta = meta.clone();
            sub_meta.offset += first;
            let expr = FSRExpr::parse(s, false, sub_meta)?;
            fn_args.push(expr.0);
        }
        return Ok(
            Self {
                name,
                args: fn_args,
                len: 0,
                single_op: None,
                meta
            }
        );
    }

    pub fn get_len(&self) -> usize {
        return self.len;
    }
}