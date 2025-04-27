use crate::backend::types::base::{FSRObject, ObjId};



pub struct TripleColorGarbageCollector {
    pub white: Vec<Option<ObjId>>,
    pub gray: Vec<Option<ObjId>>,
    pub black: Vec<Option<ObjId>>,
}

impl TripleColorGarbageCollector {
    pub fn new() -> Self {
        TripleColorGarbageCollector {
            white: Vec::new(),
            gray: Vec::new(),
            black: Vec::new(),
        }
    }

    pub fn add_object(&mut self, obj: ObjId) {
        self.white.push(Some(obj));
    }

    pub fn mark_roots(&mut self, root_indices: &[usize]) {
        for &index in root_indices {
            if index < self.white.len() {
                if let Some(obj) = self.white[index].take() {
                    self.gray.push(Some(obj));
                }
            }
        }
    }

    pub fn mark(&mut self) {
        while !self.gray.is_empty() {
            if let Some(obj_id) = self.gray.pop().unwrap() {
                let obj = FSRObject::id_to_obj(obj_id);
                // 获取对象引用的其他对象
                let references = obj.get_references();
                
                self.black.push(Some(obj_id));
                
                for ref_idx in references {
                    if ref_idx < self.white.len() {
                        if let Some(ref_obj) = self.white[ref_idx].take() {
                            self.gray.push(Some(ref_obj));
                        }
                    }
                }
            }
        }
    }


    pub fn sweep(&mut self) {

        self.white.clear();
    }


    pub fn reset(&mut self) {

        self.white.append(&mut self.black);

        debug_assert!(self.gray.is_empty(), "Gray set should be empty after marking");
    }


    pub fn collect(&mut self, root_indices: &[usize]) {
        self.mark_roots(root_indices);
        self.mark();
        self.sweep();
        self.reset();
    }
}