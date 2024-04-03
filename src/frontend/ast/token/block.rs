#![allow(unused)]

use super::base::{FSRMeta, FSRToken};
use super::for_statement::FSRFor;
use super::function_def::FSRFnDef;
use super::if_statement::FSRIf;
use super::return_def::FSRReturn;
use crate::frontend::ast::token::assign;
use crate::frontend::ast::token::assign::FSRAssign;
use crate::frontend::ast::utils::automaton::{FSTrie, NodeType};
use crate::frontend::ast::{parse::ASTParser, token::expr::FSRExpr};
use crate::utils::error::SyntaxError;
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
        return Self { states: vec![] };
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
        return self.peek().eq(state);
    }

    pub fn is_empty(&self) -> bool {
        return self.states.len() == 0;
    }
}

#[derive(Debug, Clone)]
pub struct FSRBlock<'a> {
    tokens: Vec<FSRToken<'a>>,
    len: usize,
    meta: FSRMeta
}

impl<'a> FSRBlock<'a> {
    pub fn get_meta(&self) -> &FSRMeta {
        return &self.meta;
    }

    pub fn get_tokens(&self) -> &Vec<FSRToken<'a>> {
        return &self.tokens;
    }

    pub fn get_len(&self) -> usize {
        return self.len;
    }

    pub fn parse(source: &'a [u8], meta: FSRMeta) -> Result<Self, SyntaxError> {
        let mut trie = FSTrie::new();
        let mut start = 0;
        let mut length = 0;
        let mut states = BlockStates::new();
        let mut is_start = true;
        states.push_state(BlockState::Start);
        let mut block = Self {
            tokens: vec![],
            len: 0,
            meta: meta.clone(),
        };
        loop {
            if start + length >= source.len() {
                break;
            }

            let mut c = source[start + length] as char;
            if (states.peek() == &BlockState::Start || states.peek() == &BlockState::Block)
                && ASTParser::is_blank_char(c as u8)
            {
                start += 1;
                continue;
            }

            if c == '}' && states.peek() == &BlockState::Start && is_start != true {
                break;
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
                start = start + length;
                length = 0;
                let mut sub_meta = meta.clone();
                sub_meta.offset = meta.offset + start;
                let l = ASTParser::read_valid_bracket(&source[start..], sub_meta)?;
                length += l;
                let s = String::from_utf8_lossy(&source[start..start + length]).to_string();
                let mut sub_block_meta = meta.clone();
                sub_block_meta.offset = meta.offset + start;
                let sub_block = Self::parse(&source[start..start + length], sub_block_meta)?;
                block.tokens.push(FSRToken::Block(sub_block));
                start += length;
                length = 0;
                continue;
            }

            if states.peek() == &BlockState::Block {
                while ASTParser::is_blank_char(c as u8) {
                    start += 1;
                    c = source[start+length] as char;
                    continue;
                }

                let t = match trie.match_token(&source[start..]) {
                    Some(s) => s,
                    None => {
                        let mut sub_meta = meta.clone();
                        sub_meta.offset = meta.offset + start;
                        let expr = FSRExpr::parse(&source[start..], false, sub_meta)?;
                        length += expr.1;
                        block.tokens.push(expr.0);
                        start = length + start;
                        length = 0;
                        continue;
                    }
                };
                
                if t == &NodeType::IfState {
                    let mut sub_meta = meta.clone();
                    sub_meta.offset = meta.offset + start;
                    let if_block = FSRIf::parse(&source[start..], sub_meta)?;
                    length += if_block.get_len();
                    block.tokens.push(FSRToken::IfExp(if_block));
                    start = length + start;
                    length = 0;
                } 
                else if t == &NodeType::ForState {
                    let mut sub_meta = meta.clone();
                    sub_meta.offset = meta.offset + start;
                    let for_block = FSRFor::parse(&source[start..], sub_meta)?;
                    length += for_block.get_len();
                    block.tokens.push(FSRToken::ForExp(for_block));
                    start = length + start;
                    length = 0;
                }
                else if t == &NodeType::FnState {
                    let mut sub_meta = meta.clone();
                    sub_meta.offset = meta.offset + start;
                    let fn_def = FSRFnDef::parse(&source[start..], sub_meta)?;
                    length += fn_def.get_len();
                    block.tokens.push(FSRToken::FunctionDef(fn_def));
                    start = start + length;
                    length = 0;
                }
                else if t == &NodeType::ReturnState {
                    let mut sub_meta = meta.clone();
                    sub_meta.offset = meta.offset + start;
                    let ret_expr = FSRReturn::parse(&source[start..], sub_meta)?;
                    length += ret_expr.1;
                    block.tokens.push(FSRToken::Return(ret_expr.0));
                    start = start + length;
                    length = 0;
                }
            }
        }
        block.len = start + length;
        return Ok(block);
    }
}
