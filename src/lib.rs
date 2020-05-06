use std::convert::AsRef;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

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