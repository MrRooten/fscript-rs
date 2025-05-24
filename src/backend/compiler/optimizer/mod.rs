use crate::{frontend::ast::token::base::FSRToken, utils::error::FSRError};

pub mod const_fold;

pub trait Optimizer {
    fn optimize(&self, token: &FSRToken) -> Result<FSRToken, FSRError>;
}