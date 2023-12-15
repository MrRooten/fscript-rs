use std::fmt::Error;

use super::base::FSRToken;
use super::statement::ASTTokenInterface;
use super::statement::ASTTokenEnum;



#[derive(Debug)]
pub struct FSRIf<'a> {
    test    : Box<FSRToken<'a>>,
    body    : Vec<FSRToken<'a>>,
    len     : usize,
}

impl FSRIf<'_> {
    pub fn parse(source: &[u8]) -> Result<FSRIf, Error> {
        unimplemented!()
    }

    pub fn parse_len(&self) -> usize {
        return self.len;
    }
}

pub enum FSRIfState {
    IfTestStart,
    IfTestEnd,
    IfBodyStart,
    IfBodyEnd
}

impl ASTTokenInterface for FSRIf<'_> {
    fn get_expect_states() -> Vec<ASTTokenEnum> {
        unimplemented!()
    }
}