use crate::{
    frontend::ast::{parse::ASTParser, token::if_statement::FSRIf},
    utils::error::SyntaxError,
};

use super::{
    base::{FSRPosition, FSRToken},
    block::FSRBlock,
};

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct ElseIf<'a> {
    test: Option<Box<FSRToken<'a>>>,
    body: Box<FSRBlock<'a>>,
}

impl<'a> ElseIf<'a> {
    pub fn get_test(&self) -> Option<&FSRToken<'a>> {
        match &self.test {
            Some(s) => Some(s),
            None => None
        }
    }

    pub fn get_block(&self) -> &FSRBlock<'a> {
        &self.body
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct FSRElse<'a> {
    else_ifs: Vec<ElseIf<'a>>,
    len: usize,
    meta: FSRPosition,
}

impl<'a> FSRElse<'a> {

    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }

    pub fn get_len(&self) -> usize {
        self.len
    }

    pub fn get_elses(&self) -> &Vec<ElseIf<'a>> {
        &self.else_ifs
    }

    pub fn parse(source: &'a [u8], meta: FSRPosition) -> Result<FSRElse<'a>, SyntaxError> {
        let mut else_ifs = vec![];
        let mut s = unsafe { std::str::from_utf8_unchecked(&source[0..4]) };
        let mut start = 4;
        while s.eq("else") {
            while source[start] as char == ' ' {
                start += 1;
            }

            let may_if_token = unsafe { std::str::from_utf8_unchecked(&source[start..start + 2]) };
            if may_if_token.eq("if") {
                let sub_meta = meta.from_offset(start);
                let if_block = FSRIf::parse(&source[start..], sub_meta)?;
                start += if_block.get_len();
                let e = ElseIf {
                    test: Some(if_block.test),
                    body: if_block.body,
                };
                
                else_ifs.push(e);
            } else if source[start] as char == '{' {
                let sub_meta = meta.from_offset(start);
                let block = FSRBlock::parse(&source[start..], sub_meta)?;
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
            s = unsafe { std::str::from_utf8_unchecked(&source[start..start + 4]) };
            
        }

        let sub_meta = meta.from_offset(start);
        Ok(Self {
            else_ifs,
            len: start,
            meta: sub_meta,
        })
    }
}
