use super::base::FSRPosition;

#[derive(Debug, Clone)]
pub struct FSRVariable<'a> {
    pub(crate) name: &'a str,
    pub(crate) single_op: Option<&'a str>,
    pub(crate) len: usize,
    pub(crate) meta: FSRPosition,
}

impl<'a> FSRVariable<'a> {
    pub fn get_meta(&self) -> &FSRPosition {
        &self.meta
    }
    pub fn parse(name: &'a str, meta: FSRPosition) -> Result<FSRVariable<'a>, &'a str> {
        Ok(Self {
            name,
            single_op: None,
            len: 0,
            meta,
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
