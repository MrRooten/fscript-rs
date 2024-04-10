use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FSRClassInst<'a> {
    attrs       : HashMap<&'a str, u64>
}

impl FSRClassInst<'_> {
    pub fn new(name: &str) -> FSRClassInst {
        unimplemented!()
    }

    pub fn get_attr(&self, name: &str) -> Option<&u64> {
        return self.attrs.get(name);
    }
}