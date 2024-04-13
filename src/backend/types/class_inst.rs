use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FSRClassInst<'a> {
    name        : &'a str,
    attrs       : HashMap<&'a str, u64>
}

impl<'a> FSRClassInst<'a> {
    pub fn new(name: &'a str) -> FSRClassInst<'a> {
        Self {
            name,
            attrs: HashMap::new(),
        }
    }

    pub fn get_attr(&self, name: &str) -> Option<&u64> {
        return self.attrs.get(name);
    }

    pub fn set_attr(&mut self, name: &'a str, value: u64) {
        self.attrs.insert(name, value);
    }
}