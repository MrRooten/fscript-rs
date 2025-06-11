
use super::{
    base::{FSRPosition, FSRToken},
    expr::{FSRExpr, SingleOp}, ASTContext,
};
use crate::{frontend::ast::parse::ASTParser, utils::error::SyntaxError};
use std::str;

#[derive(Debug, Clone)]
pub struct FSRCall {
    name: String,
    args: Vec<FSRToken>,
    pub(crate) len: usize,
    pub(crate) single_op: Option<SingleOp>,
    meta: FSRPosition,
    pub(crate) is_defined: bool,
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

    pub fn parse(source: &[u8], meta: FSRPosition, context: &mut ASTContext, pre_args: bool) -> Result<Self, SyntaxError> {
        let mut state = CallState::Start;
        let mut start = 0;
        let mut length = 0;
        let name ;
        if b'(' == source[start] {
            name = "";
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
                    context.add_column();
                    continue;
                }
    
                if state == CallState::Name && t_i as char == '(' {
                    name = str::from_utf8(&source[start..start + length]).unwrap();
                    start += length;
                    start += 1;
                    break;
                }
            }
        }

        let end_blasket = ASTParser::read_valid_bracket(&source[start-1..], context.new_pos(), &context).unwrap();

        let s = str::from_utf8(source).unwrap();
        let first = s.find('(').unwrap();
        //let last = s.rfind(')').unwrap();
        let args = &source[start..end_blasket + start - 2];
        let tmp = std::str::from_utf8(args).unwrap();
        let sub_meta = context.new_pos();
        //let exprs = ASTParser::split_by_comma(args, sub_meta)?;
        let (expr, expr_len) = FSRExpr::parse(args, true, sub_meta, context).unwrap();
        let expr = Box::new(expr);
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
            len: start + expr_len,
            single_op: None,
            meta,
            is_defined: false,
        })
    }

    pub fn get_len(&self) -> usize {
        self.len
    }
}


mod test {
    use super::*;
    use crate::frontend::ast::parse::ASTParser;
    use crate::utils::error::SyntaxError;

    #[test]
    fn test_call() {
        let source = b"";
        let meta = FSRPosition::new();
        let mut context = ASTContext::new_context();
        let args = FSRExpr::parse(source, true, meta, &mut context).unwrap().0;
        let args = args.flatten_comma();
        println!("{:#?}", args);
    }
}