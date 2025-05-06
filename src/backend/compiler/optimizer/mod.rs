use crate::{frontend::ast::token::base::FSRToken, utils::error::FSRError};

pub mod const_fold;

pub trait Optimizer {
    fn optimize<'a>(&self, token: &FSRToken<'a>) -> Result<FSRToken<'a>, FSRError>;
}