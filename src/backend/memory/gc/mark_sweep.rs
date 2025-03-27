use std::sync::atomic::Ordering;
use std::collections::HashSet;

use crate::backend::{
    memory::{size_alloc::FSRObjectAllocator, FSRAllocator, GarbageCollector}, 
    types::base::{FSRObject, FSRValue, ObjId}
};

pub struct MarkSweepGarbageCollector<'a> {
    // 存储所有管理的对象
    objects: Vec<Option<Box<FSRObject<'a>>>>,
    // 空闲列表 - 维护可重用的对象槽位
    free_slots: Vec<usize>,
    // 根对象集合
    roots: HashSet<ObjId>,
    // 对象分配器
    allocator: FSRObjectAllocator<'a>,
    // 标记位图
    marks: Vec<bool>,
}

impl<'a> MarkSweepGarbageCollector<'a> {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            free_slots: Vec::new(),
            roots: HashSet::new(),
            allocator: FSRObjectAllocator::new(),
            marks: Vec::new(),
        }
    }
    
    // 添加根对象
    pub fn add_root(&mut self, id: ObjId) {
        self.roots.insert(id);
    }
    
    // 移除根对象
    pub fn remove_root(&mut self, id: ObjId) {
        self.roots.remove(&id);
    }
    
    // 通过ObjId获取对象的garbage_id（数组下标）
    fn get_garbage_id(&self, id: ObjId) -> Option<usize> {
        // 尝试通过对象访问，获取garbage_id
        let obj = unsafe { FSRObject::id_to_obj(id) };
        let garbage_id = obj.garbage_id.load(Ordering::Relaxed) as usize;
        
        // 验证garbage_id是否有效
        if garbage_id < self.objects.len() {
            if let Some(Some(stored_obj)) = self.objects.get(garbage_id) {
                if FSRObject::obj_to_id(stored_obj) == id {
                    return Some(garbage_id);
                }
            }
        }
        
        // 如果缓存的garbage_id无效，则遍历查找
        for (i, obj_slot) in self.objects.iter().enumerate() {
            if let Some(obj) = obj_slot {
                if FSRObject::obj_to_id(obj) == id {
                    // 找到了对象，更新garbage_id缓存
                    let obj_ref = unsafe { FSRObject::id_to_obj(id) };
                    obj_ref.garbage_id.store(i as u32, Ordering::Relaxed);
                    return Some(i);
                }
            }
        }
        
        None
    }
    
    // 通过ObjId获取对象引用
    fn get_object(&self, id: ObjId) -> Option<&Box<FSRObject<'a>>> {
        self.get_garbage_id(id).and_then(|idx| {
            self.objects.get(idx).and_then(|slot| slot.as_ref())
        })
    }
    
    // 通过ObjId获取可变对象引用
    fn get_object_mut(&mut self, id: ObjId) -> Option<&mut Box<FSRObject<'a>>> {
        if let Some(idx) = self.get_garbage_id(id) {
            if idx < self.objects.len() {
                return self.objects.get_mut(idx).and_then(|slot| slot.as_mut());
            }
        }
        None
    }
    
    // 标记对象
    fn mark(&mut self, id: ObjId) {
        if let Some(idx) = self.get_garbage_id(id) {
            if idx >= self.marks.len() {
                self.marks.resize(self.objects.len(), false);
            }
            self.marks[idx] = true;
        }
    }
    
    // 检查对象是否被标记
    fn is_marked(&self, id: ObjId) -> bool {
        self.get_garbage_id(id)
            .map(|idx| idx < self.marks.len() && self.marks[idx])
            .unwrap_or(false)
    }
    
    // 清除所有标记
    fn clear_marks(&mut self) {
        self.marks.iter_mut().for_each(|m| *m = false);
    }
}

