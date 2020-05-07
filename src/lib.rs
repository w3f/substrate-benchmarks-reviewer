use std::convert::AsRef;
use std::fs::{self, File};
use std::io::prelude::*;
use std::iter::Iterator;
use std::path::Path;
use std::path::PathBuf;

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
            return Some(read_file(path.as_path()));
        }

        None
    }
}

pub struct BenchmarkAnalyser {}

#[derive(Debug, Fail)]
enum AnalyserError {
    #[fail(display = "header value of the benchmark result is missing")]
    MissingHeader,
    #[fail(display = "header value of the benchmark result is invalid")]
    InvalidHeader,
}

use self::AnalyserError::*;

impl BenchmarkAnalyser {
    pub fn new() -> Self {
        BenchmarkAnalyser {}
    }
    /// Parses the header of the result file. This function has strict requirements.
    ///
    /// Example:
    /// ```
    /// Pallet: "balances", Extrinsic: "set_balance", Lowest values: [], Highest values: [], Steps: [10], Repeat: 10
    /// u,e,extrinsic_time,storage_root_time
    /// ```
    #[rustfmt::skip]
    pub fn parse_header(content: &ResultContent) -> Result<(), Error> {
        let mut step_entry = StepEntry::default();

        let lines: Vec<&str> = content.lines().take(2).collect();

        // Parse the first line
        {
            let parts: Vec<&str> = lines
                .get(0)
                .ok_or(MissingHeader)?
                .split_whitespace()
                .collect();

            fn check_requirements(
                input_key: &str,
                input_val: &str,
                key_name: &str,
                len: usize,
                val_start: &str,
                val_end: &str,
                to_remove: &[&str],
            ) -> Result<String, Error> {
                // E.g. ... == "Pallet:"
                check(|| input_key == key_name)?;

                // E.g. ... == "Extrinsic:"
                check(|| {
                    input_val.len() > len
                        && input_val.starts_with(val_start)
                        && input_val.ends_with(val_end)
                })?;

                // E.g. from `"set_balance",` -> `set_balance`
                let mut value = String::from(input_val);
                for r in to_remove {
                    value = value.replace(r, "");
                }

                Ok(value)
            }

            check(|| parts.len() == 13)?;

            // Parse pallet name
            step_entry.pallet =
                check_requirements(parts[0], parts[1], "Pallet:", 3, "\"", "\",", &["\"", ","])?;

            // Parse extrinsic name
            step_entry.extrinsic = check_requirements(parts[2], parts[3], "Extrinsic:", 3, "\"", "\",", &["\"", ","])?;

            // Parse steps amount
            step_entry.steps =
                check_requirements(parts[9], parts[10], "Steps:", 2, "[", "],", &["[", ","])?
                    .parse::<usize>()
                    .map_err(|_| InvalidHeader)?;

            // Parse repeat amount. The amount does not have brackets around it, probably removed by accident.
            // Generally not an issue, just a small inconsistency.
            step_entry.steps =
                check_requirements(parts[11], parts[12], "Repeat:", 2, "", "", &["", ""])?
                    .parse::<usize>()
                    .map_err(|_| InvalidHeader)?;
        }

        Ok(())
    }
}

fn check<F>(func: F) -> Result<(), Error>
where
    F: Fn() -> bool,
{
    if !func() {
        return Err(InvalidHeader.into());
    }

    Ok(())
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
    value: usize,
}
