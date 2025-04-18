use std::{
    borrow::Cow,
    collections::hash_map::Keys,
    fmt::Debug,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
};

use crate::{
    backend::{
        compiler::bytecode::BinaryOffset,
        memory::{size_alloc::FSRObjectAllocator, GarbageCollector},
        types::fn_def::FSRnE,
        vm::{
            thread::FSRThreadRuntime,
            virtual_machine::{FSRVM, OBJECTS},
        },
    },
    utils::error::{FSRErrCode, FSRError},
};

use super::{
    any::AnyType,
    class::FSRClass,
    class_inst::FSRClassInst,
    code::FSRCode,
    fn_def::FSRFn,
    iterator::FSRInnerIterator,
    list::FSRList,
    module::FSRModule,
    range::FSRRange,
    string::{FSRInnerString, FSRString},
};

pub type ObjId = usize;
pub type AtomicObjId = AtomicUsize;
pub enum FSRGlobalObjId {
    None = 0,
    True = 1,
    False = 2,
    IntegerCls = 3,
    FnCls = 4,
    InnerIterator = 5,
    ListCls = 6,
    StringCls = 7,
    ClassCls = 8,
    CodeCls = 9,
    BoolCls = 10,
    FloatCls = 11,
    Exception = 12,
    RangeCls = 13,
    ModuleCls = 14,
    ThreadCls = 15,
    HashMapCls = 16
}


