use analyzer::FileCollector;

use failure::Error;

fn main() -> Result<(), Error> {
    let collector = FileCollector::new("dir/")?;
    for result in collector {
        let extrinsic_result = result?.parse()?;
    }

    Ok(())
}
