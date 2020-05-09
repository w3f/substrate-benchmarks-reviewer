pub mod file_collector;
mod parser;

pub use file_collector::{FileCollector, FileContent};

#[macro_use]
extern crate failure;

#[derive(Debug, Default)]
pub struct ExtrinsicResult {
    pallet: String,
    extrinsic: String,
    steps: usize,
    repeats: usize,
    input_var_names: Vec<String>,
    repeat_entries: Vec<RepeatEntry>,
}

trait RoundBy {
    fn round_by(&self, by: i32) -> Self;
}

impl RoundBy for f64 {
    fn round_by(&self, by: i32) -> Self {
        let precision = 10.0_f64.powi(by);
        (self * precision).round() / precision
    }
}

#[derive(Debug, Default)]
struct RepeatEntry {
    input_vars: Vec<u64>,
    extrinsic_time: u64,
    storage_root_time: u64,
}

impl ExtrinsicResult {
    // TODO: Ensure each generated average uses the same amount
    // of entries as the data it gets compared to.
    // TODO: Ensure length is never 0.
    fn average_extrinsic_time(&self) -> f64 {
        let mut total = 0;
        self.repeat_entries
            .iter()
            .for_each(|e| total += e.extrinsic_time);

        (total as f64 / self.repeat_entries.len() as f64)
    }
}

#[derive(Debug)]
pub struct ExtrinsicCollection {
    inner: Vec<ExtrinsicResult>,
}

impl ExtrinsicCollection {
    pub fn new() -> Self {
        ExtrinsicCollection { inner: Vec::new() }
    }
    pub fn push(&mut self, result: ExtrinsicResult) {
        self.inner.push(result);
    }
    pub fn generate_overview_table(&self) -> OverviewTable {
        // find base (lowest value)
        // TODO: Handle unwrap
        let base = self
            .inner
            .iter()
            .min_by(|x, y| {
                x.average_extrinsic_time()
                    .partial_cmp(&y.average_extrinsic_time())
                    .unwrap()
            })
            .unwrap()
            .average_extrinsic_time();

        let mut table = OverviewTable::new();

        self.inner.iter().for_each(|result| {
            let avg_time = result.average_extrinsic_time();
            table.push(RatioEntry {
                pallet: &result.pallet,
                extrinsic: &result.extrinsic,
                ratio: (avg_time / base).round_by(4),
                percentage: ((avg_time / base - 1.0) * 100.0).round_by(4),
            });
        });

        table
    }
}

#[derive(Debug)]
pub struct OverviewTable<'a> {
    inner: Vec<RatioEntry<'a>>,
}

#[derive(Debug)]
struct RatioEntry<'a> {
    pallet: &'a str,
    extrinsic: &'a str,
    ratio: f64,
    percentage: f64,
}

impl<'a> OverviewTable<'a> {
    pub fn new() -> Self {
        OverviewTable { inner: Vec::new() }
    }
    fn push(&mut self, entry: RatioEntry<'a>) {
        self.inner.push(entry);
    }
    pub fn sort_by_ratio(&mut self) {
        // TODO: Handle unwrap
        self.inner
            .sort_by(|a, b| a.ratio.partial_cmp(&b.ratio).unwrap());
    }
    pub fn sort_by_pallet(&mut self) {
        // TODO...
    }
    pub fn print_entries(&self) {
        let width = 14;

        // Print table header
        println!(
            "|{:^width$}|{:^width$}|{:^width$}|{:^width$}|",
            "Pallet",
            "Extrinsic",
            "Ratio",
            "Increase",
            width = width
        );

        // Print line
        for _ in 0..4 {
            print!("|{:-<width$}", "", width = width);
        }
        println!("|");

        // Print table body
        for entry in &self.inner {
            println!(
                "|{:<width$}|{:<width$}|{:<width$}|{:>width_incr$} %|",
                entry.pallet,
                entry.extrinsic,
                entry.ratio,
                entry.percentage,
                width = width,
                width_incr = width - 2
            );
        }
    }
}
