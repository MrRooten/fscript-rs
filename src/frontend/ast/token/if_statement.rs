

use crate::frontend::ast::parse::ASTParser;
use crate::frontend::ast::token::block::FSRBlock;
use crate::frontend::ast::token::expr::FSRExpr;
use crate::utils::error::SyntaxError;

use super::base::FSRPosition;
use super::base::FSRToken;
use super::r#else::FSRElse;
use super::statement::ASTTokenEnum;
use super::statement::ASTTokenInterface;
use super::ASTContext;

#[derive(Debug, Clone)]
pub struct FSRIf {
    pub(crate) test: Box<FSRToken>,
    pub(crate) body: Box<FSRBlock>,
    #[allow(unused)]
    elses        : Option<Box<FSRElse>>,
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

impl FSRIf {
    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }

    pub fn get_test(&self) -> &FSRToken {
        &self.test
    }

    pub fn get_block(&self) -> &FSRBlock {
        &self.body
    }

    pub fn parse_without_else(source: &[u8], meta: FSRPosition, context: &mut ASTContext) -> Result<FSRIf, SyntaxError> {
        let s = std::str::from_utf8(&source[0..2]).unwrap();
        if source.len() < 3 {
            let sub_meta = meta.from_offset(0);
            let err = SyntaxError::new(&sub_meta, "if define body length too small");
            return Err(err);
        }
        if s != "if" {
            let sub_meta = meta.from_offset(0);
            let err = SyntaxError::new(&sub_meta, "not if token");
            return Err(err);
        }

        if source[2] as char != ' ' && source[2] as char != '(' {
            let sub_meta = meta.from_offset(2);
            let err = SyntaxError::new(&sub_meta, "not a valid if delemiter");
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

        let test = &source[2..2 + len];
        let mut sub_meta = meta.clone();
        sub_meta.offset = meta.offset + 2;
        let test_expr = FSRExpr::parse(test, false, sub_meta, context)?.0;

        let mut start = 2 + len;
        let mut sub_meta = meta.clone();
        sub_meta.offset = meta.offset + start;
        let mut b_len = ASTParser::read_valid_bracket(&source[start..], sub_meta)?;
        let mut sub_meta = meta.clone();
        sub_meta.offset = meta.offset + start;
        let body = FSRBlock::parse(&source[start..start + b_len], sub_meta, context)?;

        start += b_len;
        b_len = 0;

        Ok(Self {
            test: Box::new(test_expr),
            body: Box::new(body),
            len: start + b_len,
            elses: None,
            meta,
        })
    }

    pub fn parse(source: &[u8], meta: FSRPosition, context: &mut ASTContext) -> Result<FSRIf, SyntaxError> {
        let s = std::str::from_utf8(&source[0..2]).unwrap();
        if source.len() < 3 {
            let sub_meta = meta.from_offset(0);
            let err = SyntaxError::new(&sub_meta, "if define body length too small");
            return Err(err);
        }
        if s != "if" {
            let sub_meta = meta.from_offset(0);
            let err = SyntaxError::new(&sub_meta, "not if token");
            return Err(err);
        }

        if source[2] as char != ' ' && source[2] as char != '(' {
            let sub_meta = meta.from_offset(2);
            let err = SyntaxError::new(&sub_meta, "not a valid if delemiter");
            return Err(err);
        }
        let sub_meta = meta.from_offset(2);
        let len = ASTParser::read_valid_bracket_until_big(&source[2..], sub_meta)?;

        let test = &source[2..2 + len];
        let mut sub_meta = meta.clone();
        sub_meta.offset = meta.offset + 2;
        let test_expr = FSRExpr::parse(test, false, sub_meta, context)?.0;

        let mut start = 2 + len;
        let mut sub_meta = meta.clone();
        sub_meta.offset = meta.offset + start;
        let mut b_len = ASTParser::read_valid_bracket(&source[start..], sub_meta)?;
        let mut sub_meta = meta.clone();
        sub_meta.offset = meta.offset + start;
        let body = FSRBlock::parse(&source[start..start + b_len], sub_meta, context)?;

        start += b_len;
        b_len = 0;
        while start < source.len() && ASTParser::is_blank_char_with_new_line(source[start]) {
            start += 1;
        }

        let mut may_else = None;

        if start + 4 < source.len() {
            let may_else_token = std::str::from_utf8(&source[start..start+4]).unwrap();
            if may_else_token.eq("else") {
                let sub_meta = meta.from_offset(start);
                let elses = FSRElse::parse(&source[start..], sub_meta, context)?;
                start += elses.get_len();
                may_else = Some(Box::new(elses));
            }
        }
        Ok(Self {
            test: Box::new(test_expr),
            body: Box::new(body),
            len: start + b_len,
            elses: may_else,
            meta,
        })
    }

    pub fn get_len(&self) -> usize {
        self.len
    }

    pub fn get_elses(&self) -> Option<&FSRElse> {
        match &self.elses {
            Some(s) => Some(s),
            None => {
                None
            }
        }
    }
}

mod test {
    #[test]
    fn test1() {
        let soruce = r#"if a < b.len() {

}"#;
        let meta = super::FSRPosition::new();
        let mut context = super::ASTContext::new_context();
        let if_token = super::FSRIf::parse(soruce.as_bytes(), meta, &mut context).unwrap();
        assert_eq!(if_token.get_len(), soruce.len());
    }
}
