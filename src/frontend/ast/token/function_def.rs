#![allow(unused)]

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    chars_to_string, frontend::ast::{
        parse::ASTParser,
        token::{block::FSRBlock, call::FSRCall, expr::FSRExpr, variable::FSRVariable},
    }, utils::error::SyntaxError
};

use super::{
    base::{FSRPosition, FSRToken, FSRTypeName},
    tell::FSRTell,
    ASTContext, ASTVariableState,
};

#[derive(Debug, Clone)]
pub struct FSRFnDef {
    pub(crate) teller: Option<FSRTell>,
    lambda: bool,
    name: String,
    args: Vec<FSRToken>,
    body: Rc<FSRBlock>,
    len: usize,
    meta: FSRPosition,
    pub(crate) ret_type: Option<FSRTypeName>,
    pub(crate) ref_map: Rc<RefCell<HashMap<String, ASTVariableState>>>,
}

#[derive(PartialEq, Clone)]
enum State {
    SingleQuote,
    DoubleQuote,
    EscapeNewline,
    EscapeQuote,
    Continue,
}

const FN_IDENTIFY: &str = "fn";

impl FSRFnDef {
    pub fn clone_ref_map(&self) -> HashMap<String, ASTVariableState> {
        self.ref_map.borrow().clone()
    }

    pub fn is_lambda(&self) -> bool {
        self.lambda
    }

    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_len(&self) -> usize {
        self.len
    }

    pub fn get_args(&self) -> &Vec<FSRToken> {
        &self.args
    }

    pub fn get_body(&self) -> &FSRBlock {
        &self.body
    }

    pub fn is_jit(&self) -> bool {
        self.teller
            .as_ref()
            .map(|x| x.value.iter().any(|x| x.eq("@jit") || x.eq("@static") || x.eq("@entry")))
            .unwrap_or(false)
    }

    pub fn is_async(&self) -> bool {
        self.teller
            .as_ref()
            .map(|x| x.value.iter().any(|x| x.eq("@async")))
            .unwrap_or(false)
    }

    pub fn is_static(&self) -> bool {
        self.teller
            .as_ref()
            .map(|x| x.value.iter().any(|x| x.eq("@static") || x.eq("@entry")))
            .unwrap_or(false)
    }

    pub fn is_static_entry(&self) -> bool {
        self.teller
            .as_ref()
            .map(|x| x.value.iter().any(|x| x.eq("@entry")))
            .unwrap_or(false)
    }

    pub fn parse_lambda(
        source: &[char],
        meta: FSRPosition,
        name: &str,
        context: &mut ASTContext,
    ) -> Result<Self, SyntaxError> {
        if source[0] != '|' {
            let sub_meta = meta.new_offset(0);
            let err = SyntaxError::new(&sub_meta, "Invalid lambda function");
            return Err(err);
        }
        let mut args_len = 1;

        while args_len < source.len() {
            if source[args_len] == '|' {
                break;
            }

            args_len += 1;
        }

        if args_len == source.len() {
            let sub_meta = meta.new_offset(1);
            let err = SyntaxError::new(&sub_meta, "Invalid lambda function, args not closed");
            return Err(err);
        }

        let args = &source[1..args_len];

        // let args_s = std::str::from_utf8(args).unwrap();
        let args_s = chars_to_string!(args);
        let mut arg_collect = if args_s.trim().is_empty() {
            vec![]
        } else {
            let args_define = args_s
                .split(",")
                .enumerate()
                .filter(|x| !x.1.is_empty())
                .collect::<Vec<_>>();

            let mut arg_collect = vec![];
            // check arg is valid variable name
            for pos_arg in args_define {
                let arg = pos_arg.1.trim().chars().collect::<Vec<char>>();
                if arg.is_empty() {
                    let sub_meta = meta.new_offset(1 + pos_arg.0);
                    let err = SyntaxError::new(&sub_meta, "Invalid lambda function, empty arg");
                    return Err(err);
                }

                let b_arg = &arg;

                let mut i = 0;
                if ASTParser::is_name_letter_first(b_arg[0]) {
                    i += 1;
                } else {
                    let sub_meta = meta.new_offset(1 + pos_arg.0);
                    let err = SyntaxError::new(&sub_meta, "Invalid lambda function, invalid arg");
                    return Err(err);
                }

                while i < b_arg.len() {
                    if !ASTParser::is_name_letter(b_arg[i]) {
                        let sub_meta = meta.new_offset(1 + pos_arg.0);
                        let err =
                            SyntaxError::new(&sub_meta, "Invalid lambda function, invalid arg");
                        return Err(err);
                    }

                    i += 1;
                }
                let chars = chars_to_string!(arg);
                let mut variable = FSRVariable::parse(
                    &chars,
                    meta.new_offset(1 + pos_arg.0),
                    Some(FSRTypeName::new("Function")),
                )?;
                variable.is_defined = true;
                arg_collect.push(FSRToken::Variable(variable));
            }

            arg_collect
        };

        while source[args_len] != '{' {
            args_len += 1;
        }

        context.push_scope();
        for arg in &arg_collect {
            if let FSRToken::Variable(v) = arg {
                context.add_variable(&v.name, Some(arg.clone()));
            } else {
                unimplemented!("Lambda function args should be variables");
            }
        }
        // check is end of source
        if args_len == source.len() {
            let sub_meta = meta.new_offset(1);
            let err = SyntaxError::new(&sub_meta, "Invalid lambda function, body not found");
            return Err(err);
        }

        let sub_meta = meta.new_offset(args_len);
        let fn_block_len =
            ASTParser::read_valid_bracket(&source[args_len..], sub_meta.clone(), context)?;
        let fn_block = FSRBlock::parse(
            &source[args_len..args_len + fn_block_len],
            sub_meta,
            context,
            None,
        )?;
        let scope = context.pop_scope();
        Ok(Self {
            name: name.to_string(),
            args: arg_collect,
            body: Rc::new(fn_block),
            len: args_len + fn_block_len,
            meta,
            lambda: true,
            ref_map: scope,
            ret_type: None,
            teller: None,
        })
    }

