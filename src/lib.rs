use std::convert::AsRef;
use std::path::Path;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::PathBuf;
use std::iter::Iterator;

#[macro_use]
extern crate failure;

use failure::Error;

type ResultContent = String;

pub struct FileCollector {
    files: Vec<PathBuf>,
    count: usize,
}

impl FileCollector {
    /// Recursively searches for all files within the specified `path`, saving those internally.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<FileCollector, Error> {
        Ok(
            FileCollector {
                files: Self::find_files(path)?,
                count: 0,
            }
        )
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
fn read_file<P: AsRef<Path>>(path: P) -> Result<String, Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

impl Iterator for FileCollector {
    type Item = Result<ResultContent, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let path = self.files.get(self.count).map(|e| e.clone());
        self.count += 1;

        if let Some(path) = path {
            return Some(read_file(path.as_path()))
        }

        None
    }
}

pub struct BenchmarkAnalyser {

}

#[derive(Debug, Fail)]
enum AnalyserError {
    #[fail(display = "header value of the benchmark result is invalid")]
    MissingHeader
}

use self::AnalyserError::*;

impl BenchmarkAnalyser {
    pub fn new() -> Self {
        BenchmarkAnalyser {

        }
    }
    /// Parses the header of the result file.
    /// 
    /// Example:
    /// ```
    /// Pallet: "balances", Extrinsic: "set_balance", Lowest values: [], Highest values: [], Steps: [10], Repeat: 10
    /// u,e,extrinsic_time,storage_root_time
    /// ```
    pub fn parse_header(content: &ResultContent) -> Result<(), Error> {
        let mut step_entry = StepEntry::default();

        let lines: Vec<&str> = content
            .lines()
            .take(2)
            .collect();

        let bench_info = lines.get(0).ok_or(MissingHeader)?;

        Ok(())
    }
}

#[derive(Default)]
struct StepEntry {
    pallet: String,
    extrinsic: String,
    repeat_entries: Vec<RepeatEntry>,
    steps: usize,
    repeats: usize,
}

struct RepeatEntry {
    input_vars: Vec<InputVar>,
    extrinsic_time: u64,
    storage_root_time: u64,
}

struct InputVar {
    name: String,
    value: usize
}
