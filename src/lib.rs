pub mod file_collector;
mod parser;

pub use file_collector::{FileContent, FileCollector};

#[macro_use]
extern crate failure;

#[derive(Default)]
pub struct ExtrinsicResult {
    pallet: String,
    extrinsic: String,
    steps: usize,
    repeats: usize,
    input_var_names: Vec<String>,
    repeat_entries: Vec<RepeatEntry>,
}

#[derive(Default)]
pub struct RepeatEntry {
    input_vars: Vec<usize>,
    extrinsic_time: u64,
    storage_root_time: u64,
}

