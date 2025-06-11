use std::{any::Any, path::PathBuf};

use anyhow::{anyhow, Context};

use crate::{
    backend::{
        types::{
            any::{AnyDebugSend, AnyType, GetReference},
            base::{FSRGlobalObjId, FSRObject, FSRRetValue, FSRValue, ObjId}, class::FSRClass, fn_def::FSRFn,
        },
        vm::{thread::FSRThreadRuntime, virtual_machine::get_object_by_global_id},
    },
    utils::error::{FSRErrCode, FSRError},
};

#[derive(Debug)]
pub struct FSRInnerFile {
    pub file: std::fs::File,
    pub path: PathBuf,
}

impl GetReference for FSRInnerFile {
    fn get_reference<'a>(
        &'a self,
        full: bool,
        worklist: &mut Vec<crate::backend::types::base::ObjId>,
        is_add: &mut bool,
    ) -> Box<dyn Iterator<Item = crate::backend::types::base::ObjId> + 'a> {
        Box::new(std::iter::empty())
    }

    fn set_undirty(&mut self) {}
}

impl AnyDebugSend for FSRInnerFile {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl FSRInnerFile {
    pub fn new(path: &str) -> Result<Self, FSRError> {
        let path_buf = PathBuf::from(path);
        let file = std::fs::File::open(&path_buf)?;
        Ok(FSRInnerFile {
            file,
            path: path_buf,
        })
    }

    pub fn get_path(&self) -> &str {
        self.path.to_str().unwrap_or("")
    }

    pub fn seek(&mut self, offset: usize) -> Result<(), FSRError> {
        use std::io::Seek;
        self.file
            .seek(std::io::SeekFrom::Start(offset as u64))
            .with_context(|| {
                anyhow!(
                    "FSRInnerFile::seek: Failed to seek in file: {}",
                    self.get_path()
                )
            })?;
        Ok(())
    }

    pub fn to_any_type(self) -> FSRValue<'static> {
        FSRValue::Any(Box::new(AnyType {
            value: Box::new(self),
        }))
    }

    pub fn get_class() -> FSRClass<'static> {
        let mut cls = FSRClass::new("File");
        let open = FSRFn::from_rust_fn_static(fsr_fn_open_file, "new");
        cls.insert_attr("open", open);
        cls
    }
}

pub fn fsr_fn_open_file(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId,
) -> Result<FSRRetValue, FSRError> {
    if len < 2 {
        return Err(FSRError::new("fsr_fn_open_file requires at least 2 arguments", FSRErrCode::RuntimeError));
    }
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let file_cls = args[0];
    let file_path = args[1];
    let file_path_obj = FSRObject::id_to_obj(file_path);
    if let FSRValue::String(s) = &file_path_obj.value {
        let inner_file = FSRInnerFile::new(s.as_str())?;
        let object = thread.garbage_collect.new_object(
            inner_file.to_any_type(),
            file_cls,
        );
        return Ok(FSRRetValue::GlobalId(object))
    }

    panic!("Invalid file path argument")
}
