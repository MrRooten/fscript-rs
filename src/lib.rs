#![allow(unused)]
#![allow(static_mut_refs)]
#![allow(clippy::vec_box)]
pub mod frontend;
pub mod backend;
pub mod utils;
pub mod std;


#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;