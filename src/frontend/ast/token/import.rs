use crate::utils::error::SyntaxError;

use super::{base::FSRPosition, ASTContext};
use std::str;
#[derive(Debug, Clone)]
pub struct FSRImport {
    pub(crate) module_name: Vec<String>,
    meta: FSRPosition,
}

impl FSRImport {
    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }

    pub fn parse(source: &[u8], meta: FSRPosition, context: &mut ASTContext) -> Result<(Self, usize), SyntaxError> {
        let mut len = 0;
        while len < source.len() && source[len] != b'\n' {
            if source[len] as char == '\\' {
                len += 1;
            }
            len += 1;
        }

        let sub = str::from_utf8(&source[0..len]).unwrap();
        if !sub.starts_with("import") {
            return Err(SyntaxError::new(
                &meta.clone(),
                "Expected 'import' keyword at the start of import statement",
                
            ));
        }

        let module_start = sub.find(' ').unwrap();
        let mod_name = sub[module_start..len].trim();

        context.add_variable(mod_name.split('.').last().unwrap(), None);
        Ok((
            Self {
                module_name: mod_name.split('.').map(|s| s.to_string()).collect(),
                meta,
            },
            len,
        ))
    }
}
