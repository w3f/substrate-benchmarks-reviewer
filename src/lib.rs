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

impl BenchmarkAnalyser {
    pub fn new() -> Self {
        BenchmarkAnalyser {}
    }
    /// Parses the header of the result file. This function has slightly stricter requirements.
    ///
    /// Example:
    /// ```
    /// Pallet: "balances", Extrinsic: "set_balance", Lowest values: [], Highest values: [], Steps: [10], Repeat: 10
    /// u,e,extrinsic_time,storage_root_time
    /// ```
    #[rustfmt::skip]
    pub fn parse_header(content: &ResultContent) -> Result<StepEntry, Error> {
        let mut step_entry = StepEntry::default();

        let lines: Vec<&str> = content.lines().take(2).collect();

        // Parse the first line
        {
            let parts: Vec<&str> = lines
                .get(0)
                .ok_or(MissingHeader)?
                .split_whitespace()
                .collect();

            // Length is checked here, so directly indexing
            // the vector after this is safe.
            check(|| parts.len() == 13)?;

            // Parse pallet name
            step_entry.pallet =
                check_requirements(parts[0], parts[1], "Pallet:", "\"", "\",")?;

            // Parse extrinsic name
            step_entry.extrinsic =
                check_requirements(parts[2], parts[3], "Extrinsic:", "\"", "\",")?;

            // Parse steps amount
            step_entry.steps =
                check_requirements(parts[9], parts[10], "Steps:", "[", "],")?
                    .parse::<usize>()
                    .map_err(|_| InvalidHeader)?;

            // Parse repeat amount. The amount does not have brackets around it,
            // probably skipped by accident. Generally not an issue, just a
            // small inconsistency.
            step_entry.steps =
                check_requirements(parts[11], parts[12], "Repeat:", "", "")?
                    .parse::<usize>()
                    .map_err(|_| InvalidHeader)?;
        }

        // Parse second line
        {
            let parts: Vec<&str> = lines
                .get(1)
                .ok_or(MissingHeader)?
                .split(",")
                .collect();

            check(|| parts.len() > 2)?;
            check(|| parts.contains(&"extrinsic_time"))?;
            check(|| parts.contains(&"storage_root_time"))?;

            let mut offset = 0;
            for part in &parts {
                if part == &"extrinsic_time" {
                    break;
                }

                // E.g. part = `u`
                check(|| part.len() == 1);

                offset += 1;
            }

            parts
                .iter()
                .take(offset)
                .for_each(|var| {
                    step_entry.input_var_names.push(var.to_string())
                });
        }

        Ok(step_entry)
    }
    fn parse_body(content: &ResultContent, expected_len: usize) -> Result<Vec<RepeatEntry>, Error> {
        let mut coll = Vec::new();
        let lines: Vec<&str> = content.lines().skip(2).collect();

        for line in lines {
            let parts: Vec<&str> = line.split(",").collect();

            // Must have the expected length:
            // -> variables + "extrinsic_time" + "storage_root_time"
            if parts.len() != expected_len || parts.len() < 2 {
                break;
            }

            // All parts must be numeric
            if parts.iter().all(|p| !p.parse::<usize>().is_ok()) {
                break;
            }

            let mut repeat_entry = RepeatEntry::default();

            // Fill in the data. The length and conversion validity is checked above,
            // so directly indexing and unwrapping is safe here.
            let temp: Vec<&&str> = parts.iter().rev().take(2).collect();
            repeat_entry.storage_root_time = temp[0].parse::<u64>().unwrap();
            repeat_entry.extrinsic_time = temp[1].parse::<u64>().unwrap();
            repeat_entry.input_vars = parts
                .iter()
                .take(expected_len - 2)
                .map(|p| p.parse::<usize>().unwrap())
                .collect();

            coll.push(repeat_entry);
        }

        Ok(coll)
    }
}

/// Checks the requirements of the header (benchmark description) key and value.
/// - input key
/// - input value
/// - key must equal ...
/// - value starts with ...
/// - value ends with ...
fn check_requirements(
    input_key: &str,
    input_val: &str,
    key_name: &str,
    val_start: &str,
    val_end: &str,
) -> Result<String, Error> {
    // E.g. ... == "Pallet:"
    check(|| input_key == key_name)?;

    // E.g. ... starts with `"` and ends with `",`
    check(|| input_val.starts_with(val_start) && input_val.ends_with(val_end))?;

    // E.g. from `"balances",` -> `balances`
    Ok(String::from(input_val)
        .replace(val_start, "")
        .replace(val_end, ""))
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
