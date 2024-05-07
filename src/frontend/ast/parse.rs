use super::token::base::FSRMeta;
use super::token::statement::{ASTToken, ASTTokenEnum};
use crate::frontend::ast::parse::BracketState::{DoubleQuote, SingleQuote};
use crate::utils::error::{SyntaxErrType, SyntaxError};
use std::fmt::Error;
use std::str;

pub struct ASTParser {
    tokens: Vec<ASTToken>,
}

type FnExpectTokens = fn() -> Vec<ASTTokenEnum>;

#[derive(PartialEq)]
pub enum BracketState {
    Parenthesis,
    Bracket,
    Braces,
    SingleQuote,
    DoubleQuote,
    EscapeQuote,
}

impl BracketState {
    pub fn is_bracket(&self) -> bool {
        self == &BracketState::Bracket
            || self == &BracketState::Braces
            || self == &BracketState::Parenthesis
    }

    pub fn is_string(&self) -> bool {
        self == &SingleQuote || self == &DoubleQuote
    }
}

pub struct BracketStates {
    states: Vec<(BracketState, usize)>,
}

impl Default for BracketStates {
    fn default() -> Self {
        Self::new()
    }
}

impl BracketStates {
    pub fn new() -> Self {
        Self { states: vec![] }
    }

    pub fn set_up_state(&mut self, new_state: BracketState, offset: usize) {
        self.states.pop();
        self.states.push((new_state, offset));
    }

    pub fn push_state(&mut self, state: BracketState, offset: usize) {
        self.states.push((state, offset));
    }

    pub fn pop_state(&mut self) {
        self.states.pop();
    }

    pub fn peek(&self) -> &(BracketState, usize) {
        &self.states[self.states.len() - 1]
    }

    pub fn eq_peek(&self, state: BracketState) -> bool {
        return self.peek().0 == state;
    }

    pub fn is_empty(&self) -> bool {
        self.states.len() == 0
    }
}

impl ASTParser {
    pub fn get_max_token_length() -> usize {
        unimplemented!()
    }

    pub fn match_token(token: &str) -> (Option<ASTToken>, bool) {
        unimplemented!()
    }

    pub fn get_fn_expect_token(token: &ASTTokenEnum) -> FnExpectTokens {
        unimplemented!()
    }

    pub fn is_blank_char_with_new_line(c: u8) -> bool {
        c as char == ' ' || c as char == '\r' || c as char == '\t' || c as char == '\n'
    }

    pub fn is_blank_char(c: u8) -> bool {
        c as char == ' ' || c as char == '\r' || c as char == '\t'
    }

    pub fn is_name_letter_first(c: u8) -> bool {
        (c as char).is_lowercase() || (c as char).is_uppercase() || (c as char) == '_'
    }

    pub fn is_name_letter(c: u8) -> bool {
        (c as char).is_lowercase()
            || (c as char).is_uppercase()
            || (c as char).is_ascii_digit()
            || (c as char) == '_'
            || (c as char) == ':'
    }

    pub fn is_token_letter(c: u8) -> bool {
        (c as char).is_lowercase() || (c as char).is_uppercase()
    }

    pub fn end_token_char(c: u8) -> bool {
        unimplemented!()
    }

    fn token_process(token: &ASTTokenEnum, source: &str) {}

    pub fn parse(source: &str) -> Result<ASTParser, Error> {
        unimplemented!()
    }

    pub fn is_end_expr(c: u8) -> bool {
        (c as char) == '\n' || (c as char) == ';'
    }

    pub(crate) fn read_valid_expr(source: &[u8]) -> usize {
        let stack: Vec<u32> = Vec::new();
        let mut index = 0;
        loop {
            if index >= source.len() {
                break;
            }
            let c = source[index];
            if stack.is_empty() && Self::is_end_expr(c) {
                index += 1;
                break;
            }
        }
        index
    }

