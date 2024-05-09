use std::{collections::HashMap, sync::atomic::AtomicU64};

use crate::backend::types::base::FSRObject;

struct ObjectRef {
    obj_id      : u64,
    ref_count   : AtomicU64
}

pub struct GarbageCollect<'a> {
    obj_map: HashMap<u64, Box<FSRObject<'a>>>
}