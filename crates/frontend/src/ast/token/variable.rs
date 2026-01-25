
use crate::ast::SyntaxError;

use super::{base::{FSRPosition, FSRTypeName}, expr::SingleOp};

#[derive(Debug, Clone)]
pub struct FSRVariable {
    pub name: String,
    pub single_op: Option<SingleOp>,
    pub len: usize,
    pub meta: FSRPosition,
    pub is_defined: bool,
    pub var_type: Option<FSRTypeName>,
    pub force_type: bool
}

impl FSRVariable {
    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }
    pub fn parse(name: &str, meta: FSRPosition, var_type: Option<FSRTypeName>) -> Result<FSRVariable, SyntaxError> {
        
        Ok(Self {
            name: name.to_string(),
            single_op: None,
            len: 0,
            meta,
            is_defined: false,
            var_type,
            force_type: false,
        })
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_len(&self) -> usize {
        self.len
    }

    pub fn set_parse_len(&mut self, len: usize) {
        self.len = len;
    }

    /// Set the variable type
    /// If force_type is true, the type will not be set even if it is already set
    pub fn set_type_hint(&mut self, var_type: Option<FSRTypeName>) {
        if self.force_type{
            return;
        }
        self.var_type = var_type;
    }

    pub fn get_type_hint(&self) -> Option<&FSRTypeName> {
        self.var_type.as_ref()
    }
}
