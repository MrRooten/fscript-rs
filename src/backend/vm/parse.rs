use std::{fmt::Error};

use crate::backend::vm::token::state::ASTState;

use super::token::state::{ASTTokenEnum, ASTToken};

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

    fn token_process(token: &ASTTokenEnum, source: &str) {}

    pub fn parse(source: &str) -> Result<ASTParser, Error> {
        let mut state = ASTTokenEnum::Start;
        let mut start: usize = 0;
        let mut cur_state = ASTState::ContinueToken;
        let mut token_stack: Vec<ASTTokenEnum> = vec![];
        for (i, c) in source.chars().enumerate() {
            if c == ' ' && cur_state == ASTState::ContinueToken {
                let token = &source[start..i];
                let token = ASTParser::match_token(token);
                if let Some(t) = token.0 {
                    let func = ASTParser::get_fn_expect_token(&t);
                    token_stack.push(t);
                    start = i;
                    cur_state = ASTState::TokenEnd;
                } else {
                    if token.1 == false {
                        // not match token
                        unimplemented!()
                    }
                }
                continue;
            }

            if c == ' ' && cur_state == ASTState::TokenEnd {
                start = i;

                continue;
            }

            if is_token_letter(c) && cur_state == ASTState::TokenEnd {
                cur_state = ASTState::ContinueToken;
            }
        }
        unimplemented!()
    }
}
