
use crate::{
    ast::{SyntaxError, parse::ASTParser, token::block::FSRBlock}, chrs2str
};

use super::{base::FSRPosition, ASTContext};
use std::str;
#[derive(Debug, Clone)]
pub struct FSRStructFrontEnd {
    name: String,
    block: FSRBlock,
    meta: FSRPosition,
}

impl FSRStructFrontEnd {
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_block(&self) -> &FSRBlock {
        &self.block
    }

    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }

    pub fn parse(source: &[char], meta: FSRPosition, context: &mut ASTContext) -> Result<(Self, usize), SyntaxError> {
        // let start_token = str::from_utf8(&source[0..6]).unwrap();
        let start_token = chrs2str!(&source[0..6]);
        if !start_token.eq("struct") {
            unimplemented!()
        }

        let mut start = 6;
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
        // let name = str::from_utf8(&source[start..start + length]).unwrap();
        let name = chrs2str!(&source[start..start + length]);
        start += length;

        while start < source.len() && ASTParser::is_blank_char(source[start]) {
            start += 1;
        }

        if source[start] as char != '{' {
            unimplemented!()
        }
        let sub_meta = meta.new_offset(start);
        let len = ASTParser::read_valid_bracket(&source[start..], sub_meta, context)?;
        let sub_meta = meta.new_offset(start);
        let block = FSRBlock::parse(&source[start..start + len], sub_meta, context, Some(name.to_string()))?;
        for stmt in block.get_tokens() {
            if !stmt.is_variable() && !stmt.is_function() {
                let offset = stmt.get_meta();
                return Err(SyntaxError::new(&offset, "only variable definitions are allowed in struct block"));
            }
        }
        context.add_variable(&name, None);
        Ok((Self { name: name.to_string(), block, meta }, start + len))
    }
}
