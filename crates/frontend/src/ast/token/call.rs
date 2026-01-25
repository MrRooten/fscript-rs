use super::{
    base::{FSRPosition, FSRToken},
    expr::{FSRExpr, SingleOp},
    ASTContext,
};
use crate::{ast::{SyntaxError, parse::ASTParser}, chrs2str};
use std::str;

#[derive(Debug, Clone)]
pub struct FSRCall {
    name: String,
    args: Vec<FSRToken>,
    pub len: usize,
    pub single_op: Option<SingleOp>,
    meta: FSRPosition,
    pub is_defined: bool,
}

#[derive(PartialEq)]
enum CallState {
    Name,
    Start,
    _Args,
    _WaitToken,
}

impl FSRCall {
    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }

    pub fn get_args(&self) -> &Vec<FSRToken> {
        &self.args
    }

    pub fn get_args_mut(&mut self) -> &mut Vec<FSRToken> {
        &mut self.args
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn parse(
        source: &[char],
        meta: FSRPosition,
        context: &mut ASTContext,
        pre_args: bool,
    ) -> Result<Self, SyntaxError> {
        let mut state = CallState::Start;
        let mut start = 0;
        let mut length = 0;
        let mut name = "".to_string();
        
        if '(' == source[start] {
            name = "".to_string();
        } else {
            loop {
                let i = source[start];
                let t_i = source[start + length];

                if state == CallState::Start && ASTParser::is_blank_char_with_new_line(i) {
                    start += 1;
                    continue;
                }

                if ASTParser::is_name_letter(i) && state == CallState::Start {
                    state = CallState::Name;
                    continue;
                }

                if state == CallState::Name && ASTParser::is_name_letter(t_i) {
                    length += 1;
                    continue;
                }

                if state == CallState::Name && !ASTParser::is_name_letter(t_i) {
                    // name = str::from_utf8(&source[start..start + length]).unwrap();
                    name = chrs2str!(&source[start..start + length]);
                    let mut blank_length = 0;
                    while ASTParser::is_blank_char(source[start + length + blank_length]) {
                        blank_length += 1;
                    }

                    if state == CallState::Name && source[blank_length + start + length] as char == '(' {
                        // name = str::from_utf8(&source[start..start + length]).unwrap();
                        name = chrs2str!(&source[start..start + length]);
                        start += length + blank_length;
                        break;
                    }
                }

                panic!("Invalid function call syntax");
            }
        }

        let end_blasket =
            ASTParser::read_valid_bracket(&source[start..], meta.new_offset(start), context)?;

        //let s = str::from_utf8(source).unwrap();
        //let first = s.find('(').unwrap();
        //let last = s.rfind(')').unwrap();
        let args = &source[start + 1..end_blasket + start - 1];
        //let tmp = std::str::from_utf8(args).unwrap();
        let sub_meta = meta.new_offset(start);
        //let exprs = ASTParser::split_by_comma(args, sub_meta)?;
        let (expr, expr_len) = FSRExpr::parse(args, true, sub_meta, context).unwrap();
        let expr: Box<FSRToken> = Box::new(expr);
        let expr = Box::leak(expr);
        let fn_args = expr.flatten_comma();
        let fn_args = if fn_args.len() == 1 && fn_args[0].is_empty() {
            vec![]
        } else {
            fn_args
        };

        Ok(Self {
            name: name.to_string(),
            args: fn_args,
            len: start + expr_len + 2,
            single_op: None,
            meta,
            is_defined: false,
        })
    }

    pub fn get_len(&self) -> usize {
        self.len
    }
}

