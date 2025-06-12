use crate::backend::types::class::FSRClass;

#[derive(Debug, PartialEq, Clone)]
pub struct FSRInnerBytes {
    pub(crate) bytes: Vec<u8>,
}

impl FSRInnerBytes {
    pub fn new(bytes: Vec<u8>) -> Self {
        FSRInnerBytes { bytes }
    }

    pub fn get_bytes(&self) -> &Vec<u8> {
        &self.bytes
    }

    pub fn to_string(&self) -> String {
        String::from_utf8_lossy(&self.bytes).to_string()
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn get_class() -> FSRClass<'static> {
        let cls = FSRClass::new("Bytes");
        return cls
    }
}