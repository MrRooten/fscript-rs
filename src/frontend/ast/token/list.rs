use crate::{
    frontend::ast::{parse::ASTParser, token::expr::FSRExpr},
    utils::error::SyntaxError,
};

use super::{base::{FSRPosition, FSRToken}, ASTContext};

#[derive(Debug, Clone)]
pub struct FSRListFrontEnd {
    items: Vec<FSRToken>,
    #[allow(unused)]
    pub(crate) len: usize,

    meta: FSRPosition,
}

impl FSRListFrontEnd {
    pub fn get_items(&self) -> &Vec<FSRToken> {
        &self.items
    }

    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }

    pub fn parse(source: &[u8], meta: FSRPosition, context: &mut ASTContext) -> Result<FSRListFrontEnd, SyntaxError> {
        //let tokens = ASTParser::split_by_comma(&source[1..1 + source.len() - 2], sub_meta)?;
        let expr = FSRExpr::parse(&source[1..1 + source.len() - 2], true, context.new_pos(), context).unwrap();
        let vs = expr.0.flatten_comma();
        // let mut start = 1;
        // for t in tokens {
        //     let mut _sub_meta = meta.clone();
        //     _sub_meta.offset += start;
        //     let token = FSRExpr::parse(t, true, _sub_meta, context)?;
        //     vs.push(token.0);
        //     start += t.len();
        //     start += 1; //escape comma
        // }
        Ok(Self {
            items: vs,
            len: source.len(),
            meta,
        })
    }
}