#[derive(Debug)]
pub enum FSRValue<'a> {
    Integer(i64),
    Float(f64),
    String(Arc<FSRInnerString>),
    Class(Box<FSRClass<'a>>),
    ClassInst(Box<FSRClassInst<'a>>),
    Function(Box<FSRFn<'a>>),
    Bool(bool),
    List(Box<FSRList>),
    Iterator(Box<FSRInnerIterator>),
    Code(Box<FSRCode<'a>>),
    Range(Box<FSRRange>),
    Any(Box<AnyType>),
    // module is define in single file
    Module(Box<FSRModule<'a>>),
    None,
}

impl<'a> FSRValue<'a> {
    fn get_references(&self) -> Vec<ObjId> {
        match self {
            FSRValue::Class(fsrclass) => fsrclass
                .iter_values()
                .map(|x| x.load(Ordering::Relaxed))
                .collect(),
            FSRValue::ClassInst(fsrclass_inst) => fsrclass_inst
                .iter_values()
                .map(|x| x.load(Ordering::Relaxed))
                .collect(),
            FSRValue::List(fsrlist) => fsrlist
                .iter_values()
                .map(|x| x.load(Ordering::Relaxed))
                .collect(),
            FSRValue::Function(f) => f.get_references(),
            FSRValue::Iterator(iterator) => iterator.get_references(),
            _ => vec![],
        }
    }

    pub fn get_size(&self) -> usize {
        match self {
            FSRValue::Class(_) => std::mem::size_of::<FSRClass>(),
            FSRValue::ClassInst(_) => std::mem::size_of::<FSRClassInst>(),
            FSRValue::List(_) => std::mem::size_of::<FSRList>(),
            FSRValue::Function(_) => std::mem::size_of::<FSRFn>(),
            FSRValue::Iterator(_) => std::mem::size_of::<FSRInnerIterator>(),
            FSRValue::Integer(_) => std::mem::size_of::<i64>(),
            FSRValue::Float(_) => std::mem::size_of::<f64>(),
            FSRValue::String(s) => std::mem::size_of::<FSRInnerString>() + s.len(),
            FSRValue::Code(_) => std::mem::size_of::<FSRCode>(),
            FSRValue::Range(_) => std::mem::size_of::<FSRRange>(),
            FSRValue::Module(_) => std::mem::size_of::<FSRModule>(),
            FSRValue::Bool(_) => std::mem::size_of::<bool>(),
            FSRValue::Any(_) => std::mem::size_of::<AnyType>(),
            FSRValue::None => std::mem::size_of::<()>(),
        }
    }
}

#[derive(Debug)]
pub enum FSRRetValue<'a> {
    // Value(Box<FSRObject<'a>>),
    GlobalId(ObjId),
    Reference(&'a AtomicObjId),
}

impl<'a> FSRRetValue<'a> {
    pub fn get_id(&self) -> ObjId {
        match self {
            FSRRetValue::GlobalId(id) => *id,
            FSRRetValue::Reference(id) => id.load(Ordering::Relaxed),
        }
    }
}

impl<'a> FSRValue<'a> {
    fn inst_to_string(
        inst: &FSRClassInst,
        self_id: ObjId,
        thread: &mut FSRThreadRuntime<'a>,
        module: ObjId,
    ) -> Option<Arc<FSRInnerString>> {
        let _ = inst;
        let cls = FSRObject::id_to_obj(self_id).cls;
        let cls = FSRObject::id_to_obj(cls);
        let cls = cls.as_class();

        let v = cls.get_attr("__str__");
        if let Some(obj_id) = v {
            let obj_id = obj_id.load(Ordering::Relaxed);
            let obj = FSRObject::id_to_obj(obj_id);
            let ret = obj.call(&[self_id], thread, module, obj_id);
            let ret_value = match ret {
                Ok(o) => o,
                Err(_) => {
                    return None;
                }
            };

            // if let FSRRetValue::Value(v) = ret_value {
            //     return Some(Box::new(Cow::Owned(v.as_string().to_string())));
            // }

            let id = ret_value.get_id();
            let obj = FSRObject::id_to_obj(id);
            if let FSRValue::String(s) = &obj.value {
                return Some(s.clone());
            }
        }
        None
    }

    fn to_string(
        &self,
        self_id: ObjId,
        thread: &mut FSRThreadRuntime<'a>,
        module: ObjId,
    ) -> Option<Arc<FSRInnerString>> {
        let s = match self {
            FSRValue::Integer(e) => Some(Arc::new(FSRInnerString::new(e.to_string()))),
            FSRValue::Float(e) => Some(Arc::new(FSRInnerString::new(e.to_string()))),
            FSRValue::String(e) => Some(e.clone()),
            FSRValue::Class(_) => None,
            FSRValue::ClassInst(inst) => Self::inst_to_string(inst, self_id, thread, module),
            FSRValue::Function(_) => None,
            FSRValue::None => Some(Arc::new(FSRInnerString::new("None"))),
            FSRValue::Bool(e) => Some(Arc::new(FSRInnerString::new(e.to_string()))),
            FSRValue::List(_) => {
                let res = FSRObject::invoke_method("__str__", &[self_id], thread, module).unwrap();
                match &res {
                    FSRRetValue::GlobalId(id) => {
                        let obj = FSRObject::id_to_obj(*id);
                        if let FSRValue::String(s) = &obj.value {
                            return Some(s.clone());
                        }

                        return None;
                    }
                    FSRRetValue::Reference(atomic_usize) => {
                        let id = atomic_usize.load(Ordering::Relaxed);
                        let obj = FSRObject::id_to_obj(id);
                        if let FSRValue::String(s) = &obj.value {
                            return Some(s.clone());
                        }

                        return None;
                    }
                }
            }
            FSRValue::Iterator(_) => None,
            FSRValue::Code(fsrmodule) => Some(Arc::new(FSRInnerString::new(fsrmodule.as_string()))),
            FSRValue::Range(fsrrange) => Some(Arc::new(FSRInnerString::new(format!(
                "Range({}..{})",
                fsrrange.range.start, fsrrange.range.end
            )))),
            FSRValue::Module(fsrmodule) => {
                Some(Arc::new(FSRInnerString::new(fsrmodule.as_string())))
            }
            FSRValue::Any(_) => Some(Arc::new(FSRInnerString::new("AnyType"))),
        };

        s
    }
}

impl Drop for FSRValue<'_> {
    fn drop(&mut self) {}
}

#[repr(u8)]
pub enum MemType {
    ThreadLocate = 0,
}

#[derive(Debug, PartialEq)]
pub enum Area {
    Minjor,
    Marjor,
    Global
}

impl Area {
    pub fn is_long(&self) -> bool {
        match self {
            Area::Minjor => false,
            Area::Marjor => true,
            Area::Global => true,
        }
    }
}

pub struct FSRObject<'a> {
    pub(crate) value: FSRValue<'a>,
    pub(crate) cls: ObjId,
    pub(crate) free: bool,
    pub(crate) mark: bool,
    pub(crate) gc_count: u32,
    pub(crate) area: Area,
    pub(crate) write_barrier: AtomicBool,
    // pub(crate) garbage_id: u32,
}

impl Debug for FSRObject<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cls = self.cls;
        let obj = FSRObject::id_to_obj(cls);
        let cls = match &obj.value {
            FSRValue::Class(c) => c,
            FSRValue::None => {
                return f
                    .debug_struct("FSRObject")
                    .field("value", &self.value)
                    .field("cls", &"None".to_string())
                    .finish();
            }
            _ => panic!("not valid cls"),
        };
        f.debug_struct("FSRObject")
            .field("value", &self.value)
            .field("cls", &cls.name.to_string())
            .field("area", &self.area)
            .field("write_barrier", &self.write_barrier.load(Ordering::Relaxed))
            .finish()
    }
}

