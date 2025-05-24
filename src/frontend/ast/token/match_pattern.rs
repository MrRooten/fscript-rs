use super::{base::{FSRPosition, FSRToken}, expr::FSRExpr};

pub struct FSRMatchPattern {
    pub(crate) expr: FSRExpr,
    pub(crate) patterns: Vec<(FSRExpr, FSRToken)>,
    pub(crate) meta: FSRPosition
}