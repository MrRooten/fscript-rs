use std::fmt::Error;

use super::base::FSRToken;

pub struct FSRBinOp<'a> {
    left        : Box<FSRToken<'a>>,
    right       : Box<FSRToken<'a>>,
    len         : usize
}

impl FSRBinOp<'_> {
    pub fn parse(source: &[u8]) -> Result<FSRBinOp, Error> {
        unimplemented!()
    }

    pub fn parse_len(&self) -> usize {
        return self.len;
    }
}