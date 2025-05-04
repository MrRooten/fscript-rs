use super::{base::{FSRPosition, FSRToken}, expr::FSRExpr};

pub struct FSRMatchPattern<'a> {
    pub(crate) expr: FSRExpr<'a>,
    pub(crate) patterns: Vec<(FSRExpr<'a>, FSRToken<'a>)>,
    pub(crate) meta: FSRPosition
}