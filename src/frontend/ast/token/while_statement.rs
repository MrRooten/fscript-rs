use crate::frontend::ast::parse::ASTParser;
use crate::frontend::ast::token::block::FSRBlock;
use crate::frontend::ast::token::expr::FSRExpr;
use crate::utils::error::SyntaxError;

use super::base::FSRPosition;
use super::base::FSRToken;
use super::ASTContext;

#[derive(Debug, Clone)]
pub struct FSRWhile {
    pub(crate) test: Box<FSRToken>,
    pub(crate) body: Box<FSRBlock>,
    pub(crate) len: usize,
    pub(crate) meta: FSRPosition,
}

#[derive(PartialEq, Clone)]
enum State {
    SingleQuote,
    DoubleQuote,
    _EscapeNewline,
    EscapeQuote,
    Continue,
}

impl FSRWhile {
    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }

    pub fn get_test(&self) -> &FSRToken {
        &self.test
    }

    pub fn get_block(&self) -> &FSRBlock {
        &self.body
    }

    pub fn parse(
        source: &[u8],
        meta: FSRPosition,
        context: &mut ASTContext,
    ) -> Result<Self, SyntaxError> {
        let s = std::str::from_utf8(&source[0..5]).unwrap();
        if source.len() < 5 {
            unimplemented!()
        }
        if s != "while" {
            let mut sub_meta = context.new_pos();
            let err = SyntaxError::new(&sub_meta, "not while token");
            return Err(err);
        }


        if source[5] as char != ' ' && source[5] as char != '(' {
            let mut sub_meta = context.new_pos();
            let err = SyntaxError::new(&sub_meta, "not a valid while delemiter");
            return Err(err);
        }

        let mut state = State::Continue;
        let mut pre_state = State::Continue;
        let mut len = 0;
        for c in &source["while".len()..] {
            let c = *c as char;
            len += 1;
            if c == '{' && (state != State::DoubleQuote && state != State::SingleQuote) {
                len -= 1;
                break;
            }

            if c == '\n' {
                let mut sub_meta = context.new_pos();
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
        let test_meta = context.new_pos();
        let test_expr = FSRExpr::parse(test, false, test_meta, context)?.0;

        let start = 5 + len;
        let mut sub_meta = context.new_pos();
        let b_len = ASTParser::read_valid_bracket(&source[start..], sub_meta, &context)?;
        let mut sub_meta = context.new_pos();
        let body = FSRBlock::parse(&source[start..start + b_len], sub_meta, context)?;

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
