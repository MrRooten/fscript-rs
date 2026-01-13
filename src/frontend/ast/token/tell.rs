use crate::{chars_to_string, frontend::ast::{parse::ASTParser, token::ASTContext}, utils::error::SyntaxError};

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
    pub fn parse(source: &[char], position: FSRPosition) -> Result<FSRTell, SyntaxError> {
        let mut start = 0;
        let mut len = 0;
        let mut res = vec![];
        loop {
            if start + len >= source.len() {
                return Err(SyntaxError::new_with_type(
                    &position,
                    "attribute must follow more token",
                    crate::utils::error::SyntaxErrType::NotMatchAttribute,
                ));
            }

            while start + len < source.len()
                && ASTParser::is_blank_char_with_new_line(source[start + len])
            {
                len += 1;
            }

            if start + len >= source.len() {
                return Err(SyntaxError::new_with_type(
                    &position,
                    "attribute must follow more token",
                    crate::utils::error::SyntaxErrType::NotMatchAttribute,
                ));
            }

            start += len;
            len = 0;
            while start + len < source.len() && source[start + len] != '\n' {
                len += 1; 
            }

            if start + len >= source.len() {
                return Err(SyntaxError::new_with_type(
                    &position,
                    "attribute must follow more token",
                    crate::utils::error::SyntaxErrType::NotMatchAttribute,
                ));
            }

            // let may_attr = std::str::from_utf8(&source[start..start + len])
            //     .unwrap()
            //     .trim();
            let may_attr = chars_to_string!(&source[start..start + len]);
            let may_attr = may_attr.trim();
            if !may_attr.starts_with("@") {
                break;
            }

            res.push(may_attr.to_string());

            start += len;
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
    use crate::chars_to_string;

    

    #[test]
    fn test() {
        use crate::frontend::ast::token::{base::FSRPosition, tell::FSRTell};
        let a = "
        @async
        @static
        ";
        let a = a.chars().collect::<Vec<char>>();
        let tell = FSRTell::parse(&a, FSRPosition::new());
        if tell.is_ok() {
            assert!(false, "not a valid tell, should be error")
        }
    }
}
