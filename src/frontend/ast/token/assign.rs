use std::fmt::Error;

use super::base::FSRToken;

pub struct FSRAssign<'a> {
    targets     : Vec<FSRToken<'a>>,
    value       : Vec<FSRToken<'a>>   
}

impl FSRAssign<'_> {
    pub fn parse(source: &[u8]) -> Result<Self, Error> {
        unimplemented!()
    }

    pub fn parse_len(&self) -> usize {
        unimplemented!()
    }
}