    pub fn helper(
        c: char,
        states: &mut BracketStates,
        offset: usize,
        meta: &FSRMeta,
    ) -> Result<(), SyntaxError> {
        if (c == ')' || c == '}' || c == ']')
            && states.peek().0.is_bracket()
            && c == ')'
            && states.peek().0 == BracketState::Parenthesis
            && c == '}'
            && states.peek().0 == BracketState::Braces
            && c == ']'
            && states.peek().0 == BracketState::Bracket
        {
            let mut sub_meta = meta.clone();
            sub_meta.offset += offset;
            let err = SyntaxError::new_with_type(
                meta,
                "can not start with right bracket",
                SyntaxErrType::BracketNotMatch,
            );
            return Err(err);
        }

        if c == ')' && states.peek().0 == BracketState::Parenthesis {
            states.pop_state();
            return Ok(());
        }

        if c == '(' && (states.is_empty() || !states.peek().0.is_string()) {
            states.push_state(BracketState::Parenthesis, offset);
            return Ok(());
        }

        if c == '[' && (states.is_empty() || !states.peek().0.is_string()) {
            states.push_state(BracketState::Bracket, offset);
            return Ok(());
        }

        if c == '{' && (states.is_empty() || !states.peek().0.is_string()) {
            states.push_state(BracketState::Braces, offset);
            return Ok(());
        }

        if c == '}' && states.peek().0 == BracketState::Braces {
            states.pop_state();
            return Ok(());
        }

        if c == ']' && states.peek().0 == BracketState::Bracket {
            states.pop_state();
            return Ok(());
        }

        if (!states.is_empty() && states.peek().0.is_string()) && c == '\\' {
            states.push_state(BracketState::EscapeQuote, offset);
            return Ok(());
        }

        if !states.is_empty() && states.peek().0 == BracketState::EscapeQuote {
            states.pop_state();
            return Ok(());
        }

        if c == '\'' && (!states.is_empty() && states.peek().0 == SingleQuote) {
            states.pop_state();
            return Ok(());
        }

        if c == '\'' && (!states.is_empty() && states.peek().0.is_bracket()) {
            states.push_state(SingleQuote, offset);
            return Ok(());
        }

        if c == '"' && (!states.is_empty() && states.peek().0.is_bracket()) {
            states.push_state(DoubleQuote, offset);
            return Ok(());
        }

        if c == '"' && (!states.is_empty() && states.peek().0 == DoubleQuote) {
            states.pop_state();
            return Ok(());
        }

        Ok(())
    }
    pub fn read_valid_name_bracket(source: &[u8], meta: FSRMeta) -> Result<usize, SyntaxError> {
        let mut states = BracketStates::new();
        let mut is_start = true;
        let mut len = 0;

        for _c in source {
            let c = *_c as char;
            if !is_start && states.is_empty() {
                break;
            }

            if (c == '(' || c == '{' || c == '[') && states.is_empty() {
                is_start = false;
            }

            Self::helper(c, &mut states, len, &meta)?;
            len += 1;
        }

        if !states.is_empty() {
            let mut sub_meta = meta.clone();
            sub_meta.offset += states.peek().1;
            let err = SyntaxError::new_with_type(
                &sub_meta,
                "not found match bracket",
                SyntaxErrType::BracketNotMatch,
            );
            return Err(err);
        }
        Ok(len)
    }

    pub fn read_valid_bracket(source: &[u8], meta: FSRMeta) -> Result<usize, SyntaxError> {
        let mut states = BracketStates::new();
        let mut is_start = true;
        let mut len = 0;
        for _c in source {
            let c = *_c as char;
            if !is_start && states.is_empty() {
                break;
            }
            is_start = false;

            Self::helper(c, &mut states, len, &meta)?;
            len += 1;
        }

        if !states.is_empty() {
            let mut sub_meta = meta.clone();
            sub_meta.offset += states.peek().1;
            let err = SyntaxError::new_with_type(
                &sub_meta,
                "not found match bracket",
                SyntaxErrType::BracketNotMatch,
            );
            return Err(err);
        }
        Ok(len)
    }

    pub fn read_to_comma(source: &[u8], meta: &FSRMeta) -> Result<usize, SyntaxError> {
        let mut states = BracketStates::new();
        let mut len = 0;
        for _c in source {
            let c = *_c as char;
            if states.is_empty() && c == ',' {
                break;
            }

            Self::helper(c, &mut states, len, meta)?;
            len += 1;
        }
        Ok(len)
    }

    pub fn split_by_comma(source: &[u8], meta: FSRMeta) -> Result<Vec<&[u8]>, SyntaxError> {
        let mut i = 0;
        let meta = FSRMeta::new();
        let mut res = vec![];
        while i < source.len() {
            let c = source[i] as char;
            let len = Self::read_to_comma(&source[i..], &meta)?;
            let expr_s = &source[i..i + len];
            res.push(expr_s);
            i += len;
            i += 1;
        }

        Ok(res)
    }

    pub fn get_static_op(op: &str) -> &'static str {
        // op reference my not life longer enough, so return static str
        if op.eq(">") {
            return ">";
        } else if op.eq("<") {
            return "<";
        } else if op.eq(">=") {
            return ">=";
        } else if op.eq("<=") {
            return "<=";
        } else if op.eq("==") {
            return "==";
        } else if op.eq("=") {
            return "=";
        } else if op.eq("+") {
            return "+";
        } else if op.eq("-") {
            return "-";
        } else if op.eq("*") {
            return "*";
        } else if op.eq(".") {
            return ".";
        } else if op.eq(",") {
            return ",";
        }

        unimplemented!()
    }
}
