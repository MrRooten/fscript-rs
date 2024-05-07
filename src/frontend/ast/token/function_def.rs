#![allow(unused)]

use std::rc::Rc;

use crate::{
    frontend::ast::{
        parse::ASTParser,
        token::{
            block::FSRBlock, call::FSRCall,
        },
    }, utils::error::SyntaxError,
};

use super::base::{FSRMeta, FSRToken};

#[derive(Debug, Clone)]
pub struct FSRFnDef<'a> {
    name: &'a str,
    args: Vec<FSRToken<'a>>,
    body: Rc<FSRBlock<'a>>,
    len: usize,
    meta: FSRMeta
}

#[derive(PartialEq, Clone)]
enum State {
    SingleQuote,
    DoubleQuote,
    EscapeNewline,
    EscapeQuote,
    Continue,
}

impl<'a> FSRFnDef<'a> {
    pub fn get_meta(&self) -> &FSRMeta {
        &self.meta
    }

    pub fn get_name(&self) -> &'a str {
        self.name
    }

    pub fn get_len(&self) -> usize {
        self.len
    }

    pub fn get_args(&self) -> &Vec<FSRToken<'a>> {
        &self.args
    }

    pub fn get_body(&self) -> &FSRBlock<'a> {
        &self.body
    }

    pub fn parse(source: &'a [u8], meta: FSRMeta) -> Result<Self, SyntaxError> {
        let s = unsafe { std::str::from_utf8_unchecked(&source[0..2]) };
        if source.len() < 3 {
            let mut sub_meta = meta.clone();
            sub_meta.offset = meta.offset;
            let err = SyntaxError::new(&sub_meta, "fn define body length too small");
            return Err(err);
        }
        if s != "fn" {
            let mut sub_meta = meta.clone();
            sub_meta.offset = meta.offset;
            let err = SyntaxError::new(&sub_meta, "not fn token");
            return Err(err);
        }

        if source[2] as char != ' ' {
            let mut sub_meta = meta.clone();
            sub_meta.offset = meta.offset + 2;
            let err = SyntaxError::new(&sub_meta, "not a valid fn delemiter");
            return Err(err);
        }

        let mut state = State::Continue;
        let mut pre_state = State::Continue;
        let mut len = 0;
        for c in &source[2..] {
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

        let fn_args = &source[2..2 + len];
        let mut sub_meta = meta.clone();
        sub_meta.offset = meta.offset + 2;
        let fn_args = FSRCall::parse(fn_args, sub_meta)?;

        let name = fn_args.get_name();

        let fn_block_start = 2 + len;
        let mut sub_meta = meta.clone();
        sub_meta.offset = meta.offset + fn_block_start;
        let fn_block_len = ASTParser::read_valid_bracket(&source[fn_block_start..], sub_meta.clone())?;
        let block_meta = sub_meta.clone();
        let fn_block =
            FSRBlock::parse(&source[fn_block_start..fn_block_start + fn_block_len], block_meta)?;

        Ok(Self {
            name,
            args: fn_args.get_args().clone(),
            body: Rc::new(fn_block),
            len: fn_block_start + fn_block_len,
            meta
        })
    }
}
