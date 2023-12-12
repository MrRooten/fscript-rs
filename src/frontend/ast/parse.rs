use std::{fmt::Error, ops::Range};
use crate::frontend::ast::token::statement::ASTState;

use super::token::statement::{ASTTokenEnum, ASTToken};



pub struct ASTParser {
    tokens      : Vec<ASTToken>
}

fn is_token_letter(c: char) -> bool {
    unimplemented!()
}

type FnExpectTokens = fn() -> Vec<ASTTokenEnum>;

impl ASTParser {
    fn get_max_token_length() -> usize {
        unimplemented!()
    }

    fn match_token(token: &str) -> (Option<ASTTokenEnum>, bool) {
        unimplemented!()
    }

    fn get_fn_expect_token(token: &ASTTokenEnum) -> FnExpectTokens {
        unimplemented!()
    }

    fn is_black_char(c: char) -> bool {
        return c == ' ' || c == '\n' || c == '\t';
    }

    fn token_process(token: &ASTTokenEnum, source: &str) {}


    pub fn parse(source: &str) -> Result<ASTParser, Error> {
        let mut cur_range = Range {
            start: 0,
            end: 0,
        };

        let mut state = ASTState::WaitToken;
        for (i, c) in source.chars().enumerate() {
            cur_range.end = i;
            if state == ASTState::WaitToken && ASTParser::is_black_char(c)   {
                continue;
            }

            cur_range.start = i;
            
        }
        unimplemented!()
    }
}
