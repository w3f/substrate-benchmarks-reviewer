use super::{parser, ExtrinsicResult};

use std::convert::AsRef;
use std::fs::{self, File};
use std::io::prelude::*;
use std::iter::Iterator;
use std::path::Path;
use std::path::PathBuf;

use failure::Error;

#[derive(Debug, Fail)]
enum FileContentError {
    #[fail(display = "Invalid document: {}", 0)]
    InvalidDocument(String),
}

use FileContentError::*;

pub struct FileContent(pub(crate) (String, PathBuf));

impl FileContent {
    pub fn parse(&self) -> Result<ExtrinsicResult, Error> {
        let mut extrinsic_result = parser::parse_header(self)
            .map_err(|_| InvalidDocument((self.0).1.to_string_lossy().to_string()))?;
        let expected_len = extrinsic_result.input_var_names.len() + 2;
        extrinsic_result.steps_repeats = parser::parse_body(self, expected_len)
            .map_err(|_| InvalidDocument((self.0).1.to_string_lossy().to_string()))?;
        Ok(extrinsic_result)
    }
}

pub struct FileScraper {
    files: Vec<PathBuf>,
    count: usize,
}

impl FileScraper {
    /// Recursively searches for all files within the specified `path`,
    /// saving those internally.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<FileScraper, Error> {
        Ok(FileScraper {
            files: find_files(path)?,
            count: 0,
        })
    }
}

/// Searches for files insides the specified `path` and saves the full path of each
/// file. If a directory is found, this function will repeat that same process for
/// that subdirectory (recursion).
fn find_files<P: AsRef<Path>>(path: P) -> Result<Vec<PathBuf>, Error> {
    let mut coll = Vec::new();

    for entry in fs::read_dir(path)? {
        let path = entry?.path();
        if path.is_dir() {
            coll.append(&mut find_files(&path)?);
        } else {
            coll.push(path);
        }
    }

    Ok(coll)
}

/// Read file directly to memory. The output of an individual benchmark
/// output is quite small, so reading the full thing will no create any
/// issues.
fn read_file<P: AsRef<Path>>(path: P) -> Result<FileContent, Error> {
    let mut file = File::open(path.as_ref())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(FileContent((contents, path.as_ref().to_path_buf())))
}

impl Iterator for FileScraper {
    type Item = Result<FileContent, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let path = self.files.get(self.count)?;
        self.count += 1;
        Some(read_file(path.as_path()))
    }
}
