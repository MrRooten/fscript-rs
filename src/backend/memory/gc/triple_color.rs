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

    // 将对象添加到白色集合（未标记对象）
    pub fn add_object(&mut self, obj: ObjId) {
        self.white.push(Some(obj));
    }

    // 标记根对象（将其从白色移到灰色）
    pub fn mark_roots(&mut self, root_indices: &[usize]) {
        for &index in root_indices {
            if index < self.white.len() {
                if let Some(obj) = self.white[index].take() {
                    self.gray.push(Some(obj));
                }
            }
        }
    }

    // 标记阶段：处理灰色对象，将其引用的对象也移到灰色，然后将自己移到黑色
    pub fn mark(&mut self) {
        while !self.gray.is_empty() {
            if let Some(obj_id) = self.gray.pop().unwrap() {
                let obj = FSRObject::id_to_obj(obj_id);
                // 获取对象引用的其他对象
                let references = obj.get_references();
                
                // 将此对象移到黑色集合
                self.black.push(Some(obj_id));
                
                // 将所有引用的对象从白色移到灰色
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

    // 清除阶段：清除白色对象（未被标记的对象）
    pub fn sweep(&mut self) {
        // 白色集合中的对象是垃圾，可以直接清除
        self.white.clear();
    }

    // 重置状态以准备下一次GC循环
    pub fn reset(&mut self) {
        // 将黑色对象移回白色集合
        self.white.append(&mut self.black);
        // 确保灰色集合为空
        debug_assert!(self.gray.is_empty(), "Gray set should be empty after marking");
    }

    // 完整的垃圾回收过程
    pub fn collect(&mut self, root_indices: &[usize]) {
        self.mark_roots(root_indices);
        self.mark();
        self.sweep();
        self.reset();
    }
}