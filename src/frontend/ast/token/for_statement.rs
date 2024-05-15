use crate::{frontend::ast::{parse::ASTParser, token::expr::FSRExpr}, utils::error::SyntaxError};

use super::{base::{FSRPosition, FSRToken}, block::FSRBlock};

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct FSRFor<'a> {
    var_name: String,
    expr: Box<FSRToken<'a>>,
    body: Box<FSRBlock<'a>>,
    len: usize,
    meta: FSRPosition,
}

#[derive(PartialEq, Clone)]
enum State {
    SingleQuote,
    DoubleQuote,
    _EscapeNewline,
    EscapeQuote,
    Continue,
}

impl<'a> FSRFor<'a> {
    pub fn get_len(&self) -> usize {
        self.len
    }

    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }

    pub fn get_var_name(&self) -> &str {
        &self.var_name
    }

    pub fn get_expr(&self) -> &FSRToken {
        &self.expr
    }

    pub fn get_block(&self) -> &FSRBlock<'a> {
        &self.body
    }

    pub fn parse(source: &'a [u8], meta: FSRPosition) -> Result<Self, SyntaxError> {
        let s = unsafe { std::str::from_utf8_unchecked(&source[0..3]) };
        
        if s != "for" {
            let mut sub_meta = meta.clone();
            sub_meta.offset = meta.offset;
            let err = SyntaxError::new(&sub_meta, "not for token");
            return Err(err);
        }
        
        if !ASTParser::is_blank_char(source[3]){
            let mut sub_meta = meta.from_offset(3);
            sub_meta.offset = meta.offset;
            let err = SyntaxError::new(&sub_meta, "blank space after for token");
            return Err(err);
        }

        let mut start = 3;
        while start < source.len() && ASTParser::is_blank_char(source[start]) {
            start += 1;
        }

        let mut name = String::new();
        if !ASTParser::is_name_letter_first(source[start]) {
            let mut sub_meta = meta.from_offset(start);
            sub_meta.offset = meta.offset;
            let err = SyntaxError::new(&sub_meta, "variable name not name letter first");
            return Err(err);
        }
        name.push(source[start] as char);
        start += 1;
        
        while start < source.len() && ASTParser::is_name_letter(source[start]) {
            name.push(source[start] as char);
            start += 1;
        }

        if !ASTParser::is_blank_char(source[start]) {
            let mut sub_meta = meta.from_offset(start);
            sub_meta.offset = meta.offset;
            let err = SyntaxError::new(&sub_meta, "blank space after for token");
            return Err(err);
        }

        start += 1;
        while start < source.len() && ASTParser::is_blank_char(source[start]) {
            start += 1;
        }

        let s = unsafe { std::str::from_utf8_unchecked(&source[start..start+2]) };
        if !s.eq("in") {
            let mut sub_meta = meta.from_offset(start);
            sub_meta.offset = meta.offset;
            let err = SyntaxError::new(&sub_meta, "in after variable in for statement");
            return Err(err);
        }

        start += 2;

        if !ASTParser::is_blank_char(source[start]) {
            let mut sub_meta = meta.from_offset(start);
            sub_meta.offset = meta.offset;
            let err = SyntaxError::new(&sub_meta, "blank space after in token");
            return Err(err);
        }

        start += 1;
        let mut len = 0;
        let mut state = State::Continue;
        let mut pre_state = State::Continue;
        for c in &source[start..] {
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

        let expr = &source[start..start + len];
        let sub_meta = meta.from_offset(start);
        let expr = FSRExpr::parse(expr, false, sub_meta)?.0;
        start += len;
        let sub_meta = meta.from_offset(start);
        let b_len = ASTParser::read_valid_bracket(&source[start..], sub_meta)?;
        let mut sub_meta = meta.clone();
        sub_meta.offset = meta.offset + start;
        let body = FSRBlock::parse(&source[start..start + b_len], sub_meta)?;
        start += body.get_len();
        
        Ok(Self {
            var_name: name,
            expr: Box::new(expr),
            body: Box::new(body),
            len: start,
            meta,
        })
    }
}