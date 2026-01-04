use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    frontend::ast::{
        parse::ASTParser, token::{ASTVariableState, xtruct::FSRStructFrontEnd}, utils::automaton::{FSTrie, NodeType}
    },
    utils::error::SyntaxError,
};

use super::{
    base::{FSRPosition, FSRToken},
    block::FSRBlock,
    class::FSRClassFrontEnd,
    expr::FSRExpr,
    for_statement::FSRFor,
    function_def::FSRFnDef,
    if_statement::FSRIf,
    import::FSRImport,
    return_def::FSRReturn,
    try_expr::FSRTryBlock,
    while_statement::FSRWhile,
    ASTContext,
};

#[derive(PartialEq)]
enum ModuleState {
    Start,
    Block,
}

struct ModuleStates {
    states: Vec<ModuleState>,
}

impl ModuleStates {
    pub fn new() -> Self {
        Self { states: vec![] }
    }

    #[allow(unused)]
    pub fn set_up_state(&mut self, new_state: ModuleState) {
        self.states.pop();
        self.states.push(new_state);
    }

    pub fn push_state(&mut self, state: ModuleState) {
        self.states.push(state);
    }

    pub fn pop_state(&mut self) {
        self.states.pop();
    }

    pub fn peek(&self) -> &ModuleState {
        &self.states[self.states.len() - 1]
    }

    #[allow(unused)]
    pub fn eq_peek(&self, state: &ModuleState) -> bool {
        self.peek().eq(state)
    }

    #[allow(unused)]
    pub fn is_empty(&self) -> bool {
        self.states.len() == 0
    }
}



#[derive(Debug, Clone)]
pub struct FSRModuleFrontEnd {
    pub(crate) tokens: Vec<FSRToken>,
    pub(crate) lambda_define_lines: Vec<usize>,
    pub(crate) ref_map: Rc<RefCell<HashMap<String, ASTVariableState>>>,
    len: usize,
    meta: FSRPosition,
}