    fn parse_ret_type(
        source: &[char],
        meta: FSRPosition,
        context: &mut ASTContext,
    ) -> Result<Option<FSRTypeName>, SyntaxError> {
        //let process_str = std::str::from_utf8(source).unwrap();
        let mut process_str = chars_to_string!(source);
        let process_str = process_str.trim();

        if process_str.is_empty() {
            return Ok(None);
        }

        if process_str.starts_with("->") {
            let mut start = 2;
            while start < process_str.len()
                && ASTParser::is_blank_char(process_str.as_bytes()[start] as char)
            {
                start += 1;
            }
            let mut end = process_str.len();

            let type_name = &process_str[start..end];
            let type_name = type_name.trim();
            if type_name.is_empty() {
                return Ok(None);
            }
            let type_name = &type_name.chars().collect::<Vec<char>>();

            let type_hint_expr = FSRExpr::parse(type_name, false, meta.clone(), context)?.0;
            let type_name = FSRExpr::parse_type_hint(&type_hint_expr, &meta);
            //let type_name = FSRTypeName::new(type_name);
            return Ok(Some(type_name));
        }

        Err(SyntaxError::new(
            &meta,
            "Invalid return type, should start with '->'",
        ))
    }

    fn count_line(source: &[char], len: usize, context: &mut ASTContext) {
        let mut i = 0;
        while i < len {
            i += 1;
        }
    }

    fn teller_parse(
        source: &[char],
        meta: FSRPosition,
        context: &mut ASTContext,
    ) -> Result<(Option<FSRTell>, usize), SyntaxError> {
        let mut start = 0;
        let teller = if source[0] == '@' {
            let teller = FSRTell::parse(source, meta.new_offset(0))?;
            Self::count_line(source, teller.len, context);
            start += teller.len;

            while start < source.len() && ASTParser::is_blank_char_with_new_line(source[start]) {
                start += 1;
            }
            Some(teller)
        } else {
            None
        };

        Ok((teller, start))
    }

    fn get_parse_len(source: &[char], meta: FSRPosition, start: usize) -> Result<usize, SyntaxError> {
        let mut state = State::Continue;
        let mut pre_state = State::Continue;
        let mut len = 0;
        for c in &source[2..] {
            let c = *c as char;
            len += 1;
            if c == '{' && (state != State::DoubleQuote && state != State::SingleQuote) {
                len -= 1;
                break;
            }

            if c == '\n' {
                let sub_meta = meta.new_offset(start);
                let err = SyntaxError::new(&sub_meta, "Invalid If statement");
                return Err(err);
            }

            if state == State::EscapeQuote {
                state = pre_state.clone();
                continue;
            }

            if c == '\'' && state == State::Continue {
                state = State::SingleQuote;
                continue;
            }

            if c == '\'' && state == State::SingleQuote {
                state = State::Continue;
                continue;
            }

            if c == '\"' && state == State::DoubleQuote {
                state = State::DoubleQuote;
                continue;
            }

            if c == '\"' && state == State::DoubleQuote {
                state = State::Continue;
                continue;
            }

            if c == '\\' && (state == State::DoubleQuote || state == State::SingleQuote) {
                pre_state = state;
                state = State::EscapeQuote;
            }
        }

        Ok(len)
    }

