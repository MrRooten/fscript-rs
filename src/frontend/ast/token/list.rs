use crate::{
    frontend::ast::{parse::ASTParser, token::expr::FSRExpr},
    utils::error::SyntaxError,
};

use super::base::{FSRPosition, FSRToken};

#[derive(Debug, Clone)]
pub struct FSRListFrontEnd<'a> {
    items: Vec<FSRToken<'a>>,
    pub(crate) len: usize,

    meta: FSRPosition,
}

impl<'a> FSRListFrontEnd<'a> {
    pub fn get_items(&self) -> &Vec<FSRToken<'a>> {
        &self.items
    }

    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }

    pub fn parse(source: &'a [u8], meta: FSRPosition) -> Result<FSRListFrontEnd<'a>, SyntaxError> {
        let mut vs = vec![];
        let mut sub_meta = meta.clone();
        sub_meta.offset += 1;
        let tokens = ASTParser::split_by_comma(&source[1..1 + source.len() - 2], sub_meta)?;
        let mut start = 1;
        for t in tokens {
            let mut _sub_meta = meta.clone();
            _sub_meta.offset += start;
            let token = FSRExpr::parse(t, true, _sub_meta)?;
            vs.push(token.0);
            start += t.len();
            start += 1; //escape comma
        }
        Ok(Self {
            items: vs,
            len: source.len(),
            meta,
        })
    }
}
