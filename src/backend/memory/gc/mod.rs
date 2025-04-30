pub mod mark_sweep;
// pub mod triple_color;

#[derive(Debug)]
pub struct Tracker {
    object_count: u32,
    throld: usize,
    pub(crate) collect_time: u64, // in microseconds
    count_free: u64,
    collect_count: u64,
    minjar_object_count: u32,
    marjor_object_count: u32,
}