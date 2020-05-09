use analyzer::FileCollector;

use failure::Error;

fn main() -> Result<(), Error> {
    let collector = FileCollector::new("dir/")?;
    for result in collector {
        let _extrinsic_result = result?.parse()?;
    }

    Ok(())
}
