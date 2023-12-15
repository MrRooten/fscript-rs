use std::fmt::Error;

use crate::frontend::ast::{parse::ASTParser, token::expr::FSRBinOp};

use super::base::FSRToken;

#[derive(Debug)]
pub struct FSRAssign<'a> {
    expr        : Box<FSRToken<'a>>,
    name        : &'a [u8]
}

#[derive(PartialEq)]
enum FSRAssignState {
    IdKeyLetStart,
    IdKeyLetEnd,
    NameStart,
    NameEnd,
    LeftValue,
    RightValue
}

impl<'a> FSRAssign<'a> {
    pub fn parse(source: &'a [u8]) -> Result<FSRAssign, Error> {
        let mut start = 0;
        let mut length = 0;
        let mut state = FSRAssignState::IdKeyLetStart;
        let mut name: Option<&[u8]> = None;
        let mut value: Option<Box<FSRToken>> = None;
        let mut len = 0;
        loop {
            let c = source[start + length];
            len += 1;
            if ASTParser::is_blank_char(c) && state == FSRAssignState::IdKeyLetEnd {
                start += 1;
                continue;
            }


            if ASTParser::is_blank_char(c) && state == FSRAssignState::IdKeyLetStart {
                state = FSRAssignState::IdKeyLetEnd;
                length = 0;
            }

            if ASTParser::is_name_letter(c) && state == FSRAssignState::IdKeyLetEnd {
                state = FSRAssignState::NameStart;
                length += 1;
                continue;
            }

            if ASTParser::is_name_letter(c) && state == FSRAssignState::NameStart {
                length += 1;
                continue;
            }

            if ASTParser::is_blank_char(c) && state == FSRAssignState::NameStart {
                state = FSRAssignState::NameEnd;
            }

            if c as char == '=' && (state == FSRAssignState::NameStart || state == FSRAssignState::NameEnd) {
                name = Some(&source[start..start+length]);
                state = FSRAssignState::RightValue;
                continue;
            }

            if state == FSRAssignState::RightValue {
                let expr = FSRBinOp::parse(&source[start..start+length]).unwrap();
                value = Some(expr);
                
                break;
            }

            start += 1;
        }

        unimplemented!()
    }

    pub fn parse_len(&self) -> usize {
        unimplemented!()
    }
}