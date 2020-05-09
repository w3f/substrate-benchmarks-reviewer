extern crate analyzer;

use analyzer::{ExtrinsicCollection, FileCollector};

use failure::Error;

#[test]
fn test_overview_table() -> Result<(), Error> {
    let collector = FileCollector::new("tests/files/")?;
    let mut collection = ExtrinsicCollection::new();

    for result in collector {
        let extrinsic_result = result?.parse()?;
        collection.push(extrinsic_result);
    }

    let mut table = collection.generate_ratio_table();
    //println!("TABLE: {:?}", table);
    table.sort_by_ratio();
    table.print_entries();

    Ok(())
}
