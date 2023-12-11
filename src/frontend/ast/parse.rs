use std::{fmt::Error};
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

    fn token_process(token: &ASTTokenEnum, source: &str) {}

    pub fn parse(source: &str) -> Result<ASTParser, Error> {
        
        unimplemented!()
    }
}
