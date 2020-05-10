extern crate analyzer;

use analyzer::{ExtrinsicCollection, FileCollector};

use failure::Error;

#[test]
fn test_step_table() -> Result<(), Error> {
    //let collector = FileCollector::new("tests/files/shortened_files/")?;
    let collector = FileCollector::new("tests/files/full_files/")?;
    let mut collection = ExtrinsicCollection::new();

    for result in collector {
        let extrinsic_result = result?.parse()?;
        collection.push(extrinsic_result);
    }

    let mut table = collection.generate_step_table();
    table.sort_by_extrinsic_incr_percentage();
    let list = table.raw_list();

    for entry in &list {
        println!("{:?}", entry);
    }

    Ok(())
}
