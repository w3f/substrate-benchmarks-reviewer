extern crate analyzer;

use analyzer::{ExtrinsicCollection, FileCollector};

use failure::Error;

#[test]
/// Tests if it can read the full files as they're generated
/// by the substrate benchmark binary.
fn test_overview_table_full_files() -> Result<(), Error> {
    let collector = FileCollector::new("tests/files/full_files/")?;
    let mut collection = ExtrinsicCollection::new();

    for result in collector {
        let extrinsic_result = result?.parse()?;
        collection.push(extrinsic_result);
    }

    let mut table = collection.generate_overview_table();
    table.sort_by_ratio();

    let expected = [
        ("identity", "add_registrar", 1.0, 0.0),
        ("treasury", "tip_new", 2.1406, 114.0605),
        ("balances", "transfer", 2.233, 123.3014),
        ("staking", "bond_extra", 2.2533, 125.333),
        ("democracy", "delegate", 18.3259, 1732.5924)
    ];


    let mut counter = 0;
    let list = table.list();
    for entry in list {
        assert_eq!(entry.0, expected[counter].0);
        assert_eq!(entry.1, expected[counter].1);
        assert_eq!(entry.2, expected[counter].2);
        assert_eq!(entry.3, expected[counter].3);

        counter += 1;
    }

    //table.print_entries();

    Ok(())
}

/// Test shortened files, where the expected results have been recalculated by hand.
#[test]
fn test_overview_table_shortened() -> Result<(), Error> {
    let collector = FileCollector::new("tests/files/shortened_files/")?;
    let mut collection = ExtrinsicCollection::new();

    for result in collector {
        let extrinsic_result = result?.parse()?;
        collection.push(extrinsic_result);
    }

    let mut table = collection.generate_overview_table();
    table.sort_by_ratio();

    let expected = [
        ("identity", "add_registrar", 1.0, 0.0),
        ("treasury", "tip_new", 1.8363, 83.6271),
        ("balances", "transfer", 2.4501, 145.0108),
        ("staking", "bond_extra", 2.4575, 145.7468),
        ("democracy", "delegate", 19.6006, 1860.0573)
    ];


    let mut counter = 0;
    let list = table.list();
    for entry in list {
        assert_eq!(entry.0, expected[counter].0);
        assert_eq!(entry.1, expected[counter].1);
        assert_eq!(entry.2, expected[counter].2);
        assert_eq!(entry.3, expected[counter].3);

        counter += 1;
    }

    //table.print_entries();

    Ok(())
}
