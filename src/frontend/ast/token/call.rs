use std::fmt::Error;

use super::base::FSRToken;

#[derive(Debug)]
pub struct FSRCall<'a> {
    name        : &'a str,
    args        : Vec<FSRToken<'a>>
}

enum CallState {
    Name,
    Start,
    Args
}

impl FSRCall<'_> {
    pub fn parse(source: &[u8]) -> Result<Self, Error> {

        unimplemented!()
    }

    pub fn parse_len(&self) -> usize {
        unimplemented!()
    }
}