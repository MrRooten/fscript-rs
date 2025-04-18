use crate::backend::types::base::FSRObject;

use super::Tracker;

pub struct TripleColorGarbageCollector<'a> {
    pub white: Vec<Option<Box<FSRObject<'a>>>>,
    pub gray: Vec<Option<Box<FSRObject<'a>>>>,
    pub black: Vec<Option<Box<FSRObject<'a>>>>,
    tracker: Tracker,
}