use std::convert::AsRef;
use std::path::Path;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::PathBuf;

use failure::Error;

/// Read file directly to memory. The output of an individual benchmark
/// output is quite small, so reading the full thing will no create any
/// issues.
pub fn read_file<P: AsRef<Path>>(path: &Path) -> Result<String, Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

struct ResultsReader {
    files: Vec<PathBuf>,
}

impl ResultsReader {
    pub fn new<P: AsRef<Path>>(target: &Path) -> Result<ResultsReader, Error> {
        let files = fs::read_dir(target)?
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| !path.is_dir())
                .collect();

        Ok(
            ResultsReader {
                files: files,
            }
        )
    }
}
