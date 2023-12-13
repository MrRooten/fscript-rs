use std::{fmt::Error, ops::Range, ascii::AsciiExt};
use crate::frontend::ast::token::{statement::ASTState, base::FSRTokenState, if_statement::FSRIfState};

use super::token::statement::{ASTTokenEnum, ASTToken};



pub struct ASTParser {
    tokens      : Vec<ASTToken>
}

fn is_token_letter(c: char) -> bool {
    unimplemented!()
}

type FnExpectTokens = fn() -> Vec<ASTTokenEnum>;

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

    pub fn is_blank_char(c: u8) -> bool {
        return c as char == ' ' || c as char == '\n' || c as char == '\t';
    }

    pub fn is_name_letter(c: u8) -> bool {
        unimplemented!()
    }

    pub fn end_token_char(c: u8) -> bool {
        unimplemented!()
    }

    fn token_process(token: &ASTTokenEnum, source: &str) {}



    pub fn parse(source: &str) -> Result<ASTParser, Error> {
        
        unimplemented!()
    }
}
