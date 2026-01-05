
use crate::{
    frontend::ast::{parse::ASTParser, token::{base::FSRToken, block::FSRBlock, expr::FSRExpr, for_statement::FSRFor, function_def::FSRFnDef, if_statement::FSRIf, import::FSRImport, return_def::FSRReturn, try_expr::FSRTryBlock, while_statement::FSRWhile}, utils::automaton::NodeType},
    utils::error::SyntaxError,
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

    pub fn parse(source: &[u8], meta: FSRPosition, context: &mut ASTContext) -> Result<(Self, usize), SyntaxError> {
        let start_token = str::from_utf8(&source[0..6]).unwrap();
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
        let name = str::from_utf8(&source[start..start + length]).unwrap();
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
        context.add_variable(name, None);
        Ok((Self { name: name.to_string(), block, meta }, start + len))
    }
}

#[derive(PartialEq)]
enum BlockState {
    Start,
    Block,
}

struct BlockStates {
    states: Vec<BlockState>,
}

impl BlockStates {
    pub fn new() -> Self {
        Self { states: vec![] }
    }

    pub fn set_up_state(&mut self, new_state: BlockState) {
        self.states.pop();
        self.states.push(new_state);
    }

    pub fn push_state(&mut self, state: BlockState) {
        self.states.push(state);
    }

    pub fn pop_state(&mut self) {
        self.states.pop();
    }

    pub fn peek(&self) -> &BlockState {
        &self.states[self.states.len() - 1]
    }

    pub fn eq_peek(&self, state: &BlockState) -> bool {
        self.peek().eq(state)
    }

    pub fn is_empty(&self) -> bool {
        self.states.len() == 0
    }
}
