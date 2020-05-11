use clap::{App, Arg, SubCommand};
use failure::Error;

use libreviewer::{ExtrinsicCollection, FileScraper};

fn build_collection(path: &str) -> Result<ExtrinsicCollection, Error> {
    let scraper = FileScraper::new(path)?;
    let mut collection = ExtrinsicCollection::new();

    for result in scraper {
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
        .subcommand(
            SubCommand::with_name("ratio")
                .arg(Arg::with_name("PATH").required(true))
                .arg(Arg::with_name("csv").long("csv")),
        )
        .subcommand(
            SubCommand::with_name("step")
                .arg(Arg::with_name("PATH").required(true))
                .arg(Arg::with_name("csv").long("csv")),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("ratio") {
        // Unwrapping is ok, since "PATH" is set to required
        let collection = build_collection(matches.value_of("PATH").unwrap())?;

        let mut table = collection.generate_ratio_table()?;
        table.sort_by_ratio();

        if matches.is_present("csv") {
            table.print_csv();
        } else {
            table.print();
        }
    }

    if let Some(matches) = matches.subcommand_matches("step") {
        // Unwrapping is ok, since "PATH" is set to required
        let collection = build_collection(matches.value_of("PATH").unwrap())?;

        let mut table = collection.generate_step_table()?;
        table.sort_by_extrinsic_incr_percentage();

        if matches.is_present("csv") {
            table.print_csv();
        } else {
            table.print();
        }
    }

    Ok(())
}
