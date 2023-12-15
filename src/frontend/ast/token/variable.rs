#[derive(Debug)]
pub struct FSRVariable<'a> {
    name    : &'a str,
    len     : usize
}

impl<'a> FSRVariable<'a> {
    pub fn parse(name: &'a str) -> Result<FSRVariable, &str> {
        Ok(
            Self {
                name: name,
                len : 0
            }
        )
    }

    pub fn parse_len(&self) -> usize {
        return self.len;
    }

    pub fn set_parse_len(&mut self, len: usize) {
        self.len = len;
    }
}