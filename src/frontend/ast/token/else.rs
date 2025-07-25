use crate::{
    frontend::ast::{parse::ASTParser, token::if_statement::FSRIf},
    utils::error::SyntaxError,
};

use super::{
    base::{FSRPosition, FSRToken},
    block::FSRBlock, ASTContext,
};

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct ElseIf {
    test: Option<Box<FSRToken>>,
    body: Box<FSRBlock>,
}

impl ElseIf {
    pub fn get_test(&self) -> Option<&FSRToken> {
        match &self.test {
            Some(s) => Some(s),
            None => None
        }
    }

    pub fn get_block(&self) -> &FSRBlock {
        &self.body
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct FSRElse {
    else_ifs: Vec<ElseIf>,
    len: usize,
    meta: FSRPosition,
}

impl FSRElse {

    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }

    pub fn get_len(&self) -> usize {
        self.len
    }

    pub fn get_elses(&self) -> &Vec<ElseIf> {
        &self.else_ifs
    }


    pub fn parse(source: &[u8], meta: FSRPosition, context: &mut ASTContext) -> Result<FSRElse, SyntaxError> {
        let mut else_ifs = vec![];
        let mut s = std::str::from_utf8(&source[0..4]).unwrap();
        let mut start = 0;
        while s.eq("else") {
            start += 4;
            while source[start] as char == ' ' {
                start += 1;
            }

            let may_if_token = std::str::from_utf8(&source[start..start + 2]).unwrap();
            if may_if_token.eq("if") {
                let sub_meta = context.new_pos();
                let if_block = FSRIf::parse_without_else(&source[start..], sub_meta, context)?;
                start += if_block.get_len();
                let e = ElseIf {
                    test: Some(if_block.test),
                    body: if_block.body,
                };
                
                else_ifs.push(e);
            } else if source[start] as char == '{' {
                let sub_meta = context.new_pos();
                let b_len = ASTParser::read_valid_bracket(&source[start..], sub_meta, &context)?;
                let sub_meta = context.new_pos();
                let block = FSRBlock::parse(&source[start..start + b_len], sub_meta, context)?;
                let len = block.get_len();
                start += len;
                let e = ElseIf {
                    test: None,
                    body: Box::new(block),
                };
                else_ifs.push(e);
            }

            while start < source.len() && ASTParser::is_blank_char_with_new_line(source[start]) {
                start += 1;
            }


            if start + 4 >= source.len() {
                break;
            }
            s = std::str::from_utf8(&source[start..start + 4]).unwrap();
            
        }

        let sub_meta = context.new_pos();
        Ok(Self {
            else_ifs,
            len: start,
            meta: sub_meta,
        })
    }
}
