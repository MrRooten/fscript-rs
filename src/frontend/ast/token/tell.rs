use crate::utils::error::SyntaxError;

use super::base::FSRPosition;

#[derive(Debug, Clone)]
pub struct FSRTell {
    pub(crate) position: FSRPosition,
    pub(crate) value: String,
    pub(crate) len: usize,
}

impl FSRTell {
    // parse something like 
    // @abc
    // @static
    // contains multiple lines
    pub fn parse(source: &[u8], position: FSRPosition) -> Result<FSRTell, SyntaxError> {
        // read until new line of source
        let mut end = 0;
        while end < source.len() && source[end] != b'\n' {
            end += 1;
        }
        let value = String::from_utf8(source[0..end].to_vec()).unwrap();

        if value.is_empty() {
            return Err(SyntaxError::new(&position, "value is empty"));
        }

        Ok(FSRTell {
            position,
            value,
            len: end,
        })
    }

    pub fn position(&self) -> &FSRPosition {
        &self.position
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}