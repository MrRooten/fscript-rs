use std::{any::Any, fs::File, io::{BufRead, BufReader, BufWriter, Lines}, path::PathBuf};

use anyhow::{anyhow, Context};

use crate::{
    backend::{
        types::{
            any::{ExtensionTrait, FSRExtension}, base::{FSRObject, FSRRetValue, FSRValue, GlobalObj, ObjId}, bytes::FSRInnerBytes, class::FSRClass, fn_def::FSRFn, iterator::{FSRInnerIterator, FSRIterator, FSRIteratorReferences}, string::FSRString
        },
        vm::{thread::FSRThreadRuntime, virtual_machine::gid},
    },
    utils::error::{FSRErrCode, FSRError},
};

#[derive(PartialEq, Debug)]
enum OpMode {
    Bytes,
    String
}

#[derive(Debug)]
pub struct FSRInnerFile {
    pub reader: BufReader<File>,
    pub writer: Option<BufWriter<File>>,
    mode: OpMode,
    pub path: PathBuf,
}


impl ExtensionTrait for FSRInnerFile {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

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

impl FSRInnerFile {
    fn get_mode(mode_str: &str) -> OpMode {
        match mode_str {
            "b" => OpMode::Bytes,
            "s" => OpMode::String,
            _ => panic!("Invalid mode for FSRInnerFile: {}", mode_str),
            
        }
    }

    pub fn new(path: &str, mode: &str) -> Result<Self, FSRError> {
        let path_buf = PathBuf::from(path);
        let file = std::fs::File::open(&path_buf)?;
        Ok(FSRInnerFile {
            reader: BufReader::new(file),
            writer: None, // Writer can be initialized later if needed
            mode: Self::get_mode(mode),
            path: path_buf,
        })
    }

    pub fn get_path(&self) -> &str {
        self.path.to_str().unwrap_or("")
    }

    pub fn seek(&mut self, offset: usize) -> Result<(), FSRError> {
        use std::io::Seek;
        self.reader
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
        FSRValue::Extension(Box::new(FSRExtension {
            value: Box::new(self),
        }))
    }

    pub fn get_class() -> FSRClass {
        let mut cls = FSRClass::new("File");
        cls.init_method();
        let open = FSRFn::from_rust_fn_static(fsr_fn_open_file, "new");
        cls.insert_attr("open", open);
        let read_all = FSRFn::from_rust_fn_static(fsr_fn_read_all, "read_all");
        cls.insert_attr("read_all", read_all);
        let file_lines = FSRFn::from_rust_fn_static(fsr_fn_file_lines, "lines");
        cls.insert_attr("lines", file_lines);
        let read = FSRFn::from_rust_fn_static(fsr_fn_read, "read");
        cls.insert_attr("read", read);
        let is_file = FSRFn::from_rust_fn_static(fsr_fn_is_file, "is_file");
        cls.insert_attr("is_file", is_file);
        let is_dir = FSRFn::from_rust_fn_static(fsr_fn_is_dir, "is_dir");
        cls.insert_attr("is_dir", is_dir);
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
        return Err(FSRError::new(
            "fsr_fn_open_file requires at least 2 arguments",
            FSRErrCode::RuntimeError,
        ));
    }
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let file_cls = args[0];
    let file_path = args[1];
    let mode = if let Some(s) = args.get(2) {
        if let FSRValue::String(s) = &FSRObject::id_to_obj(*s).value {
            s.as_str()
        } else {
            return Err(FSRError::new(
                "Invalid mode argument, expected a string",
                FSRErrCode::RuntimeError,
            ));
        }
    } else {
        "s" // Default to string mode if not provided
    };
    let file_path_obj = FSRObject::id_to_obj(file_path);
    if let FSRValue::String(s) = &file_path_obj.value {
        let inner_file = FSRInnerFile::new(s.as_str(), mode)?;
        let object = thread
            .garbage_collect
            .new_object(inner_file.to_any_type(), file_cls);
        return Ok(FSRRetValue::GlobalId(object));
    }

