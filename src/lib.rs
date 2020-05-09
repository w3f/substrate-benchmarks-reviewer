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

#[derive(Debug, Default)]
struct RepeatEntry {
    input_vars: Vec<u64>,
    extrinsic_time: u64,
    storage_root_time: u64,
}

impl ExtrinsicResult {
    // TODO: Ensure each generated average uses the same amount
    // of entries as the data it gets compared to.
    fn average_extrinsic_time(&self) -> f64 {
        let mut total = 0;
        self.repeat_entries
            .iter()
            .for_each(|e| total += e.extrinsic_time);

        (0 as f64 / self.repeat_entries.len() as f64)
    }
}

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
    pub fn generate_ratio_table(&self) -> RatioTable {
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

        let mut table = RatioTable::new();

        self.inner.iter().for_each(|result| {
            let avg_time = result.average_extrinsic_time();
            table.push(RatioEntry {
                pallet: &result.pallet,
                extrinsic: &result.extrinsic,
                ratio: avg_time / base,
                percentage: ((avg_time / base - 1.0) * 100.0),
            });
        });

        table
    }
}

struct RatioEntry<'a> {
    pallet: &'a str,
    extrinsic: &'a str,
    ratio: f64,
    percentage: f64,
}

pub struct RatioTable<'a> {
    inner: Vec<RatioEntry<'a>>,
}

impl<'a> RatioTable<'a> {
    pub fn new() -> Self {
        RatioTable { inner: Vec::new() }
    }
    fn push(&mut self, entry: RatioEntry<'a>) {
        self.inner.push(entry);
    }
    fn sort_by_ratio(&mut self) {
        // TODO: Handle unwrap
        self.inner
            .sort_by(|a, b| a.ratio.partial_cmp(&b.ratio).unwrap());
    }
    fn sort_by_pallet(&mut self) {
        // TODO...
    }
}
