#[derive(Debug, Clone)]
pub struct FSRSlice<'a> {
    _name: &'a str,
    _start: usize,
    _end: usize,
}


impl FSRSlice<'_> {
    pub fn parse(_source: &[u8]) -> Result<Self, &str> {
        unimplemented!()
    }

    pub fn is_valid_char(c: u8) -> bool {
        let c = c as char;
        c.is_ascii_hexdigit() || c == ':'
    }
}
