use crate::{frontend::ast::token::base::FSRToken, utils::error::FSRError};

use super::ASTOptimizer;

pub struct ConstFoldOptimizer;

impl ASTOptimizer for ConstFoldOptimizer {
    fn optimize(&self, token: &FSRToken) -> Result<FSRToken, FSRError> {
        // Implement the constant folding optimization logic here
        Ok(token.clone())
    }
}