use crate::utils::error::SyntaxError;

use super::{
    base::{FSRPosition, FSRToken},
    expr::FSRExpr, ASTContext,
};
use std::str;
#[derive(Debug, Clone)]
pub struct FSRReturn {
    expr: Box<FSRToken>,
    meta: FSRPosition,
}

impl FSRReturn {
    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }

    pub fn get_return_expr(&self) -> &FSRToken {
        &self.expr
    }

    pub fn parse(source: &[u8], meta: FSRPosition, context: &mut ASTContext) -> Result<(Self, usize), SyntaxError> {
        let mut len = 0;
        let sub = &source[0..6];
        let first_6_char = str::from_utf8(sub).unwrap();
        if !first_6_char.eq("return") {
            let err = SyntaxError::new(&meta, "Not a return token");
            return Err(err);
        }

        let start_expr = 6;
        len += 6;
        let expr = &source[start_expr..];
        let mut sub_meta = context.new_pos();
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
