
use crate::{
    frontend::ast::{parse::ASTParser, token::block::FSRBlock},
    utils::error::SyntaxError,
};

use super::{base::FSRPosition, ASTContext};
use std::str;
#[derive(Debug, Clone)]
pub struct FSRClassFrontEnd {
    name: String,
    block: FSRBlock,
    meta: FSRPosition,
}

impl FSRClassFrontEnd {
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_block(&self) -> &FSRBlock {
        &self.block
    }

    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }

    pub fn parse(source: &[u8], meta: FSRPosition, context: &mut ASTContext) -> Result<(Self, usize), SyntaxError> {
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
        let sub_meta = context.new_pos();
        let len = ASTParser::read_valid_bracket(&source[start..], sub_meta, &context)?;
        let sub_meta = context.new_pos();
        let block = FSRBlock::parse(&source[start..start + len], sub_meta, context)?;
        context.add_variable(name, None);
        Ok((Self { name: name.to_string(), block, meta }, start + len))
    }
}
