use crate::frontend::ast::token::base::FSRToken;

#[derive(Debug, Clone)]
pub struct FSRSlice<'a> {
    name        : &'a str,
    start       : usize,
    end         : usize
}

#[derive(PartialEq)]
enum SliceState {
    Name,
    Start,
    Args,
    WaitToken
}

impl FSRSlice<'_> {

    pub fn parse(source: &[u8]) -> Result<Self, &str> {
        unimplemented!()
    }

    pub fn is_valid_char(c: u8) -> bool {
        let c = c as char;
        c.is_ascii_hexdigit() || c == ':'
    }
}