    panic!("Invalid file path argument")
}

// file.read_all() -> Bytes
pub fn fsr_fn_read_all(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId,
) -> Result<FSRRetValue, FSRError> {
    if len < 1 {
        return Err(FSRError::new(
            "fsr_fn_read_all requires at least 1 argument",
            FSRErrCode::RuntimeError,
        ));
    }
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let file_obj_id = args[0];
    let file_obj = FSRObject::id_to_mut_obj(file_obj_id).unwrap();

    if let FSRValue::Extension(any_type) = &mut file_obj.value {
        if let Some(inner_file) = any_type.value.as_any_mut().downcast_mut::<FSRInnerFile>() {
            use std::io::Read;
            if inner_file.mode == OpMode::Bytes {
                let mut content = Vec::new();
                inner_file
                    .reader
                    .read_to_end(&mut content)
                    .with_context(|| anyhow!("Failed to read from file: {}", inner_file.get_path()))?;
                let ret = FSRValue::Bytes(Box::new(FSRInnerBytes::new(content)));
                let ret = thread
                    .garbage_collect
                    .new_object(ret, gid(GlobalObj::BytesCls));
                return Ok(FSRRetValue::GlobalId(ret));
            }
            let mut content = String::new();
            inner_file
                .reader
                .read_to_string(&mut content)
                .with_context(|| anyhow!("Failed to read from file: {}", inner_file.get_path()))?;
            let ret = FSRString::new_value(content);
            let ret = thread
                .garbage_collect
                .new_object(ret, gid(GlobalObj::StringCls));
            return Ok(FSRRetValue::GlobalId(ret));
        }
    }

    Err(FSRError::new(
        "Invalid file object",
        FSRErrCode::RuntimeError,
    ))
}

pub fn fsr_fn_is_file(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId,
) -> Result<FSRRetValue, FSRError> {
    if len < 1 {
        return Err(FSRError::new(
            "fsr_fn_is_file requires at least 1 argument",
            FSRErrCode::RuntimeError,
        ));
    }
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let file_path = args[0];
    let file_path_obj = FSRObject::id_to_obj(file_path);
    if let FSRValue::String(s) = &file_path_obj.value {
        let path = PathBuf::from(s.as_str());
        if path.is_file() {
            return Ok(FSRRetValue::GlobalId(FSRObject::true_id()));
        }
        return Ok(FSRRetValue::GlobalId(FSRObject::false_id()));
    }

    Err(FSRError::new(
        "Invalid file path argument",
        FSRErrCode::RuntimeError,
    ))
}

pub fn fsr_fn_is_dir(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId,
) -> Result<FSRRetValue, FSRError> {
    if len < 1 {
        return Err(FSRError::new(
            "fsr_fn_is_dir requires at least 1 argument",
            FSRErrCode::RuntimeError,
        ));
    }
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let dir_path = args[0];
    let dir_path_obj = FSRObject::id_to_obj(dir_path);
    if let FSRValue::String(s) = &dir_path_obj.value {
        let path = PathBuf::from(s.as_str());
        if path.is_dir() {
            return Ok(FSRRetValue::GlobalId(FSRObject::true_id()));
        }
        return Ok(FSRRetValue::GlobalId(FSRObject::false_id()));
    }

    Err(FSRError::new(
        "Invalid directory path argument",
        FSRErrCode::RuntimeError,
    ))
}

// file.read(offset: Integer, size: Integer) -> Bytes | String
pub fn fsr_fn_read(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId,
) -> Result<FSRRetValue, FSRError> {
    if len < 3 {
        return Err(FSRError::new(
            "fsr_fn_read requires at least 2 arguments",
            FSRErrCode::RuntimeError,
        ));
    }
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let file_obj_id = args[0];
    let file_obj = FSRObject::id_to_mut_obj(file_obj_id).unwrap();
    let offset = args[1];
    let size = args[2];

    let offset = if let FSRValue::Integer(i) = &FSRObject::id_to_obj(offset).value {
        *i as usize
    } else {
        return Err(FSRError::new(
            "Offset must be an integer",
            FSRErrCode::RuntimeError,
        ));
    };

    let size = if let FSRValue::Integer(i) = &FSRObject::id_to_obj(size).value {
        *i as usize
    } else {
        return Err(FSRError::new(
            "Size must be an integer",
            FSRErrCode::RuntimeError,
        ));
    };
    
    if let FSRValue::Extension(any_type) = &mut file_obj.value {
        if let Some(inner_file) = any_type.value.as_any_mut().downcast_mut::<FSRInnerFile>() {
            use std::io::Read;
            inner_file.seek(offset)?;
            let mut buffer = vec![0; size];
            inner_file
                .reader
                .read_exact(&mut buffer)
                .with_context(|| anyhow!("Failed to read from file: {}", inner_file.get_path()))?;
            if inner_file.mode == OpMode::Bytes {
                let ret = FSRValue::Bytes(Box::new(FSRInnerBytes::new(buffer)));
                let ret = thread
                    .garbage_collect
                    .new_object(ret, gid(GlobalObj::BytesCls));
                return Ok(FSRRetValue::GlobalId(ret));
            } else {
                let ret = String::from_utf8(buffer)
                    .map_err(|e| FSRError::new(e.to_string(), FSRErrCode::RuntimeError))?;
                let ret = FSRString::new_value(ret);
                let ret = thread
                    .garbage_collect
                    .new_object(ret, gid(GlobalObj::StringCls));
                return Ok(FSRRetValue::GlobalId(ret));
            }
        }
    }

    Err(FSRError::new(
        "Invalid file object",
        FSRErrCode::RuntimeError,
    ))
}

pub fn fsr_fn_file_lines(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
    code: ObjId,
) -> Result<FSRRetValue, FSRError> {
    if len < 1 {
        return Err(FSRError::new(
            "fsr_fn_file_lines requires at least 1 argument",
            FSRErrCode::RuntimeError,
        ));
    }
    let args = unsafe { std::slice::from_raw_parts(args, len) };
    let file_obj_id = args[0];
    let file_obj = FSRObject::id_to_mut_obj(file_obj_id).unwrap();

    if let FSRValue::Extension(any_type) = &mut file_obj.value {
        if let Some(inner_file) = any_type.value.as_any_mut().downcast_mut::<FSRInnerFile>() {
            let file = File::open(inner_file.get_path())
                .with_context(|| anyhow!("Failed to open file: {}", inner_file.get_path()))?;
            let reader = BufReader::new(file);
            let iter = reader.lines();
            let line_iter = FSRFileLineIterator {
                file_obj: file_obj_id,
                iter,
            };

            let inner_iter = FSRInnerIterator {
                obj: file_obj_id,
                iterator: Some(Box::new(line_iter)),
            };

            let value = FSRValue::Iterator(Box::new(inner_iter));

            let iter_obj_id = thread
                .garbage_collect
                .new_object(value, gid(GlobalObj::InnerIterator));
            return Ok(FSRRetValue::GlobalId(iter_obj_id));
        }
    }

    Err(FSRError::new(
        "Invalid file object",
        FSRErrCode::RuntimeError,
    ))
}




pub struct FSRFileLineIterator {
    pub(crate) file_obj: ObjId,
    pub(crate) iter: std::io::Lines<BufReader<File>>,
}

impl FSRIteratorReferences for FSRFileLineIterator {
    fn ref_objects(&self) -> Vec<ObjId> {
        vec![self.file_obj]
    }
}

impl FSRIterator for FSRFileLineIterator {
    fn next(&mut self, thread: &mut FSRThreadRuntime) -> Result<Option<ObjId>, FSRError> {
        let line = self.iter.next();
        match line {
            Some(Ok(line)) => {
                let line_obj = FSRString::new_value(line);
                let line_obj_id = thread
                    .garbage_collect
                    .new_object(line_obj, gid(GlobalObj::StringCls));
                Ok(Some(line_obj_id))
            }
            Some(Err(e)) => Err(FSRError::new(
                format!("Error reading line: {}", e),
                FSRErrCode::RuntimeError,
            )),
            None => Ok(None),
        }
    }
}
