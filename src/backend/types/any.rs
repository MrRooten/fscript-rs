use std::{any::Any, fmt::Debug};

use crate::{
    backend::{types::base::FSRObject, vm::thread::FSRThreadRuntime},
    utils::error::FSRError,
};

use super::{
    base::{AtomicObjId, FSRRetValue, FSRValue, ObjId},
    class::FSRClass,
    fn_def::FSRFn,
};

// pub trait GetReference {
    
// }

pub trait ExtensionTrait: Send {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn get_reference<'a>(
        &'a self,
        full: bool,
        worklist: &mut Vec<ObjId>,
        is_add: &mut bool,
    ) -> Box<dyn Iterator<Item = ObjId> + 'a>;

    fn set_undirty(&mut self);
}


pub struct FSRExtension {
    pub value: Box<dyn ExtensionTrait>,
}

impl Debug for FSRExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AnyType({:?})", self.value.as_any())
    }
}

impl FSRExtension {
    pub fn iter_values<'a>(
        &'a self,
        full: bool,
        worklist: &mut Vec<ObjId>,
        is_add: &mut bool,
    ) -> Box<dyn Iterator<Item = ObjId> + 'a> {
        self.value.get_reference(full, worklist, is_add)
    }

    pub fn undirty(&mut self) {
        self.value.set_undirty();
    }
}

#[derive(Debug)]
pub struct FSRThreadHandle {
    pub thread: Option<std::thread::JoinHandle<()>>,
}

impl ExtensionTrait for FSRThreadHandle {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_reference<'a>(
        &'a self,
        full: bool,
        _: &mut Vec<ObjId>,
        _: &mut bool,
    ) -> Box<dyn Iterator<Item = ObjId> + 'a> {
        Box::new(std::iter::empty())
    }

    fn set_undirty(&mut self) {}
}


fn join(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId
) -> Result<FSRRetValue, FSRError> {
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let self_object = FSRObject::id_to_mut_obj(args[0]).expect("msg: not a any and hashmap");

    if let FSRValue::Extension(any) = &mut self_object.value {
        if let Some(handle) = any.value.as_any_mut().downcast_mut::<FSRThreadHandle>() {
            //thread.release();
            let _ = handle.join();
            thread.acquire();
        } else {
            unimplemented!()
        }
    } else {
        unimplemented!()
    }
    Ok(FSRRetValue::GlobalId(FSRObject::none_id()))
}

impl FSRThreadHandle {
    pub fn new(thread: std::thread::JoinHandle<()>) -> Self {
        FSRThreadHandle {
            thread: Some(thread),
        }
    }

    pub fn join(&mut self) -> Result<(), FSRError> {
        if let Some(handle) = self.thread.take() {
            handle.join().unwrap();
        } else {
            unimplemented!()
        }

        Ok(())
    }

    pub fn to_any_type(self) -> FSRValue<'static> {
        FSRValue::Extension(Box::new(FSRExtension {
            value: Box::new(self),
        }))
    }

    pub fn thread_cls() -> FSRClass {
        let mut cls = FSRClass::new("Thread");
        let thread_join_fn = FSRFn::from_rust_fn_static(join, "__thread_join");
        cls.insert_attr("join", thread_join_fn);
        let thread_finish_fn = FSRFn::from_rust_fn_static(join, "__thread_finish");
        cls.insert_attr("is_finish", thread_finish_fn);
        cls
    }
}
