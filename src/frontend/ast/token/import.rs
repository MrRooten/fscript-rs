use crate::utils::error::SyntaxError;

use super::base::FSRPosition;
use std::str;
#[derive(Debug, Clone)]
pub struct FSRImport<'a> {
    _module_name: Vec<&'a str>,
    meta: FSRPosition,
}

impl<'a> FSRImport<'a> {
    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }

    pub fn parse(source: &'a [u8], meta: FSRPosition) -> Result<(Self, usize), SyntaxError> {
        let mut len = 0;
        while len < source.len() && source[len] != b'\n' {
            if source[len] as char == '\\' {
                len += 1;
            }
            len += 1;
        }

        let sub = str::from_utf8(&source[0..len]).unwrap();
        if !sub.starts_with("import") {
            unimplemented!()
        }

        let module_start = sub.find(' ').unwrap();
        let mod_name = sub[module_start..len].trim();

        Ok((
            Self {
                _module_name: mod_name.split('.').collect::<Vec<&str>>(),
                meta,
            },
            len,
        ))
    }
}
