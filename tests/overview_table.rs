extern crate libreviewer;

use libreviewer::{ExtrinsicCollection, FileCollector};

use failure::Error;

#[test]
#[rustfmt::skip]
/// Tests if it can read the full files as they're generated
/// by the substrate benchmark binary.
fn test_overview_table_full_files() -> Result<(), Error> {
    let collector = FileCollector::new("tests/files/full/")?;
    let mut collection = ExtrinsicCollection::new();

    for result in collector {
        let extrinsic_result = result?.parse()?;
        collection.push(extrinsic_result);
    }

    let mut table = collection.generate_ratio_table().unwrap();
    table.sort_by_ratio();

    let expected = [
        ("identity", "add_registrar", 82669.6368, 47011.8684, 1.0, 0.0),
        ("treasury", "tip_new", 176963.0696, 88054.7609, 2.1406, 114.0605),
        ("balances", "transfer", 184602.4227, 82891.3318, 2.233, 123.3014),
        ("staking", "bond_extra", 186282.0091, 121143.1636, 2.2533, 125.333),
        ("democracy", "delegate", 1514997.45, 464842.2167, 18.3259, 1732.5924),
    ];

    let list = table.raw_list();
    assert_eq!(list.len(), 5);

    let mut counter = 0;
    for entry in list {
        assert_eq!(entry.0, expected[counter].0);
        assert_eq!(entry.1, expected[counter].1);
        assert_eq!(entry.2, expected[counter].2);
        assert_eq!(entry.3, expected[counter].3);
        assert_eq!(entry.4, expected[counter].4);
        assert_eq!(entry.5, expected[counter].5);

        counter += 1;
    }

    //table.print_entries();

    Ok(())
}

#[test]
#[rustfmt::skip]
/// Test shortened files, where the expected results have been re-calculated by hand.
fn test_overview_table_shortened() -> Result<(), Error> {
    let collector = FileCollector::new("tests/files/shortened/")?;
    let mut collection = ExtrinsicCollection::new();

    for result in collector {
        let extrinsic_result = result?.parse()?;
        collection.push(extrinsic_result);
    }

    let mut table = collection.generate_ratio_table().unwrap();
    table.sort_by_ratio();

    let expected = [
        ("identity", "add_registrar", 76600.8, 43874.4, 1.0, 0.0),
        ("treasury", "tip_new", 140659.8333, 61608.3333, 1.8363, 83.6271),
        ("balances", "transfer", 187680.2, 83780.8, 2.4501, 145.0108),
        ("staking", "bond_extra", 188244.0, 117963.0, 2.4575, 145.7468),
        ("democracy", "delegate", 1501419.6, 464099.8, 19.6006, 1860.0573),
    ];

    let list = table.raw_list();
    assert_eq!(list.len(), 5);

    /*
    for entry in &list {
        println!("{:?}", entry);
    }
    */

    let mut counter = 0;
    for entry in list {
        assert_eq!(entry.0, expected[counter].0);
        assert_eq!(entry.1, expected[counter].1);
        assert_eq!(entry.2, expected[counter].2);
        assert_eq!(entry.3, expected[counter].3);
        assert_eq!(entry.4, expected[counter].4);
        assert_eq!(entry.5, expected[counter].5);

        counter += 1;
    }

    Ok(())
}
