use clap::{App, Arg, SubCommand};
use failure::Error;

use libreviewer::{ExtrinsicCollection, FileScraper};

fn build_collection(path: &str, skip_warn: bool) -> Result<ExtrinsicCollection, Error> {
    let scraper = FileScraper::new(path)?;
    let mut collection = ExtrinsicCollection::new();

    for result in scraper {
        let _ = result?
            .parse()
            .map(|result| {
                collection.push(result);
                ()
            })
            .map_err(|err| {
                if !skip_warn {
                    eprintln!("Warn: {}", err.to_string());
                }
                ()
            });
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
                .arg(Arg::with_name("csv").long("csv"))
                .arg(Arg::with_name("skip-warnings").long("skip-warnings")),
        )
        .subcommand(
            SubCommand::with_name("step")
                .arg(Arg::with_name("PATH").required(true))
                .arg(Arg::with_name("csv").long("csv"))
                .arg(Arg::with_name("skip-warnings").long("skip-warnings")),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("ratio") {
        // Unwrapping is ok, since "PATH" is set to required
        let collection = build_collection(
            matches.value_of("PATH").unwrap(),
            matches.is_present("skip-warnings"),
        )?;

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
        let collection = build_collection(
            matches.value_of("PATH").unwrap(),
            matches.is_present("skip-warnings"),
        )?;

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
