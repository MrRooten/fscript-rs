use std::{collections::HashMap, fmt::Error};

use crate::{frontend::ast::{token::{base::FSRTokenState, statement::ASTState, if_statement::FSRIf, assign::FSRAssign, call::FSRCall}, parse::ASTParser}, backend::base_type::function};

use super::{base::FSRToken, hashtable::FSRHashtable};

#[derive(Debug)]
pub struct FSRFunctionDef<'a> {
    name        : &'a str,
    args        : Vec<&'a str>,
    body        : Vec<FSRToken<'a>>,
    defaults    : Vec<FSRToken<'a>>,
    len         : usize
}

impl FSRFunctionDef<'_> {
    pub fn parse_len(&self) -> usize {
        return self.len;
    }

    pub fn parse(source: &[u8]) -> Result<Self, Error> {
        let mut cur_start = 0;
        let mut cur_length = 0;
        let mut process_stack: Vec<FSRTokenState> = Vec::new();
        let mut state = ASTState::WaitToken;
        let mut body = vec![];
        let mut i = 0;
        while i < source.len() {
            let c = source[i];
            if state == ASTState::WaitToken && ASTParser::is_blank_char(c)   {
                continue;
            }

            
            state = ASTState::StartToken;

            if ASTParser::end_token_char(c) {
                
                let token_s = &source[cur_start..cur_start+cur_length];

                if token_s.eq("if".as_bytes()) {
                    let if_token = FSRIf::parse(&source[cur_start..]).unwrap();
                    i += if_token.parse_len();
                    body.push(FSRToken::IfExp(if_token));
                    
                }

                if token_s.eq("fn".as_bytes()) {
                    let function = FSRFunctionDef::parse(&source[cur_start..]).unwrap();
                    i += function.parse_len();
                    body.push(FSRToken::FunctionDef(function));
                }

                if token_s.eq("let".as_bytes()) {
                    let assign = FSRAssign::parse(&source[cur_start..]).unwrap();
                    i += assign.parse_len();
                    body.push(FSRToken::Assign(assign));
                }

                if token_s.eq("for".as_bytes()) {

                }

                if token_s.eq("while".as_bytes()) {

                }

                if c as char == '(' {
                    let call = FSRCall::parse(&source[cur_start..]).unwrap();
                    i += call.parse_len();
                    body.push(FSRToken::Call(call));
                }
            }
            
        }
        unimplemented!()
    }
}