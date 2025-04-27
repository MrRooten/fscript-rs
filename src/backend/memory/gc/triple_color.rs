use crate::backend::{memory::mempool::memory::TCMemoryManager, types::base::{FSRObject, ObjId, TripleColor}};



pub struct TripleColorGarbageCollector<'a> {
    //pub white: Vec<Option<ObjId>>,
    pub gray: Vec<Option<ObjId>>,
    pub black: Vec<Option<ObjId>>,
    pub memory: TCMemoryManager<'a>
}

impl<'a> TripleColorGarbageCollector<'a> {
    pub fn new() -> Self {
        TripleColorGarbageCollector {
            //white: Vec::new(),
            gray: Vec::new(),
            black: Vec::new(),
            memory: TCMemoryManager::new(),
        }
    }

    pub fn mark_roots(&mut self, root_indices: &[ObjId]) {
        for obj_id in root_indices {
            if let Some(obj) = FSRObject::id_to_mut_obj(*obj_id) {
                if obj.color == TripleColor::White {
                    obj.color = TripleColor::Gray;
                    self.gray.push(Some(*obj_id));
                }
            }
        }
    }

    pub fn mark(&mut self) {
        while !self.gray.is_empty() {
            if let Some(obj_id) = self.gray.pop().unwrap() {
                let obj = match FSRObject::id_to_mut_obj(obj_id) {
                    Some(obj) => obj,
                    None => continue,
                };
                obj.color = TripleColor::Black;
                let references = obj.get_references();
                
                
                self.black.push(Some(obj_id));
                
                for ref_idx in references {
                    self.gray.push(Some(ref_idx));
                }
            }
        }
    }


    pub fn reset(&mut self) {
        debug_assert!(self.gray.is_empty(), "Gray set should be empty after marking");
    }

    pub fn white_objects(&mut self) {
        self.memory.process_objects(|x| {
            x.color = TripleColor::White;
        });
    }


    pub fn collect(&mut self, root_indices: &[usize]) {
        self.mark();
        self.reset();
    }
}