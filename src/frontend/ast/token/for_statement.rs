use crate::{
    frontend::ast::{parse::ASTParser, token::expr::FSRExpr},
    utils::error::SyntaxError,
};

use super::{
    base::{FSRPosition, FSRToken},
    block::FSRBlock,
    ASTContext,
};

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct FSRFor {
    var_name: String,
    expr: Box<FSRToken>,
    body: Box<FSRBlock>,
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

#[derive(PartialEq, Clone)]
enum Bracket {
    Round,
    Square,
    Curly,
}

impl FSRFor {
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

    pub fn get_block(&self) -> &FSRBlock {
        &self.body
    }

    pub fn parse(
        source: &[u8],
        meta: FSRPosition,
        context: &mut ASTContext,
    ) -> Result<Self, SyntaxError> {
        let s = std::str::from_utf8(&source[0..3]).unwrap();

        if s != "for" {
            let mut sub_meta = meta.new_offset(0);
            let err = SyntaxError::new(&sub_meta, "not for token");
            return Err(err);
        }

        if !ASTParser::is_blank_char(source[3]) {
            let mut sub_meta = meta.new_offset(3);
            let err = SyntaxError::new(&sub_meta, "blank space after for token");
            return Err(err);
        }

        let mut start = 3;
        while start < source.len() && ASTParser::is_blank_char(source[start]) {
            start += 1;
        }

        let mut name = String::new();
        if !ASTParser::is_name_letter_first(source[start]) {
            let mut sub_meta = meta.new_offset(start);
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
            let mut sub_meta = meta.new_offset(start);
            let err = SyntaxError::new(&sub_meta, "blank space after for token");
            return Err(err);
        }

        start += 1;
        while start < source.len() && ASTParser::is_blank_char(source[start]) {
            start += 1;
        }

        if !ASTParser::is_blank_char(source[start + 2]) {
            let mut sub_meta = meta.new_offset(start);
            let err = SyntaxError::new(&sub_meta, "in after variable in for statement");
            return Err(err);
        }

        let s = std::str::from_utf8(&source[start..start + 2]).unwrap();
        if !s.eq("in") {
            let mut sub_meta = meta.new_offset(start);
            let err = SyntaxError::new(&sub_meta, "in after variable in for statement");
            return Err(err);
        }

        start += 2;

        if !ASTParser::is_blank_char(source[start]) {
            let mut sub_meta = meta.new_offset(start);
            let err = SyntaxError::new(&sub_meta, "blank space after in token");
            return Err(err);
        }

        start += 1;
        let mut len = 0;
        let mut state = State::Continue;
        let mut pre_state = State::Continue;
        let mut brackets = vec![];
        for c in &source[start..] {
            let c = *c as char;
            len += 1;
            if c == '{'
                && (state != State::DoubleQuote && state != State::SingleQuote)
                && brackets.is_empty()
            {
                len -= 1;
                break;
            }

            if c == '(' && (state != State::DoubleQuote && state != State::SingleQuote) {
                brackets.push(Bracket::Round);
                continue;
            }

            if c == ')'
                && (state != State::DoubleQuote && state != State::SingleQuote)
                && !brackets.is_empty()
            {
                if brackets.last().unwrap() != &Bracket::Round {
                    let mut sub_meta = meta.new_offset(start);
                    let err = SyntaxError::new(&sub_meta, "Invalid for statement");
                    return Err(err);
                }
                if brackets.is_empty() {
                    let mut sub_meta = meta.new_offset(start);
                    let err = SyntaxError::new(&sub_meta, "Invalid for statement");
                    return Err(err);
                }
                brackets.pop();
                continue;
            }

            if c == '\n'
                && (state != State::DoubleQuote && state != State::SingleQuote)
                && brackets.is_empty()
            {
                let mut sub_meta = meta.new_offset(start);
                let err = SyntaxError::new(&sub_meta, "Invalid If statement");
                return Err(err);
            }

            if c == '{' && (state != State::DoubleQuote && state != State::SingleQuote) {
                brackets.push(Bracket::Curly);
                continue;
            }
            if c == '}'
                && (state != State::DoubleQuote && state != State::SingleQuote)
                && !brackets.is_empty()
            {
                if brackets.last().unwrap() != &Bracket::Curly {
                    let mut sub_meta = meta.new_offset(start);
                    let err = SyntaxError::new(&sub_meta, "Invalid for statement");
                    return Err(err);
                }
                if brackets.is_empty() {
                    let mut sub_meta = meta.new_offset(start);
                    let err = SyntaxError::new(&sub_meta, "Invalid for statement");
                    return Err(err);
                }
                brackets.pop();
                continue;
            }
            if c == '[' && (state != State::DoubleQuote && state != State::SingleQuote) {
                brackets.push(Bracket::Square);
                continue;
            }
            if c == ']'
                && (state != State::DoubleQuote && state != State::SingleQuote)
                && !brackets.is_empty()
            {
                if brackets.last().unwrap() != &Bracket::Square {
                    let mut sub_meta = meta.new_offset(start);
                    let err = SyntaxError::new(&sub_meta, "Invalid for statement");
                    return Err(err);
                }
                if brackets.is_empty() {
                    let mut sub_meta = meta.new_offset(start);
                    let err = SyntaxError::new(&sub_meta, "Invalid for statement");
                    return Err(err);
                }
                brackets.pop();
                continue;
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
        // println!("expr: {}", String::from_utf8_lossy(expr));
        let sub_meta = meta.new_offset(start);
        let expr = FSRExpr::parse(expr, false, sub_meta, context)?.0;
        start += len;
        let sub_meta = meta.new_offset(start);
        let b_len = ASTParser::read_valid_bracket(&source[start..], sub_meta, &context)?;
        let mut sub_meta = meta.new_offset(start);
        let body = FSRBlock::parse(&source[start..start + b_len], sub_meta, context)?;
        start += body.get_len();
        context.add_variable(&name, None);
        Ok(Self {
            var_name: name,
            expr: Box::new(expr),
            body: Box::new(body),
            len: start,
            meta,
        })
    }
}

mod test {
    use super::*;

    use crate::utils::error::SyntaxError;

    #[test]
    fn test_for() {
        let expr = "for i in [1, 2, 3].map(|| {}) { println(i) }";
        let meta = FSRPosition::new();
        let mut context = ASTContext::new_context();
        let token = FSRFor::parse(expr.as_bytes(), meta, &mut context).unwrap();
        println!("{:#?}", token);
    }
}
