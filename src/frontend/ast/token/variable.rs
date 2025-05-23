use crate::utils::error::SyntaxError;

use super::base::{FSRPosition, FSRType};

#[derive(Debug, Clone)]
pub struct FSRVariable<'a> {
    pub(crate) name: &'a str,
    pub(crate) single_op: Option<&'a str>,
    pub(crate) len: usize,
    pub(crate) meta: FSRPosition,
    pub(crate) is_defined: bool,
    pub(crate) var_type: Option<FSRType>,
    pub(crate) force_type: bool
}

impl<'a> FSRVariable<'a> {
    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }
    pub fn parse(name: &'a str, meta: FSRPosition, var_type: Option<FSRType>) -> Result<FSRVariable<'a>, SyntaxError> {
        
        Ok(Self {
            name,
            single_op: None,
            len: 0,
            meta,
            is_defined: false,
            var_type,
            force_type: false,
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

    /// Set the variable type
    /// If force_type is true, the type will not be set even if it is already set
    pub fn set_type_hint(&mut self, var_type: Option<FSRType>) {
        if self.force_type{
            return;
        }
        self.var_type = var_type;
    }

    pub fn get_type_hint(&self) -> Option<&FSRType> {
        self.var_type.as_ref()
    }
}