impl<'a> GarbageCollector<'a> for MarkSweepGarbageCollector<'a> {
    fn new_object(&mut self, cls: ObjId, value: FSRValue<'a>) -> Option<ObjId> {
        // 创建新对象
        let mut obj = self.allocator.allocate(value, cls);
        
        // 分配一个存储槽位
        let slot_idx = if let Some(free_idx) = self.free_slots.pop() {
            // 重用空闲槽位
            free_idx
        } else {
            // 创建新槽位
            let idx = self.objects.len();
            self.objects.push(None);
            idx
        };
        
        // 设置对象的garbage_id为槽位索引
        obj.garbage_id.store(slot_idx as u32, Ordering::Relaxed);
        
        // 获取对象ID（内存地址）
        let obj_id = FSRObject::obj_to_id(&obj);
        
        // 存储对象
        self.objects[slot_idx] = Some(obj);
        
        // 确保marks数组长度足够
        if self.marks.len() <= slot_idx {
            self.marks.resize(self.objects.len(), false);
        }
        
        Some(obj_id)
    }

    fn free_object(&mut self, id: ObjId) {
        if let Some(idx) = self.get_garbage_id(id) {
            if idx < self.objects.len() {
                if let Some(obj) = self.objects[idx].take() {
                    // 释放对象内存
                    self.allocator.free_object(obj);
                    // 将释放的槽位添加到空闲列表
                    self.free_slots.push(idx);
                }
            }
        }
    }

    fn collect(&mut self) {
        // 清除之前的标记
        self.clear_marks();
        
        // 标记阶段: 从根对象开始标记所有可达对象
        let mut work_list: Vec<ObjId> = self.roots.iter().copied().collect();
        
        while let Some(id) = work_list.pop() {
            // 跳过已标记的对象
            if self.is_marked(id) {
                continue;
            }
            
            // 标记当前对象
            self.mark(id);
            
            // 获取并标记该对象引用的所有对象
            if let Some(obj) = self.get_object(id) {
                // 获取对象所有引用
                let refs = obj.get_references();
                
                // 将未标记的引用添加到工作列表
                for ref_id in refs {
                    if !self.is_marked(ref_id) {
                        work_list.push(ref_id);
                    }
                }
            }
        }
        
        // 清除阶段: 回收所有未标记的对象
        let mut to_free = Vec::new();
        
        for (idx, obj_opt) in self.objects.iter().enumerate() {
            if let Some(obj) = obj_opt {
                let id = FSRObject::obj_to_id(obj);
                
                // 检查对象是否被标记（可达）
                if idx >= self.marks.len() || !self.marks[idx] {
                    to_free.push(id);
                }
            }
        }
        
        // 释放所有未标记的对象
        for id in to_free {
            self.free_object(id);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::backend::{types::{base::FSRGlobalObjId, list::FSRList}, vm::virtual_machine::FSRVM};
    use super::*;
    
    #[test]
    fn test_mark_sweep_gc() {
        // 初始化虚拟机，确保全局对象ID可用
        let _vm = FSRVM::new();
        let mut gc = MarkSweepGarbageCollector::new();
        
        // 用于测试的类ID
        let integer_cls = FSRGlobalObjId::IntegerCls as ObjId;
        let list_cls = FSRGlobalObjId::ListCls as ObjId;
        
        // 创建一些对象
        let int1 = gc.new_object(integer_cls, FSRValue::Integer(10)).unwrap();
        let int2 = gc.new_object(integer_cls, FSRValue::Integer(20)).unwrap();
        let int3 = gc.new_object(integer_cls, FSRValue::Integer(30)).unwrap();
        
        // 创建一个列表，包含对前两个整数的引用
        let mut list_val = vec![];
        list_val.push(int1);
        list_val.push(int2);
        let list = gc.new_object(list_cls, FSRList::new_value(list_val)).unwrap();
        
        // 确认所有对象都被成功创建
        assert!(gc.get_object(int1).is_some());
        assert!(gc.get_object(int2).is_some());
        assert!(gc.get_object(int3).is_some());
        assert!(gc.get_object(list).is_some());
        
        // 测试1: 没有根对象的情况下，所有对象都应该被回收
        gc.collect();
        
        // 所有对象应该被回收
        assert!(gc.get_object(int1).is_none());
        assert!(gc.get_object(int2).is_none());
        assert!(gc.get_object(int3).is_none());
        assert!(gc.get_object(list).is_none());
        
        // 重新创建对象
        let int1 = gc.new_object(integer_cls, FSRValue::Integer(10)).unwrap();
        let int2 = gc.new_object(integer_cls, FSRValue::Integer(20)).unwrap();
        let int3 = gc.new_object(integer_cls, FSRValue::Integer(30)).unwrap();
        
        let mut list_val = vec![];
        list_val.push(int1);
        list_val.push(int2);
        let list = gc.new_object(list_cls, FSRList::new_value(list_val)).unwrap();
        
        // 测试2: 添加list作为根，list和int1、int2应该保留，int3应该被回收
        gc.add_root(list);
        gc.collect();
        
        // list及其引用的对象应该保留
        assert!(gc.get_object(int1).is_some());
        assert!(gc.get_object(int2).is_some());
        assert!(gc.get_object(list).is_some());
        
        // int3 没有被引用，应该被回收
        assert!(gc.get_object(int3).is_none());
        
        // 测试3: 测试从根中移除对象
        gc.remove_root(list);
        gc.collect();
        
        // 所有对象都应该被回收
        assert!(gc.get_object(int1).is_none());
        assert!(gc.get_object(int2).is_none());
        assert!(gc.get_object(list).is_none());
        
        // 测试4: 测试对象槽位重用
        let before_alloc = gc.objects.len();
        let free_count = gc.free_slots.len();
        
        // 创建新对象，应该重用已释放的槽位
        let new_int = gc.new_object(integer_cls, FSRValue::Integer(100)).unwrap();
        
        // 对象应该成功创建
        assert!(gc.get_object(new_int).is_some());
        
        // 应该重用了空闲槽位，没有创建新槽位
        assert_eq!(gc.objects.len(), before_alloc);
        assert_eq!(gc.free_slots.len(), free_count - 1);
    }
}