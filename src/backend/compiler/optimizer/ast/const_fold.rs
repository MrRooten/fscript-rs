use crate::{utils::error::FSRError};
use frontend::ast::token::base::FSRToken;
use super::ASTOptimizer;

pub struct ConstFoldOptimizer;

impl ASTOptimizer for ConstFoldOptimizer {
    fn optimize(&self, token: &FSRToken) -> Result<FSRToken, FSRError> {
        // Implement the constant folding optimization logic here
        Ok(token.clone())
    }
}