#[cfg(feature = "alloc_trace")]
pub struct HeapTrace {
    total_object: AtomicI64,
}

#[cfg(feature = "alloc_trace")]
impl HeapTrace {
    pub fn add_object(&self) {
        self.total_object.fetch_add(1, Ordering::AcqRel);
    }

    pub fn dec_object(&self) {
        self.total_object.fetch_sub(1, Ordering::AcqRel);
    }

    pub fn object_count(&self) -> i64 {
        self.total_object.load(Ordering::Relaxed)
    }
}

#[cfg(feature = "alloc_trace")]
pub(crate) static HEAP_TRACE: HeapTrace = HeapTrace {
    total_object: AtomicI64::new(0),
};

impl Default for FSRObject<'_> {
    fn default() -> Self {
        Self::new()
    }
}

pub trait FSRObjectCmp {
    fn cmp(&self, other: &FSRObject, thread: &mut FSRThreadRuntime) -> std::cmp::Ordering;
}

impl<'a> FSRObject<'a> {
    pub fn new_inst(value: FSRValue<'a>, cls: ObjId) -> FSRObject<'a> {
        FSRObject {
            value,
            cls,
            // garbage_id: 0,
            // garbage_collector_id: 0,
            free: false,
            mark: false,
            area: Area::Global,
            write_barrier: AtomicBool::new(false),
            gc_count: 0,
        }
    }

    pub fn get_size(&self) -> usize {
        std::mem::size_of::<Self>() + self.value.get_size()
    }

    pub fn as_list(&self) -> &FSRList {
        match &self.value {
            FSRValue::List(fsrlist) => fsrlist,
            _ => unimplemented!(),
        }
    }

    pub fn set_write_barrier(&self, value: bool) {
        self.write_barrier.store(value, Ordering::Relaxed);
    }

    pub fn get_write_barrier(&self) -> bool {
        self.write_barrier.load(Ordering::Relaxed)
    }

    pub fn as_mut_list(&mut self) -> &mut FSRList {
        match &mut self.value {
            FSRValue::List(fsrlist) => fsrlist,
            _ => unimplemented!(),
        }
    }

    pub fn as_code(&self) -> &FSRCode<'a> {
        match &self.value {
            FSRValue::Code(fsrmodule) => fsrmodule,
            _ => unimplemented!(),
        }
    }

