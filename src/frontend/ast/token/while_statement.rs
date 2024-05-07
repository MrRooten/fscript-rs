use std::fmt::Error;

use crate::frontend::ast::parse::ASTParser;
use crate::frontend::ast::token::block::FSRBlock;
use crate::frontend::ast::token::expr::FSRExpr;
use crate::utils::error::SyntaxError;

use super::base::FSRMeta;
use super::base::FSRToken;
use super::statement::ASTTokenEnum;
use super::statement::ASTTokenInterface;

#[derive(Debug, Clone)]
pub struct FSRWhile<'a> {
    test: Box<FSRToken<'a>>,
    body: Box<FSRBlock<'a>>,
    len: usize,
    meta: FSRMeta,
}

#[derive(PartialEq, Clone)]
enum State {
    SingleQuote,
    DoubleQuote,
    EscapeNewline,
    EscapeQuote,
    Continue,
}

impl<'a> FSRWhile<'a> {
    pub fn get_meta(&self) -> &FSRMeta {
        &self.meta
    }

    pub fn get_test(&self) -> &FSRToken {
        &self.test
    }

    pub fn get_block(&self) -> &FSRBlock {
        &self.body
    }

    pub fn parse(source: &'a [u8], meta: FSRMeta) -> Result<Self, SyntaxError> {
        let s = unsafe { std::str::from_utf8_unchecked(&source[0..5]) };
        if source.len() < 5 {
            unimplemented!()
        }
        if s != "while" {
            let mut sub_meta = meta.clone();
            sub_meta.offset = meta.offset;
            let err = SyntaxError::new(&sub_meta, "not if token");
            return Err(err);
        }

        if source[5] as char != ' ' && source[5] as char != '(' {
            let mut sub_meta = meta.clone();
            sub_meta.offset = meta.offset + 5;
            let err = SyntaxError::new(&sub_meta, "not a valid if delemiter");
            return Err(err);
        }

        let mut state = State::Continue;
        let mut pre_state = State::Continue;
        let mut len = 0;
        for c in &source[5..] {
            let c = *c as char;
            len += 1;
            if c == '{' && (state != State::DoubleQuote && state != State::SingleQuote) {
                len -= 1;
                break;
            }

            if c == '\n' {
                let mut sub_meta = meta.clone();
                sub_meta.offset = meta.offset + len - 1;
                let err = SyntaxError::new(&sub_meta, "Invalid If statement");
                return Err(err);
            }

            if state == State::EscapeQuote {
                state = pre_state.clone();
                continue;
            }

            if c == '\'' && state == State::Continue {
                state = State::SingleQuote;
                continue;
            }

            if c == '\'' && state == State::SingleQuote {
                state = State::Continue;
                continue;
            }

            if c == '\"' && state == State::DoubleQuote {
                state = State::DoubleQuote;
                continue;
            }

            if c == '\"' && state == State::DoubleQuote {
                state = State::Continue;
                continue;
            }

            if c == '\\' && (state == State::DoubleQuote || state == State::SingleQuote) {
                pre_state = state;
                state = State::EscapeQuote;
            }
        }

        let test = &source[5..5 + len];
        let mut test_meta = meta.clone();
        test_meta.offset = meta.offset + 5;
        let test_expr = FSRExpr::parse(test, false, test_meta)?.0;

        let start = 5 + len;
        let mut sub_meta = meta.clone();
        sub_meta.offset = meta.offset + start;
        let b_len = ASTParser::read_valid_bracket(&source[start..], sub_meta)?;
        let mut sub_meta = meta.clone();
        sub_meta.offset = meta.offset + start;
        let body = FSRBlock::parse(&source[start..start + b_len], sub_meta)?;

        Ok(Self {
            test: Box::new(test_expr),
            body: Box::new(body),
            len: start + b_len,
            meta,
        })
    }

    pub fn get_len(&self) -> usize {
        self.len
    }
}
