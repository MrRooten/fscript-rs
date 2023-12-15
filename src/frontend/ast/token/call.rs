use std::fmt::Error;

use crate::frontend::ast::parse::ASTParser;
use std::str;
use super::base::FSRToken;

#[derive(Debug)]
pub struct FSRCall<'a> {
    name        : &'a str,
    args        : Vec<FSRToken<'a>>
}

#[derive(PartialEq)]
enum CallState {
    Name,
    Start,
    Args,
    WaitToken
}

impl FSRCall<'_> {
    pub fn parse(source: &[u8]) -> Result<Self, &str> {
        let mut state = CallState::Start;
        let mut start = 0;
        let mut length = 0;
        let mut name = "";
        loop {
            let i = source[start];
            let t_i = source[start + length];
            if state == CallState::Start && ASTParser::is_blank_char(i) {
                start += 1;
                continue;
            }

            if ASTParser::is_name_letter(i) && state == CallState::Start {
                state = CallState::Name;
                length += 1;
                continue;
            }

            if state == CallState::Name {
                length += 1;
                continue;
            }

            if state == CallState::Name && ASTParser::is_name_letter(t_i) == false {
                name = str::from_utf8(&source[start..start+length]).unwrap();
                state = CallState::WaitToken;
                start = start + length;
                continue;
            }

            
        }
        unimplemented!()
    }

    pub fn parse_len(&self) -> usize {
        unimplemented!()
    }
}