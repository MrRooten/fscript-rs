
use crate::{
    frontend::ast::{parse::ASTParser, token::block::FSRBlock},
    utils::error::SyntaxError,
};

use super::base::FSRPosition;
use std::str;
#[derive(Debug, Clone)]
pub struct FSRClassFrontEnd<'a> {
    name: &'a str,
    block: FSRBlock<'a>,
    meta: FSRPosition,
}

impl<'a> FSRClassFrontEnd<'a> {
    pub fn get_name(&self) -> &'a str {
        self.name
    }

    pub fn get_block(&self) -> &FSRBlock<'a> {
        &self.block
    }

    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }

    pub fn parse(source: &'a [u8], meta: FSRPosition) -> Result<(Self, usize), SyntaxError> {
        let start_token = str::from_utf8(&source[0..5]).unwrap();
        if !start_token.eq("class") {
            unimplemented!()
        }

        let mut start = 5;
        if source[start] as char != ' ' {
            unimplemented!()
        }

        start += 1;
        let mut c = source[start];
        while ASTParser::is_blank_char(c) {
            start += 1;
            c = source[start];
        }
        let mut length = 0;
        if !ASTParser::is_name_letter_first(c) {
            unimplemented!()
        }

        length += 1;

        while ASTParser::is_name_letter(c) {
            c = source[start + length];
            length += 1;
        }
        length -= 1;
        let name = str::from_utf8(&source[start..start + length]).unwrap();
        start += length;

        while start < source.len() && ASTParser::is_blank_char(source[start]) {
            start += 1;
        }

        if source[start] as char != '{' {
            unimplemented!()
        }
        let sub_meta = meta.from_offset(start);
        let len = ASTParser::read_valid_bracket(&source[start..], sub_meta)?;
        let sub_meta = meta.from_offset(start);
        let block = FSRBlock::parse(&source[start..start + len], sub_meta)?;

        Ok((Self { name, block, meta }, start + len))
    }
}
