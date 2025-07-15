pub mod error;
pub mod logger;

/// Utility functions and macros for FScript
/// This module provides various utility functions and macros that are commonly used across the FScript codebase.
#[macro_export]
macro_rules! register_fn {
    ($module:expr, $thread:expr, $name:expr, $func:expr) => {{
        let fn_obj = crate::backend::types::fn_def::FSRFn::from_rust_fn_static_value($func, $name);
        let fn_id = $thread.garbage_collect.new_object(fn_obj, crate::backend::types::base::GlobalObj::FnCls.get_id());
        $module.register_object($name, fn_id);
    }};
}

#[macro_export]
macro_rules! register_class {
    ($module:expr, $thread:expr, $name:expr, $class:expr) => {{
        let value = crate::backend::types::base::FSRValue::Class(Box::new($class));
        let class_cls_id = crate::backend::types::base::GlobalObj::ClassCls.get_id();
        let object_id = $thread.garbage_collect.new_object(value, class_cls_id);
        if let crate::backend::types::base::FSRValue::Class(c) = &mut crate::backend::types::base::FSRObject::id_to_mut_obj(object_id).unwrap().value {
            c.set_object_id(object_id);
        }
        $module.register_object($name, object_id);
    }};
}