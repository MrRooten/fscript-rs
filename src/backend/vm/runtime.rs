use std::collections::HashMap;

use super::thread::FSRThreadRuntime;

pub struct FSRVM {
    threads         : HashMap<u64, FSRThreadRuntime>,
}

impl FSRVM {
    pub fn new() -> Self {
        let main_thread = FSRThreadRuntime::new();
        let mut maps = HashMap::new();
        maps.insert(0, main_thread);
        let v = Self {
            threads: maps,
        };
        v
    }
}