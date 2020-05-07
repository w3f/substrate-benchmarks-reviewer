pub mod parser;

use std::convert::AsRef;
use std::fs::{self, File};
use std::io::prelude::*;
use std::iter::Iterator;
use std::path::Path;
use std::path::PathBuf;

#[macro_use]
extern crate failure;

use failure::Error;


pub struct ResultContent(String);

impl ResultContent {
    /*
    pub fn parse() -> Result<StepEntry, Error> {
        Ok()
    }
    */
}

pub struct FileCollector {
    files: Vec<PathBuf>,
    count: usize,
}

impl FileCollector {
    /// Recursively searches for all files within the specified `path`, saving those internally.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<FileCollector, Error> {
        Ok(FileCollector {
            files: Self::find_files(path)?,
            count: 0,
        })
    }
    /// Searches for files insides the specified `path` and collects all file paths. If a directory is found,
    /// this function will repeat that same process for that subdirectory (recursion).
    fn find_files<P: AsRef<Path>>(path: P) -> Result<Vec<PathBuf>, Error> {
        let mut coll = Vec::new();

        for entry in fs::read_dir(path)? {
            let path = entry?.path();
            if path.is_dir() {
                coll.append(&mut Self::find_files(&path)?);
            } else {
                coll.push(path);
            }
        }

        Ok(coll)
    }
}

/// Read file directly to memory. The output of an individual benchmark
/// output is quite small, so reading the full thing will no create any
/// issues.
fn read_file<P: AsRef<Path>>(path: P) -> Result<ResultContent, Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(ResultContent(contents))
}

impl Iterator for FileCollector {
    type Item = Result<ResultContent, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let path = self.files.get(self.count).map(|e| e.clone());
        self.count += 1;

        if let Some(path) = path {
            return Some(read_file(path.as_path()));
        }

        None
    }
}

#[derive(Default)]
pub struct StepEntry {
    pallet: String,
    extrinsic: String,
    repeat_entries: Vec<RepeatEntry>,
    steps: usize,
    repeats: usize,
    input_var_names: Vec<String>,
}

#[derive(Default)]
struct RepeatEntry {
    input_vars: Vec<usize>,
    extrinsic_time: u64,
    storage_root_time: u64,
}
