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

struct FileCollector {
    files: Vec<PathBuf>,
}

impl FileCollector {
    pub fn new<P: AsRef<Path>>(path: &Path) -> Result<FileCollector, Error> {
        Ok(
            FileCollector {
                files: Self::find_files::<&Path>(path)?,
            }
        )
    }
    /// Search for files insides the specified `path` and collect all file paths. If a directory is found,
    /// this function will repeat that same process for that subdirectory (recursion).
    pub fn find_files<P: AsRef<Path>>(path: &Path) -> Result<Vec<PathBuf>, Error> {
        let mut coll = Vec::new();

        for entry in fs::read_dir(path)? {
            let path = entry?.path();
            if path.is_dir() {
                coll.append(&mut Self::find_files::<&Path>(&path)?);
            } else {
                coll.push(path);
            }
        }

        Ok(coll)
    }
}
