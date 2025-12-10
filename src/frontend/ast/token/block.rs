#![allow(unused)]

use super::base::{FSRPosition, FSRToken};
use super::for_statement::FSRFor;
use super::function_def::FSRFnDef;
use super::if_statement::FSRIf;
use super::import::FSRImport;
use super::r#else::FSRElse;
use super::return_def::FSRReturn;
use super::try_expr::FSRTryBlock;
use super::while_statement::FSRWhile;
use super::ASTContext;
use crate::frontend::ast::token::assign;
use crate::frontend::ast::token::assign::FSRAssign;
use crate::frontend::ast::utils::automaton::{FSTrie, NodeType};
use crate::frontend::ast::{parse::ASTParser, token::expr::FSRExpr};
use crate::utils::error::SyntaxError;
use std::rc::Rc;
use std::str;

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

#[derive(Debug, Clone)]
pub struct FSRBlock {
    tokens: Vec<FSRToken>,
    len: usize,
    meta: FSRPosition,
}

impl FSRBlock {
    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }

    pub fn get_tokens(&self) -> &Vec<FSRToken> {
        &self.tokens
    }

    pub fn get_len(&self) -> usize {
        self.len
    }

    pub fn parse(
        source: &[u8],
        meta: FSRPosition,
        context: &mut ASTContext,
    ) -> Result<Self, SyntaxError> {
        let mut trie = FSTrie::new();
        let mut start = 0;
        let mut length = 0;
        let mut states = BlockStates::new();
        let mut is_start = true;
        states.push_state(BlockState::Start);
        let mut block = Self {
            tokens: vec![],
            len: 0,
            meta: meta.new_offset(0),
        };
        loop {
            if start + length >= source.len() {
                break;
            }

            let mut c = source[start + length] as char;

            if c == '#' {
                while start + length < source.len() && source[start + length] as char != '\n' {
                    start += 1;
                }

                continue;
            }
            if (states.peek() == &BlockState::Start || states.peek() == &BlockState::Block)
                && ASTParser::is_blank_char_with_new_line(c as u8)
            {
                start += 1;
                continue;
            }

            if c == '}' && states.peek() == &BlockState::Start && !is_start {
                return Err(SyntaxError::new_with_type(
                    &meta.new_offset(start + length),
                    "error",
                    crate::utils::error::SyntaxErrType::None,
                ));
            }

            if c == '{' && states.peek() == &BlockState::Start {
                states.push_state(BlockState::Block);
                start += 1;
                continue;
            }

            is_start = false;

            if c == '}' && states.peek() == &BlockState::Block {
                states.pop_state();
                start += 1;
                continue;
            }

            if c == '{' && states.peek() == &BlockState::Block {
                start += length;
                length = 0;

                let mut sub_meta = meta.new_offset(start);
                let l = ASTParser::read_valid_bracket(&source[start..], sub_meta, context)?;
                length += l;
                let s = String::from_utf8_lossy(&source[start..start + length]).to_string();
                let mut sub_block_meta = meta.new_offset(start);
                let sub_block =
                    Self::parse(&source[start..start + length], sub_block_meta, context)?;
                block.tokens.push(FSRToken::Block(sub_block));

                start += length;
                length = 0;

                continue;
            }

            if states.peek() == &BlockState::Block {
                // Escape all blank characters
                while ASTParser::is_blank_char_with_new_line(c as u8) {
                    start += 1;
                    c = source[start + length] as char;
                    continue;
                }

                let t = match trie.match_token(&source[start..]) {
                    Some(s) => s,
                    None => {
                        let mut sub_meta = meta.new_offset(start);
                        let expr = FSRExpr::parse(&source[start..], false, sub_meta, context)?;
                        length += expr.1;
                        block.tokens.push(expr.0);
                        start += length;
                        length = 0;
                        continue;
                    }
                };

                if t == &NodeType::IfState {
                    let mut sub_meta = meta.new_offset(start);
                    let if_block = FSRIf::parse(&source[start..], sub_meta, context)?;
                    length += if_block.get_len();
                    block.tokens.push(FSRToken::IfExp(if_block));
                    start += length;
                    length = 0;
                } else if t == &NodeType::WhileState {
                    let mut sub_meta = meta.new_offset(start);
                    let while_block = FSRWhile::parse(&source[start..], sub_meta, context)?;
                    length += while_block.get_len();
                    block.tokens.push(FSRToken::WhileExp(while_block));
                    start += length;
                    length = 0;
                } else if t == &NodeType::FnState {
                    let mut sub_meta = meta.new_offset(start);
                    let fn_def = FSRFnDef::parse(&source[start..], sub_meta, context)?;
                    length += fn_def.get_len();
                    block.tokens.push(FSRToken::FunctionDef(fn_def));
                    start += length;
                    length = 0;
                } else if t == &NodeType::ReturnState {
                    let mut sub_meta = meta.new_offset(start);
                    let ret_expr = FSRReturn::parse(&source[start..], sub_meta, context)?;
                    length += ret_expr.1;
                    block.tokens.push(FSRToken::Return(ret_expr.0));
                    start += length;
                    length = 0;
                } else if t == &NodeType::Else {
                    // not support else without if
                    return Err(SyntaxError::new(&meta.new_offset(start), "else without if"));
                    // let mut sub_meta = meta.new_offset(start);
                    // let else_expr = FSRElse::parse(&source[start..], sub_meta, context)?;
                } else if t == &NodeType::Break {
                    let mut sub_meta = meta.new_offset(start);
                    block.tokens.push(FSRToken::Break(sub_meta));
                    start += "break".len();
                } else if t == &NodeType::Continue {
                    let mut sub_meta = meta.new_offset(start);
                    block.tokens.push(FSRToken::Continue(sub_meta));
                    start += "continue".len();
                } else if t == &NodeType::ForState {
                    let mut sub_meta = meta.new_offset(start);

                    let for_def = FSRFor::parse(&source[start..], sub_meta, context)?;
                    length += for_def.get_len();
                    block.tokens.push(FSRToken::ForBlock(for_def));
                    start += length;
                    length = 0;
                } else if t == &NodeType::Try {
                    let mut sub_meta = meta.new_offset(start);

                    let try_def = FSRTryBlock::parse(&source[start..], sub_meta, context)?;
                    length += try_def.get_len();
                    block.tokens.push(FSRToken::TryBlock(try_def));
                    start += length;
                    length = 0;
                } else if t == &NodeType::Import {
                    let mut sub_meta = meta.new_offset(start);

                    let import_def = FSRImport::parse(&source[start..], sub_meta, context)?;
                    length += import_def.1;
                    block.tokens.push(FSRToken::Import(import_def.0));
                    start += length;
                    length = 0;
                } else if t == &NodeType::Root {
                    let mut sub_meta = meta.new_offset(start);
                    let expr = FSRExpr::parse(&source[start..], false, sub_meta, context)?;
                    length += expr.1;
                    block.tokens.push(expr.0);
                    start += length;
                    length = 0;
                } else {
                    let sub_meta = meta.new_offset(start);
                    let err = SyntaxError::new(&sub_meta, "invalid token in block");
                    return Err(err);
                }
            }
        }
        block.len = start + length;
        Ok(block)
    }
}

mod test {
    use crate::frontend::ast::token::{base::FSRPosition, block::FSRBlock, ASTContext};

    #[test]
    fn test_block_comment() {
        let s = "{
        
        } #absdfsdf
        ";
        let meta = FSRPosition::new();
        let mut context = ASTContext::new_context();
        let b = FSRBlock::parse(s.as_bytes(), meta, &mut context).unwrap();
        println!("{:#?}", b);
    }

    #[test]
    fn test_class_getter() {
        let s = "{
            Abc::ddc::get_abc()
        ";
        let meta = FSRPosition::new();
        let mut context = ASTContext::new_context();
        let b = FSRBlock::parse(s.as_bytes(), meta, &mut context).unwrap();
        println!("{:#?}", b);
    }
}
