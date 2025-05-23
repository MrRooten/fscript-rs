use crate::utils::error::SyntaxError;

use super::base::{FSRPosition, FSRType};

#[derive(Debug, Clone)]
pub struct FSRVariable<'a> {
    pub(crate) name: &'a str,
    pub(crate) single_op: Option<&'a str>,
    pub(crate) type_hint: Option<&'a str>,
    pub(crate) len: usize,
    pub(crate) meta: FSRPosition,
    pub(crate) is_defined: bool,
    pub(crate) var_type: FSRType
}

impl<'a> FSRVariable<'a> {
    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }
    pub fn parse(name: &'a str, meta: FSRPosition, var_type: FSRType) -> Result<FSRVariable<'a>, SyntaxError> {
        
        Ok(Self {
            name,
            single_op: None,
            len: 0,
            meta,
            type_hint: None,
            is_defined: false,
            var_type,
        })
    }

    pub fn get_name(&self) -> &'a str {
        self.name
    }

    pub fn get_len(&self) -> usize {
        self.len
    }

    pub fn set_parse_len(&mut self, len: usize) {
        self.len = len;
    }
}
