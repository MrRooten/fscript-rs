use std::collections::HashMap;

pub struct FSRClassInst<'a> {
    attrs       : HashMap<&'a str, u64>
}

impl FSRClassInst<'_> {
    pub fn new(name: &str) -> FSRClassInst {
        unimplemented!()
    }
}