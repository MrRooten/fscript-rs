use crate::{ast::SyntaxError, chrs2str};

use super::{
    base::{FSRPosition, FSRToken},
    expr::FSRExpr, ASTContext,
};
use std::str;
#[derive(Debug, Clone)]
pub struct FSRDefer {
    expr: Box<FSRToken>,
    meta: FSRPosition,
}

const DEFER_STR: &str = "defer";

impl FSRDefer {
    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }

    pub fn get_defer_expr(&self) -> &FSRToken {
        &self.expr
    }

    pub fn parse(source: &[char], meta: FSRPosition, context: &mut ASTContext) -> Result<(Self, usize), SyntaxError> {
        let mut len = 0;
        let sub = &source[0..DEFER_STR.len()];
        let first_5_char = chrs2str!(sub);
        if !first_5_char.eq(DEFER_STR) {
            let err = SyntaxError::new(&meta, "Not a defer token");
            return Err(err);
        }

        let start_expr = DEFER_STR.len();
        len += DEFER_STR.len();
        let expr = &source[start_expr..];
        let sub_meta = meta.new_offset(start_expr);
        let expr = match FSRExpr::parse(expr, false, sub_meta, context) {
            Ok(o) => o,
            Err(e) => {
                return Err(e);
            }
        };

        len += expr.1;
        Ok((
            Self {
                expr: Box::new(expr.0),
                meta,
            },
            len,
        ))
    }
}