    pub fn parse(
        source: &[char],
        meta: FSRPosition,
        context: &mut ASTContext,
        struct_info: Option<String>,
    ) -> Result<Rc<Self>, SyntaxError> {
        let (teller, start) = Self::teller_parse(&source, meta.clone(), context)?;

        let source = &source[start..];
        // let s = std::str::from_utf8(&source[0..FN_IDENTIFY.len()]).unwrap();
        let s = chars_to_string!(&source[0..FN_IDENTIFY.len()]);

        if source.len() < 3 {
            let sub_meta = meta.new_offset(start);
            let err = SyntaxError::new(&sub_meta, "fn define body length too small");
            return Err(err);
        }
        if s != FN_IDENTIFY {
            let sub_meta = meta.new_offset(start);
            let err = SyntaxError::new(&sub_meta, "not fn token");
            return Err(err);
        }

        // if source[2] as char != ' ' {
        if !ASTParser::is_blank_char(source[FN_IDENTIFY.len()]) {
            let sub_meta = meta.new_offset(start);
            let err = SyntaxError::new(&sub_meta, "not a valid fn delemiter");
            return Err(err);
        }

        let len = Self::get_parse_len(source, meta.new_offset(start), start)?;

        let mut start_fn_name = 0;
        while ASTParser::is_blank_char(source[FN_IDENTIFY.len()..][start_fn_name]) {
            start_fn_name += 1;
        }

        if !ASTParser::is_name_letter_first(source[FN_IDENTIFY.len()..][start_fn_name]) {
            let sub_meta = meta.new_offset(2 + start_fn_name);
            let err = SyntaxError::new(&sub_meta, "Invalid function name");
            return Err(err);
        }

        start_fn_name += 1;
        while !ASTParser::is_name_letter(source[FN_IDENTIFY.len()..][start_fn_name]) {
            start_fn_name += 1;
        }

        let fn_args = &source[start_fn_name..start_fn_name + len];
        let sub_meta = meta.new_offset(start);

        context.push_scope();
        let mut fn_call = FSRCall::parse(fn_args, sub_meta, context, true)?;
        let call_len = fn_call.get_len();
        let name = fn_call.get_name().to_string();

        let mut gap_call_len = 0;
        while call_len + gap_call_len + 1 < len
            && source[start_fn_name + call_len + gap_call_len + 1] != '{'
        {
            gap_call_len += 1;
        }

        let ret_type_str =
            &source[start_fn_name + call_len + 1..start_fn_name + call_len + 1 + gap_call_len];
        let ret_type = Self::parse_ret_type(
            ret_type_str,
            meta.new_offset(start_fn_name + call_len + 1),
            context,
        )?;

        context.add_var_prev_define(&name, None);

        let fn_block_start = start_fn_name + len;

        let fn_block_len = ASTParser::read_valid_bracket(
            &source[fn_block_start..],
            meta.new_offset(fn_block_start),
            context,
        )?;
        let block_meta = meta.new_offset(fn_block_start);
        let mut is_first = true;
        for arg in fn_call.get_args_mut() {
            if let FSRToken::Variable(v) = arg {
                if is_first {
                    if let Some(struct_name) = &struct_info {
                        if v.get_type_hint().is_none() {
                            v.var_type = Some(FSRTypeName::new(struct_name));
                        }
                        
                    }
                }
                let clone = v.clone();
                v.is_defined = true;
                context.add_variable(v.get_name(), Some(FSRToken::Variable(clone)));
            } else {
                panic!("Function args should be variables");
            }

            is_first = false;
        }
        let fn_block = FSRBlock::parse(
            &source[fn_block_start..fn_block_start + fn_block_len],
            block_meta,
            context,
            None,
        )?;

        let cur = context.pop_scope();

        let fn_def = Self {
            name: name.to_string(),
            args: fn_call.get_args().clone(),
            body: Rc::new(fn_block),
            len: start + fn_block_start + fn_block_len,
            meta,
            lambda: false,
            ref_map: cur,
            ret_type,
            teller,
        };

        let fn_def = Rc::new(fn_def);
        context.add_variable(&name, None);
        context.set_variable_token(&name, Some(FSRToken::FunctionDef(fn_def.clone())));

        Ok(fn_def)
    }
}

mod test {
    use crate::frontend::ast::token::base::FSRPosition;

    #[test]
    fn test_lambda() {
        let source = b"|a,b|{a+b}";
        let meta = FSRPosition::new();
        let mut context = super::ASTContext::new_context();
        let source = &source.iter().map(|&c| c as char).collect::<Vec<char>>();
        let result = super::FSRFnDef::parse_lambda(source, meta, "lambda_xxxx", &mut context);
        assert!(result.is_ok());
        println!("{:#?}", result.unwrap());
    }
}
