#[macro_use]
extern crate clap;
use clap::{App, Arg, SubCommand};
use failure::Error;

use libreviewer::{ExtrinsicCollection, FileCollector};

fn build_collection(path: &str) -> Result<ExtrinsicCollection, Error> {
    let collector = FileCollector::new(path)?;
    let mut collection = ExtrinsicCollection::new();

    for result in collector {
        let extrinsic_result = result?.parse()?;
        collection.push(extrinsic_result);
    }

    Ok(collection)
}

fn main() -> Result<(), Error> {
    let matches = App::new("bench-reviewer")
        .version("1.0")
        .author("Fabio Lama <github.com/lamafab>")
        .about("Creates overview tables of substrate module benchmarks.")
        .subcommand(SubCommand::with_name("ratio").arg(Arg::with_name("PATH").required(true)))
        .subcommand(SubCommand::with_name("step").arg(Arg::with_name("PATH").required(true)))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("ratio") {
        let collection = build_collection(matches.value_of("PATH").unwrap())?;

        let mut table = collection.generate_ratio_table()?;
        table.sort_by_ratio();

        for entry in &table.raw_list() {
            println!("{:?}", entry);
        }
    }

    if let Some(matches) = matches.subcommand_matches("step") {
        let collection = build_collection(matches.value_of("PATH").unwrap())?;

        let mut table = collection.generate_step_table()?;
        table.sort_by_extrinsic_incr_percentage();

        for entry in &table.raw_list() {
            println!("{:?}", entry);
        }
    }

    Ok(())
}
