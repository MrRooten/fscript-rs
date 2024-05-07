use crate::utils::error::SyntaxError;

use super::base::FSRMeta;
use std::str;
#[derive(Debug, Clone)]
pub struct FSRImport {
    module_name: String,
    meta: FSRMeta,
}

impl FSRImport {
    pub fn get_meta(&self) -> &FSRMeta {
        &self.meta
    }

    pub fn parse(source: &[u8], meta: FSRMeta) -> Result<(Self, usize), SyntaxError> {
        let mut len = 0;
        while len < source.len() && source[len] == b'\n' {
            len += 1;
        }

        let sub = str::from_utf8(&source[0..len]).unwrap();
        if !sub.starts_with("import") {
            unimplemented!()
        }

        let module_start = sub.find(' ').unwrap();
        let mod_name = &sub[module_start..len];

        Ok((
            Self {
                module_name: mod_name.to_string(),
                meta,
            },
            len,
        ))
    }
}
