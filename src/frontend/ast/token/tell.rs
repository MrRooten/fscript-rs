use crate::{frontend::ast::{parse::ASTParser, token::ASTContext}, utils::error::SyntaxError};

use super::base::FSRPosition;

#[derive(Debug, Clone)]
pub struct FSRTell {
    pub(crate) position: FSRPosition,
    pub(crate) value: Vec<String>,
    pub(crate) len: usize,
}

impl FSRTell {
    // parse something like
    // @abc
    // @static
    // contains multiple lines
    pub fn parse(source: &[u8], mut position: FSRPosition) -> Result<FSRTell, SyntaxError> {
        let mut start = 0;
        let mut len = 0;
        let mut res = vec![];
        loop {
            if start + len >= source.len() {
                start = start + len;
                return Err(SyntaxError::new_with_type(
                    &position,
                    "attribute must follow more token",
                    crate::utils::error::SyntaxErrType::NotMatchAttribute,
                ));
            }

            while start + len < source.len()
                && ASTParser::is_blank_char_with_new_line(source[start + len])
            {
                if source[start + len] == b'\n' {
                    position.line += 1;
                }
                len += 1;
            }

            if start + len >= source.len() {
                start = start + len;
                return Err(SyntaxError::new_with_type(
                    &position,
                    "attribute must follow more token",
                    crate::utils::error::SyntaxErrType::NotMatchAttribute,
                ));
            }

            start = start + len;
            len = 0;
            while start + len < source.len() && source[start + len] != b'\n' {
                len += 1; 
            }

            position.line += 1;

            if start + len >= source.len() {
                start = start + len;
                return Err(SyntaxError::new_with_type(
                    &position,
                    "attribute must follow more token",
                    crate::utils::error::SyntaxErrType::NotMatchAttribute,
                ));
            }

            let may_attr = std::str::from_utf8(&source[start..start + len])
                .unwrap()
                .trim();
            if !may_attr.starts_with("@") {
                break;
            }

            res.push(may_attr.to_string());

            start = start + len;
            len = 0;
        }

        Ok(FSRTell {
            position,
            value: res,
            len: start,
        })
    }

    pub fn position(&self) -> &FSRPosition {
        &self.position
    }

    pub fn value(&self) -> &[String] {
        &self.value
    }
}

mod test {
    use crate::frontend::ast::token::{base::FSRPosition, tell::FSRTell};

    #[test]
    fn test() {
        let a = "
        @async
        @jit
        ";

        let tell = FSRTell::parse(a.as_bytes(), FSRPosition::new());
        if tell.is_ok() {
            assert!(false, "not a valid tell, should be error")
        }
    }
}
