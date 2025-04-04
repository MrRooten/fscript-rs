use std::any::Any;

use crate::{backend::{types::base::FSRObject, vm::thread::FSRThreadRuntime}, utils::error::FSRError};

use super::{base::{FSRRetValue, FSRValue, ObjId}, class::FSRClass, fn_def::FSRFn};

#[derive(Debug)]
pub struct AnyType {
    pub value: Box<dyn Any + Send>,
}

#[derive(Debug)]
pub struct FSRThreadHandle {
    pub thread: Option<std::thread::JoinHandle<()>>,
}


fn join<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId
) -> Result<FSRRetValue, FSRError> {
    let self_object = FSRObject::id_to_mut_obj(args[0]);
    
    if let FSRValue::Any(any) = &mut self_object.value {
        if let Some(handle) = any.value.downcast_mut::<FSRThreadHandle>() {
            handle.join();

        } else {
            unimplemented!()
        }
    } else {
        unimplemented!()
    }
    Ok(FSRRetValue::GlobalId(FSRObject::none_id()))
}

fn is_finish<'a>(
    args: &[ObjId],
    thread: &mut FSRThreadRuntime<'a>,
    module: ObjId
) -> Result<FSRRetValue, FSRError> {
    let self_object = FSRObject::id_to_mut_obj(args[0]);
    
    let done = if let FSRValue::Any(any) = &mut self_object.value {
        if let Some(handle) = any.value.downcast_mut::<FSRThreadHandle>() {
            handle.thread.is_none()

        } else {
            unimplemented!()
        }
    } else {
        unimplemented!()
    };
    
    if done {
        Ok(FSRRetValue::GlobalId(FSRObject::true_id()))
    } else {
        Ok(FSRRetValue::GlobalId(FSRObject::false_id()))
    }
}

impl FSRThreadHandle {
    pub fn new(thread: std::thread::JoinHandle<()>) -> Self {
        FSRThreadHandle { thread: Some(thread) }
    }

    pub fn join(&mut self) {
        if let Some(handle) = self.thread.take() {
            handle.join().unwrap();
        } else {
            unimplemented!()
        }


    }

    pub fn to_any_type(self) -> FSRValue<'static> {
        FSRValue::Any(Box::new(AnyType {
            value: Box::new(self),
        }))
    }

    pub fn thread_cls() -> FSRClass<'static> {
        let mut cls = FSRClass::new("Thread");
        let thread_join_fn = FSRFn::from_rust_fn_static(join, "__thread_join");
        cls.insert_attr("join", thread_join_fn);
        let thread_finish_fn = FSRFn::from_rust_fn_static(join, "__thread_finish");
        cls.insert_attr("is_finish", thread_finish_fn);
        cls
    }
}