    pub fn as_mut_code(&mut self) -> &mut FSRCode<'a> {
        match &mut self.value {
            FSRValue::Code(fsrmodule) => fsrmodule,
            _ => unimplemented!(),
        }
    }

    pub fn as_module(&self) -> &FSRModule<'a> {
        match &self.value {
            FSRValue::Module(fsrmodule) => fsrmodule,
            _ => unimplemented!(),
        }
    }

    #[inline(always)]
    pub fn is_code(&self) -> bool {
        // matches!(&self.value, FSRValue::Code(_fsrmodule))
        if let FSRValue::Code(_) = &self.value {
            return true;
        }
        false
    }

    // pub fn get_garbage_id(&self) -> u32 {
    //     self.garbage_id
    // }

    pub fn new() -> FSRObject<'a> {
        FSRObject {
            value: FSRValue::None,
            cls: 0,
            // garbage_id: 0,
            // garbage_collector_id: 0,
            free: false,
            mark: false,
            area: Area::Global,
            write_barrier: AtomicBool::new(false),
            gc_count: 0,
        }
    }

    #[inline(always)]
    pub fn is_marked(&self) -> bool {
        self.mark
    }

    #[inline(always)]
    pub fn mark(&mut self) {
        self.mark = true
    }

    pub fn is_true_id(&self) -> ObjId {
        if let FSRValue::None = self.value {
            return 2;
        }

        if let FSRValue::Bool(b) = self.value {
            if b {
                return 1;
            } else {
                return 2;
            }
        }

        1
    }

    #[inline(always)]
    pub fn set_value(&mut self, value: FSRValue<'a>) {
        self.value = value;
    }

    #[inline(always)]
    pub fn set_cls(&mut self, cls: ObjId) {
        self.cls = cls
    }

    pub fn as_string(&self) -> &str {
        if let FSRValue::String(s) = &self.value {
            return s.as_str();
        }
        unimplemented!()
    }

    pub fn set_attr(&mut self, name: &'a str, obj_id: ObjId) {
        if let FSRValue::ClassInst(inst) = &mut self.value {
            inst.set_attr(name, obj_id);
            return;
        }

        unimplemented!()
    }

    pub fn as_class(&self) -> &FSRClass {
        if let FSRValue::Class(cls) = &self.value {
            return cls;
        }

        panic!("Not a Cls object")
    }

    #[inline]
    pub fn obj_to_id(obj: &FSRObject<'a>) -> ObjId {
        obj as *const Self as ObjId
    }

    pub fn get_cls_attr(&self, name: &str) -> Option<&'a AtomicObjId> {
        if let Some(btype) = FSRVM::get_base_cls(self.cls) {
            return btype.get_attr(name);
        }

        let cls_obj = FSRObject::id_to_obj(self.cls);
        if let FSRValue::Class(cls) = &cls_obj.value {
            return cls.get_attr(name);
        }

        None
    }

    /*
    Try get offset of cls, if not get alias name
    like BinaryOffset::Add if not -> __add__
     */
    #[inline]
    pub fn get_cls_offset_attr(&self, offset: BinaryOffset) -> Option<&'a AtomicObjId> {
        let cls_obj = FSRObject::id_to_obj(self.cls);
        if let FSRValue::Class(cls) = &cls_obj.value {
            return cls.try_get_offset_attr(offset);
        }

        None
    }

    pub fn is_false(&self) -> bool {
        if let FSRValue::Bool(b) = &self.value {
            return !b;
        }

        false
    }

    pub fn as_fn(&self) -> &FSRFn {
        if let FSRValue::Function(f) = &self.value {
            return f;
        }

        unimplemented!()
    }

    pub fn as_mut_fn(&mut self) -> &mut FSRFn<'a> {
        if let FSRValue::Function(f) = &mut self.value {
            return f;
        }

        unimplemented!()
    }

    #[inline(always)]
    pub fn is_sp_object(id: ObjId) -> bool {
        id < 1000
    }

    // #[inline]
    // pub fn ref_add(&self) {
    //     self.ref_count.fetch_add(1, Ordering::Relaxed);
    // }

    // #[inline]
    // pub fn ref_dec(&self) {
    //     self.ref_count.fetch_sub(1, Ordering::Relaxed);
    // }

    pub fn into_object(id: ObjId) -> Box<FSRObject<'a>> {
        unsafe { Box::from_raw(id as *mut Self) }
    }

    // #[inline(always)]
    // pub fn count_ref(&self) -> u32 {
    //     unsafe { *self.ref_count.as_ptr() }
    // }

    #[inline(always)]
    pub fn id_to_obj(id: ObjId) -> &'a FSRObject<'a> {
        if id >= 1000 {
            return unsafe { &*(id as *const FSRObject) };
        }

        unsafe {
            if let Some(obj) = OBJECTS.get(id) {
                return obj;
            }

            panic!("Invalid special object ID: {}", id);
        }
    }

    #[inline(always)]
    pub fn id_to_mut_obj(id: ObjId) -> Option<&'a mut FSRObject<'a>> {
        if id < 1000 {
            return None
        }
        unsafe {
            let ptr = id as *mut FSRObject;
            Some(&mut *ptr)
        }
    }

    pub fn invoke_method(
        name: &str,
        args: &[ObjId],
        thread: &mut FSRThreadRuntime<'a>,
        module: ObjId,
    ) -> Result<FSRRetValue<'a>, FSRError> {
        let self_object = Self::id_to_obj(args[0]);
        let self_method = match self_object.get_cls_attr(name) {
            Some(s) => s.load(Ordering::Relaxed),
            None => {
                return Err(FSRError::new(
                    format!("no such a method `{}`", name),
                    FSRErrCode::NoSuchMethod,
                ))
            }
        };
        let method_object = Self::id_to_obj(self_method);
        let v = method_object.call(args, thread, module, self_method)?;
        Ok(v)
    }

    #[inline(always)]
    pub fn invoke_binary_method(
        offset: BinaryOffset,
        left: ObjId,
        right: ObjId,
        thread: &mut FSRThreadRuntime<'a>,
        module: ObjId,
    ) -> Result<FSRRetValue<'a>, FSRError> {
        let left_object = Self::id_to_obj(left);

        if let Some(left_method) = left_object.get_cls_offset_attr(offset) {
            let left_method = left_method.load(Ordering::Relaxed);
            let method_object = Self::id_to_obj(left_method).as_fn();
            let v = method_object.invoke_binary(left, right, thread, module, left_method)?;
            return Ok(v);
        }

        let left_method = match left_object.get_cls_attr(offset.alias_name()) {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    format!("no such a method `{}`", offset.alias_name()),
                    FSRErrCode::NoSuchMethod,
                ))
            }
        };
        let left_method = left_method.load(Ordering::Relaxed);

        let method_object = Self::id_to_obj(left_method).as_fn();
        let v = method_object.invoke(&[left, right], thread, module, left_method)?;
        Ok(v)
    }

    #[inline]
    pub fn invoke_offset_method(
        offset: BinaryOffset,
        args: &[ObjId],
        thread: &mut FSRThreadRuntime<'a>,
        module: ObjId,
    ) -> Result<FSRRetValue<'a>, FSRError> {
        let self_object = Self::id_to_obj(args[0]);

        if let Some(self_method) = self_object.get_cls_offset_attr(offset) {
            let self_method = self_method.load(Ordering::Relaxed);
            let method_object = Self::id_to_obj(self_method);
            let v = method_object.call(args, thread, module, self_method)?;
            return Ok(v);
        }

        let self_method = match self_object.get_cls_attr(offset.alias_name()) {
            Some(s) => s,
            None => {
                return Err(FSRError::new(
                    format!("no such a method `{}`", offset.alias_name()),
                    FSRErrCode::NoSuchMethod,
                ))
            }
        };
        let self_method = self_method.load(Ordering::Relaxed);
        let method_object = Self::id_to_obj(self_method);
        let v = method_object.call(args, thread, module, self_method)?;
        Ok(v)
    }

    #[inline]
    pub fn get_attr(&self, name: &str) -> Option<&AtomicObjId> {
        if let Some(s) = self.get_cls_attr(name) {
            return Some(s);
        }

        if let FSRValue::ClassInst(inst) = &self.value {
            let v = match inst.get_attr(name) {
                Some(s) => s,
                None => {
                    return None;
                }
            };
            return Some(v);
        }

        if let FSRValue::Code(m) = &self.value {
            return m.get_object(name);
        }

        None
    }

    pub fn list_attrs(&self) -> Keys<&'a str, AtomicObjId> {
        if let FSRValue::ClassInst(inst) = &self.value {
            return inst.list_attrs();
        }

        unimplemented!()
    }

    #[inline(always)]
    pub fn call(
        &'a self,
        args: &[ObjId],
        thread: &mut FSRThreadRuntime<'a>,
        module: ObjId,
        fn_id: ObjId,
    ) -> Result<FSRRetValue<'a>, FSRError> {
        if let FSRValue::Function(fn_def) = &self.value {
            return fn_def.invoke(args, thread, module, fn_id);
        }
        unimplemented!()
    }

    pub fn as_fn_mut(&mut self) -> &FSRFn<'a> {
        if let FSRValue::Function(f) = &self.value {
            return f;
        }

        unimplemented!()
    }

    pub fn get_self_id(&self) -> u64 {
        self as *const Self as u64
    }

    pub fn to_string(
        &'a self,
        thread: &mut FSRThreadRuntime<'a>,
        module: ObjId,
    ) -> FSRValue<'a> {
        let s = self
            .value
            .to_string(FSRObject::obj_to_id(self), thread, module);
        if let Some(s) = s {
            return FSRString::new_inst_with_inner(s);
        }
        let v = FSRObject::id_to_obj(self.cls);
        if let FSRValue::Class(c) = &v.value {
            return FSRString::new_value(&format!(
                "<`{}` Class Object at {:?}>",
                self.cls, self as *const Self
            ))
        }
        // Box::new(FSRString::new_inst(&format!(
        //     "<`{}` Object at {:?}>",
        //     self.cls, self as *const Self
        // )))

        return FSRString::new_value(&format!(
            "<`{}` Object at {:?}>",
            self.cls, self as *const Self
        ))
        //return self.invoke("__str__", vec![]);
    }

    #[inline]
    pub fn is_fsr_function(&self) -> bool {
        let FSRValue::Function(fn_def) = &self.value else {
            return false;
        };

        // 检查函数类型
        matches!(fn_def.get_def(), FSRnE::FSRFn(_))
    }

    pub fn is_fsr_cls(&self) -> bool {
        if let FSRValue::Class(_) = &self.value {
            return true;
        }

        false
    }

    pub fn get_fsr_offset(&self) -> (&Cow<str>, (usize, usize)) {
        if let FSRValue::Function(fn_def) = &self.value {
            if let FSRnE::FSRFn(f) = &fn_def.get_def() {
                return (f.get_name(), f.get_ip());
            }
        }

        panic!()
    }

    pub fn get_fsr_args(&self) -> &Vec<String> {
        if let FSRValue::Function(fn_def) = &self.value {
            return fn_def.get_args();
        }

        unimplemented!()
    }

    pub fn get_fsr_class_name(&self) -> &str {
        if let FSRValue::Class(cls) = &self.value {
            return cls.get_name();
        }

        unimplemented!()
    }

    #[inline(always)]
    pub fn none_id() -> ObjId {
        0
    }

    #[inline(always)]
    pub fn true_id() -> ObjId {
        1
    }

    #[inline(always)]
    pub fn false_id() -> ObjId {
        2
    }

    pub fn iter_object(&self) -> impl Iterator<Item = &AtomicObjId> {
        match &self.value {
            FSRValue::ClassInst(inst) => inst.iter_values(),
            _ => unimplemented!(),
        }
    }

    pub fn get_references(&self) -> Box<dyn Iterator<Item = ObjId> + '_> {
        match &self.value {
            FSRValue::Class(fsrclass) => Box::new(fsrclass
                .iter_values()
                .map(|x| x.load(Ordering::Relaxed))),
            FSRValue::ClassInst(fsrclass_inst) => Box::new(fsrclass_inst
                .iter_values()
                .map(|x| x.load(Ordering::Relaxed))),
            FSRValue::List(fsrlist) => Box::new(fsrlist
                .iter_values()
                .map(|x| x.load(Ordering::Relaxed))),
            FSRValue::Function(f) => Box::new(f.get_references().into_iter()),
            FSRValue::Iterator(iterator) => Box::new(iterator.get_references().into_iter()),
            FSRValue::Any(any) => Box::new(any.iter_values().map(|x| x.load(Ordering::Relaxed))),
            _ => Box::new(std::iter::empty()),
        }
        //Box::new(self.value.get_references().into_iter())
    }

}

mod test {

    #[test]
    fn test_size_of_object() {
        use crate::backend::types::base::FSRObject;
        use crate::backend::types::base::FSRValue;

        use std::mem::size_of;
        println!("Size of FSRObject: {}", size_of::<FSRObject>());

        println!("Size of FSRValue: {}", size_of::<FSRValue>());

        println!(
            "Size of FSRInnerIterator: {}",
            size_of::<crate::backend::types::iterator::FSRInnerIterator>()
        );

        println!(
            "Size of FSRList: {}",
            size_of::<crate::backend::types::list::FSRList>()
        );
    }
}

pub trait DropObject<'a> {
    fn drop(&self, allocator: &mut FSRObjectAllocator<'a>);
}
