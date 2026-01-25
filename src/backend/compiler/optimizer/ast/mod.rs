use crate::{utils::error::FSRError};
use frontend::ast::token::base::FSRToken;
pub mod const_fold;

pub trait ASTOptimizer {
    fn optimize(&self, token: &FSRToken) -> Result<FSRToken, FSRError>;
}