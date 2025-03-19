use super::{base::FSRPosition, block::FSRBlock};

pub struct FSRTryBlock<'a> {
    pub try_block: Box<FSRBlock<'a>>,
    pub(crate) len: usize,
    pub(crate) meta: FSRPosition,
}