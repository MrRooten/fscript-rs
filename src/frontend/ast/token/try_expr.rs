use core::panic;

use crate::frontend::ast::parse::ASTParser;
use crate::frontend::ast::token::block::FSRBlock;
use crate::frontend::ast::token::expr::FSRExpr;
use crate::utils::error::SyntaxError;

use super::base::FSRPosition;
use super::base::FSRToken;
use super::r#else::FSRElse;
use super::statement::ASTTokenEnum;
use super::statement::ASTTokenInterface;
use super::ASTContext;

#[derive(PartialEq, Clone)]
enum State {
    SingleQuote,
    DoubleQuote,
    _EscapeNewline,
    EscapeQuote,
    Continue,
}

#[derive(Debug, Clone)]
pub struct FSRCatch<'a> {
    pub(crate) body: Box<FSRBlock<'a>>,
    pub(crate) len: usize,
    pub(crate) meta: FSRPosition,
}

impl<'a> FSRCatch<'a> {
    pub fn parse(source: &'a [u8], meta: FSRPosition, context: &mut ASTContext) -> Result<FSRCatch<'a>, SyntaxError> {
        let s = std::str::from_utf8(&source[0..5]).unwrap();
        if source.len() < 5 {
            let sub_meta = meta.from_offset(0);
            let err = SyntaxError::new(&sub_meta, "if define body length too small");
            return Err(err);
        }
        if s != "catch" {
            let sub_meta = meta.from_offset(0);
            let err = SyntaxError::new(&sub_meta, "not if token");
            return Err(err);
        }

        if source[5] as char != ' ' && source[5] as char != '{' {
            let sub_meta = meta.from_offset(5);
            let err = SyntaxError::new(&sub_meta, "not a valid if delemiter");
            return Err(err);
        }

        let mut start = 5;
        while ASTParser::is_blank_char_with_new_line(source[start]) {
            start += 1;
        }

        let mut sub_meta = meta.clone();
        sub_meta.offset = meta.offset + start;
        let mut b_len = ASTParser::read_valid_bracket(&source[start..], sub_meta)?;
        let mut sub_meta = meta.clone();
        sub_meta.offset = meta.offset + start;
        let body = FSRBlock::parse(&source[start..start + b_len], sub_meta, context)?;

        start += b_len;
        b_len = 0;

        Ok(Self {
            body: Box::new(body),
            len: start + b_len,
            meta,
        })
    }

    pub fn get_len(&self) -> usize {
        self.len
    }
}

#[derive(Debug, Clone)]
pub struct FSRTryBlock<'a> {
    pub(crate) body: Box<FSRBlock<'a>>,
    #[allow(unused)]
    catch: Box<FSRCatch<'a>>,
    pub(crate) len: usize,
    pub(crate) meta: FSRPosition,
}

impl<'a> FSRTryBlock<'a> {
    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }

    pub fn get_block(&self) -> &FSRBlock {
        &self.body
    }

    pub fn parse(source: &'a [u8], meta: FSRPosition, context: &mut ASTContext) -> Result<FSRTryBlock<'a>, SyntaxError> {
        let s = std::str::from_utf8(&source[0..3]).unwrap();
        if source.len() < 3 {
            let sub_meta = meta.from_offset(0);
            let err = SyntaxError::new(&sub_meta, "try define body length too small");
            return Err(err);
        }
        if s != "try" {
            let sub_meta = meta.from_offset(0);
            let err = SyntaxError::new(&sub_meta, "not try token");
            return Err(err);
        }

        if source[3] as char != ' ' && source[3] as char != '{' {
            let sub_meta = meta.from_offset(3);
            let err = SyntaxError::new(&sub_meta, "not a valid try delemiter");
            return Err(err);
        }
        let mut start = 3;
        while ASTParser::is_blank_char_with_new_line(source[start]) {
            start += 1;
        }
        let sub_meta = meta.from_offset(start);
        if source[start] != b'{' {
            let err = SyntaxError::new(&sub_meta, "not a valid try delemiter");
            return Err(err);
        }
        let len = ASTParser::read_valid_bracket_until_big(&source[start..], sub_meta)?;

        let mut start = start + len;
        let mut sub_meta = meta.clone();
        sub_meta.offset = meta.offset + start;
        let mut b_len = ASTParser::read_valid_bracket(&source[start..], sub_meta)?;
        let mut sub_meta = meta.clone();
        sub_meta.offset = meta.offset + start;
        let body = FSRBlock::parse(&source[start..start + b_len], sub_meta, context)?;

        start += b_len;
        b_len = 0;
        while start < source.len() && ASTParser::is_blank_char_with_new_line(source[start]) {
            start += 1;
        }

        let catches = if start + 5 < source.len() {
            let may_else_token = std::str::from_utf8(&source[start..start + 5]).unwrap();
            if may_else_token.eq("catch") {
                let sub_meta = meta.from_offset(start);
                let catches = FSRCatch::parse(&source[start..], sub_meta, context)?;
                start += catches.get_len();
                Box::new(catches)
            } else {
                panic!("not catch token");
            }
        } else {
            panic!("not catch token");
        };
        Ok(Self {
            body: Box::new(body),
            len: start + b_len,
            catch: catches,
            meta,
        })
    }

    pub fn get_len(&self) -> usize {
        self.len
    }

    pub fn get_catch(&self) -> &FSRCatch {
        &self.catch
    }
}

mod test {
    #[test]
    fn test_try_expr() {
        use crate::frontend::ast::token::base::FSRPosition;
        use crate::frontend::ast::token::block::FSRBlock;
        use crate::frontend::ast::token::expr::FSRExpr;
        use crate::frontend::ast::token::r#else::FSRElse;
        use crate::frontend::ast::token::try_expr::FSRCatch;
        use crate::frontend::ast::token::try_expr::FSRTryBlock;
        use crate::utils::error::SyntaxError;

        let source = r#"try { 
    a = 1
} catch { 
 b = 2 
}"#;
        let meta = FSRPosition::new();
        let mut context = super::ASTContext::new_context();
        let try_expr = FSRTryBlock::parse(source.as_bytes(), meta, &mut context).unwrap();
        println!("{:#?}", try_expr);

        assert!(try_expr.get_len() == source.len());
    }
}
