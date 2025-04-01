
use super::{
    base::{FSRPosition, FSRToken},
    expr::FSRExpr, ASTContext,
};
use crate::{frontend::ast::parse::ASTParser, utils::error::SyntaxError};
use std::str;

#[derive(Debug, Clone)]
pub struct FSRCall<'a> {
    name: &'a str,
    args: Vec<FSRToken<'a>>,
    pub(crate) len: usize,
    pub(crate) single_op: Option<&'a str>,
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

impl<'a> FSRCall<'a> {
    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }

    pub fn get_args(&self) -> &Vec<FSRToken<'a>> {
        &self.args
    }

    pub fn get_args_mut(&mut self) -> &mut Vec<FSRToken<'a>> {
        &mut self.args
    }

    pub fn get_name(&self) -> &'a str {
        self.name
    }

    pub fn parse(source: &'a [u8], meta: FSRPosition, context: &mut ASTContext, pre_args: bool) -> Result<Self, SyntaxError> {
        let mut state = CallState::Start;
        let mut start = 0;
        let mut length = 0;
        let name ;
        let mut fn_args = vec![];
        if b'(' == source[start] {
            name = str::from_utf8(&source[start..start + length]).unwrap();
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
    
                if state == CallState::Name && t_i as char == '(' {
                    name = str::from_utf8(&source[start..start + length]).unwrap();
                    start += length;
                    start += 1;
                    break;
                }
            }
        }
        

        let s = str::from_utf8(source).unwrap();
        let first = s.find('(').unwrap();
        let last = s.rfind(')').unwrap();
        let args = &source[first + 1..last];
        let sub_meta = meta.from_offset(start);
        let exprs = ASTParser::split_by_comma(args, sub_meta)?;
        for s in exprs {
            let sub_meta = meta.from_offset(first);
            let mut expr = FSRExpr::parse(s, true, sub_meta, context)?;
            if pre_args {
                match &expr.0 {
                    FSRToken::Variable(v) => {
                        context.add_variable(v.get_name());
                    },
                    FSRToken::Assign(a) => {
                        context.add_variable(a.get_name());
                    },
                    _ => {}
                }
            } else if let FSRToken::Variable(v) = &mut expr.0 {
                if context.is_variable_defined_in_curr(v.get_name()) {
                    v.is_defined = true
                } else {
                    context.ref_variable(v.get_name());
                }
            }
            
            fn_args.push(expr.0);
        }
        Ok(Self {
            name,
            args: fn_args,
            len: 0,
            single_op: None,
            meta,
            is_defined: false,
        })
    }

    pub fn get_len(&self) -> usize {
        self.len
    }
}