impl FSRModuleFrontEnd {
    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }

    pub fn get_len(&self) -> usize {
        self.len
    }

    pub fn parse(
        source: &[u8],
        meta: FSRPosition,
    ) -> Result<(FSRModuleFrontEnd, Vec<usize>), SyntaxError> {
        let mut trie = FSTrie::new();
        let mut start = 0;
        let mut length = 0;
        let mut states = ModuleStates::new();
        states.push_state(ModuleState::Start);
        let mut module = Self {
            tokens: vec![],
            len: 0,
            meta: meta.clone(),
            lambda_define_lines: vec![],
            ref_map: Rc::new(RefCell::new(HashMap::new())),
        };

        let mut context = ASTContext::new_context();
        context.push_scope();
        loop {
            if start + length >= source.len() {
                break;
            }

            let mut c = source[start + length] as char;
            if (states.peek() == &ModuleState::Start || states.peek() == &ModuleState::Block)
                && ASTParser::is_blank_char_with_new_line(c as u8)
            {
                start += 1;
                continue;
            }

            if c == '}' && states.peek() == &ModuleState::Block {
                states.pop_state();
                start += 1;
                continue;
            }

            if c == '{' {
                start += length;
                length = 0;
                let mut sub_meta = meta.new_offset(start);
                let l = ASTParser::read_valid_bracket(&source[start..], sub_meta, &context)?;
                length += l;
                let mut sub_block_meta = meta.new_offset(start);
                let sub_block = FSRBlock::parse(&source[start..start + length], sub_block_meta, &mut context)?;
                module.tokens.push(FSRToken::Block(sub_block));
                start += length;
                length = 0;
                continue;
            }

            while ASTParser::is_blank_char_with_new_line(c as u8) {
                start += 1;
                c = source[start + length] as char;
                continue;
            }

            let t = match trie.match_token(&source[start..]) {
                Some(s) => s,
                None => {
                    let sub_meta = meta.new_offset(start);
                    let expr = FSRExpr::parse(&source[start..], false, sub_meta, &mut context)?;
                    length += expr.1;
                    module.tokens.push(expr.0);
                    start += length;
                    length = 0;
                    continue;
                }
            };

            if t == &NodeType::Root {
                let mut sub_meta = meta.new_offset(start);
                let expr = FSRExpr::parse(&source[start..], false, sub_meta, &mut context)?;
                length += expr.1;
                module.tokens.push(expr.0);
                start += length;
                length = 0;
                continue;
            } else if t == &NodeType::IfState {
                let mut sub_meta = meta.new_offset(start);
                let if_block = FSRIf::parse(&source[start..], sub_meta, &mut context)?;
                length += if_block.get_len();
                module.tokens.push(FSRToken::IfExp(if_block));
                start += length;
                length = 0;
            } else if t == &NodeType::WhileState {
                let mut sub_meta = meta.new_offset(start);
                let for_block = FSRWhile::parse(&source[start..], sub_meta, &mut context)?;
                length += for_block.get_len();
                module.tokens.push(FSRToken::WhileExp(for_block));
                start += length;
                length = 0;
            } else if t == &NodeType::FnState {
                let mut sub_meta = meta.new_offset(start);
                let fn_def = FSRFnDef::parse(&source[start..], sub_meta, &mut context)?;
                length += fn_def.get_len();
                module.tokens.push(FSRToken::FunctionDef(fn_def));
                start += length;
                length = 0;
            } else if t == &NodeType::ReturnState {
                let mut sub_meta = meta.new_offset(start);
                let ret_expr = FSRReturn::parse(&source[start..], sub_meta, &mut context)?;
                length += ret_expr.1;
                module.tokens.push(FSRToken::Return(ret_expr.0));
                start += length;
                length = 0;
            } else if t == &NodeType::ImportState {
                let mut sub_meta = meta.new_offset(start);
                let import_statement = FSRImport::parse(&source[start..], sub_meta, &mut context)?;
                length += import_statement.1;
                module
                    .tokens
                    .push(FSRToken::Import(import_statement.0.to_owned()));
                start += length;
                length = 0;
            } else if t == &NodeType::ClassState {
                let mut sub_meta = meta.new_offset(start);
                let class_def = FSRClassFrontEnd::parse(&source[start..], sub_meta, &mut context)?;
                length += class_def.1;
                module.tokens.push(FSRToken::Class(class_def.0.to_owned()));
                start += length;
                length = 0;
            } else if t == &NodeType::ForState {
                let mut sub_meta = meta.new_offset(start);
                let for_def = FSRFor::parse(&source[start..], sub_meta, &mut context)?;
                length += for_def.get_len();
                module.tokens.push(FSRToken::ForBlock(for_def));
                start += length;
                length = 0;
            } else if t == &NodeType::Import {
                let mut sub_meta = meta.new_offset(start);
                let import_def = FSRImport::parse(&source[start..], sub_meta, &mut context)?;
                length += import_def.1;
                module.tokens.push(FSRToken::Import(import_def.0));
                start += length;
                length = 0;
            } else if t == &NodeType::Try {
                let mut sub_meta = meta.new_offset(start);
                let try_def = FSRTryBlock::parse(&source[start..], sub_meta, &mut context)?;
                length += try_def.get_len();
                module.tokens.push(FSRToken::TryBlock(try_def));
                start += length;
                length = 0;
            } else if t == &NodeType::Struct {
                let mut sub_meta = meta.new_offset(start);
                let struct_def = FSRStructFrontEnd::parse(&source[start..], sub_meta, &mut context)?;
                length += struct_def.1;
                module.tokens.push(FSRToken::Struct(struct_def.0));
                start += length;
                length = 0;
            } else {
                let sub_meta = meta.new_offset(start);
                let err = SyntaxError::new(&sub_meta, "invalid token in module");
                return Err(err);
            }
        }
        let scope = context.pop_scope();
        module.ref_map = scope;
        module.len = start + length;

        let mut lines: Vec<usize> = source
            .iter()
            .enumerate()
            .filter_map(|(i, &c)| if c == b'\n' { Some(i) } else { None })
            .collect();
        Ok((module, lines))
    }
}
