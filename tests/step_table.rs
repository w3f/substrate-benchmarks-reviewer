extern crate libreviewer;

use libreviewer::{ExtrinsicCollection, FileCollector};

use failure::Error;

#[test]
#[rustfmt::skip]
fn test_step_table() -> Result<(), Error> {
    let collector = FileCollector::new("tests/files/steps/")?;
    let mut collection = ExtrinsicCollection::new();

    for result in collector {
        let extrinsic_result = result?.parse()?;
        collection.push(extrinsic_result);
    }

    let mut table = collection.generate_step_table().unwrap();
    table.sort_by_extrinsic_incr_percentage();

    let expected = [
        ("balances", "set_balance_killing", &vec![1, 1000], 122488.6667, 79915.0, 15.6470, 27.0004),
        ("balances", "set_balance_killing", &vec![199, 1000], 108735.6667, 71866.3333, 2.6622, 14.2095),
        ("balances", "set_balance_killing", &vec![496, 1000], 105916.0, 62925.0, 0.0, 0.0),
        ("democracy", "propose", &vec![19], 135927.0, 106922.0, 7.9605, 27.1982),
        ("democracy", "propose", &vec![10], 130751.0, 89704.8, 3.8495, 6.7160),
        ("democracy", "propose", &vec![1], 125904.3333, 84059.3333, 0.0, 0.0),
    ];

    let list = table.raw_list();

    /*
    for entry in &list {
        println!("{:?}", entry);
    }
    */

    let mut counter = 0;
    for entry in list {
        assert_eq!(entry.0, expected[counter].0);
        assert_eq!(entry.1, expected[counter].1);
        assert_eq!(entry.2, expected[counter].2.as_slice());
        assert_eq!(entry.3, expected[counter].3);
        assert_eq!(entry.4, expected[counter].4);
        assert_eq!(entry.5, expected[counter].5);
        assert_eq!(entry.6, expected[counter].6);

        counter += 1;
    }

    Ok(())
}
