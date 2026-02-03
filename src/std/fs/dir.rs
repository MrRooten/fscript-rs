use std::{
    fs::{self, DirEntry},
    path::PathBuf,
};

use crate::{
    backend::{
        types::{
            any::ExtensionTrait,
            base::{FSRObject, FSRRetValue, FSRValue, GlobalObj, ObjId},
            class::FSRClass,
            fn_def::FSRFn,
            list::FSRList,
            string::FSRString,
        },
        vm::thread::FSRThreadRuntime,
    },
    to_rs_list,
    utils::error::FSRError,
};
use std::fmt::Debug;

#[derive(Debug)]
pub struct FSRDir {
    inner_dir: DirEntry,
}

impl ExtensionTrait for FSRDir {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
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

pub fn fsr_fn_sub_path(
    args: *const ObjId,
    len: usize,
    thread: &mut FSRThreadRuntime,
) -> Result<FSRRetValue, FSRError> {
    if len < 1 {
        panic!("Invalid arguments for sub_paths");
    }

    let args = to_rs_list!(args, len);
    let path_id = args[0];
    let path_obj = FSRObject::id_to_obj(path_id);

    let s = if let FSRValue::String(s) = &path_obj.value {
        s
    } else {
        panic!("sub_paths expects a string path argument");
    };

    let dir_path = std::path::Path::new(s.as_str());
    let mut sub_paths = Vec::new();
    for entry in fs::read_dir(dir_path).map_err(|e| {
        FSRError::new(
            format!("Read dir error: {}", e),
            crate::utils::error::FSRErrCode::RuntimeError,
        )
    })? {
        let entry = entry.map_err(|e| {
            FSRError::new(
                format!("DirEntry error: {}", e),
                crate::utils::error::FSRErrCode::RuntimeError,
            )
        })?;
        let sub_path = entry
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        sub_paths.push(sub_path);
    }

    let path_vec = sub_paths
        .iter()
        .map(|x| {
            let value = FSRString::new_value(x);
            thread
                .garbage_collect
                .new_object(value, GlobalObj::StringCls.get_id())
        })
        .collect::<Vec<_>>();
    let value = FSRList::new_value(path_vec);
    let res = thread
        .garbage_collect
        .new_object(value, GlobalObj::ListCls.get_id());

    Ok(FSRRetValue::GlobalId(res))
}

impl FSRDir {
    pub fn get_class() -> FSRClass {
        let mut cls = FSRClass::new("Dir");
        cls.init_method();
        let sub_path = FSRFn::from_rust_fn_static(fsr_fn_sub_path, "sub_paths");
        cls.insert_attr("sub_paths", sub_path);
        cls
    }
}
