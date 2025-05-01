#![allow(unused)]
#![allow(static_mut_refs)]
#![allow(clippy::vec_box)]
pub mod frontend;
pub mod backend;
pub mod utils;
pub mod std;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;