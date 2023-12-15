#[derive(Debug)]
pub struct FSRVariable<'a> {
    name    : &'a str
}

impl<'a> FSRVariable<'a> {
    pub fn parse(name: &'a str) -> Result<FSRVariable, &str> {
        Ok(
            Self {
                name: name
            }
        )
    }
}