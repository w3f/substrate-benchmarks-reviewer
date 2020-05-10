pub mod file_collector;
mod parser;
pub mod tables;

pub use file_collector::{FileCollector, FileContent};
use tables::{RatioTable, RatioTableEntry, StepIncr, StepIncrTable, StepIncrTableEntry};

use std::cmp::Ordering;

#[macro_use]
extern crate failure;

#[derive(Debug, Default)]
pub struct ExtrinsicResult {
    pallet: String,
    extrinsic: String,
    steps: usize,
    repeats: usize,
    input_var_names: Vec<String>,
    repeat_entries: Vec<StepRepeatEntry>,
}

#[derive(Debug, Default)]
struct StepRepeatEntry {
    input_vars: Vec<u64>,
    extrinsic_time: u64,
    storage_root_time: u64,
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

trait CalculateAverage {
    fn calc_average(&self, count: Option<usize>) -> f64;
}

impl CalculateAverage for Vec<u64> {
    fn calc_average(&self, _: Option<usize>) -> f64 {
        let mut total = 0;

        for num in self {
            total += num;
        }

        total as f64 / self.len() as f64
    }
}

impl CalculateAverage for u64 {
    fn calc_average(&self, count: Option<usize>) -> f64 {
        if let Some(count) = count {
            *self as f64 / count as f64
        } else {
            *self as f64
        }
    }
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

        total as f64 / self.repeat_entries.len() as f64
    }
    // TODO: Ensure each generated average uses the same amount
    // of entries as the data it gets compared to.
    // TODO: Ensure length is never 0.
    fn average_storage_root_time(&self) -> f64 {
        let mut total = 0;
        self.repeat_entries
            .iter()
            .for_each(|e| total += e.storage_root_time);

        total as f64 / self.repeat_entries.len() as f64
    }
}

#[derive(Debug)]
pub struct ExtrinsicCollection {
    results: Vec<ExtrinsicResult>,
}

impl ExtrinsicCollection {
    pub fn new() -> Self {
        ExtrinsicCollection {
            results: Vec::new(),
        }
    }
    pub fn push(&mut self, result: ExtrinsicResult) {
        self.results.push(result);
    }
    pub fn generate_overview_table(&self) -> RatioTable {
        // find base (lowest value)
        // TODO: Handle unwrap
        let base = self
            .results
            .iter()
            .min_by(|x, y| {
                x.average_extrinsic_time()
                    .partial_cmp(&y.average_extrinsic_time())
                    // can occur if there's only one entry
                    .unwrap_or(Ordering::Equal)
            })
            .unwrap()
            .average_extrinsic_time();

        let mut table = RatioTable::new();

        self.results.iter().for_each(|result| {
            let avg_time = result.average_extrinsic_time();
            table.push(RatioTableEntry {
                pallet: &result.pallet,
                extrinsic: &result.extrinsic,
                avg_extrinsic_time: avg_time.round_by(4),
                avg_storage_root_time: result.average_storage_root_time().round_by(4),
                ratio: (avg_time / base).round_by(4),
                percentage: ((avg_time / base - 1.0) * 100.0).round_by(4),
            });
        });

        table
    }
    pub fn generate_step_table(&self) -> StepIncrTable {
        use std::collections::HashMap;

        // Signature: (pallet, extrinsic) -> ((input vars) -> (count, extrinsic time, storage root time))
        let mut db: HashMap<(&str, &str), HashMap<&Vec<u64>, (usize, u64, u64)>> = HashMap::new();

        // For each extrinsic result...
        for result in &self.results {
            // ... and for each of its steps...
            for entry in &result.repeat_entries {
                // ... create an entry...
                db.entry((&result.pallet, &result.extrinsic))
                    .and_modify(|sub_map| {
                        // ... and add the measured times of each repeat to the current value,
                        // identified by the step (input vars). Additionally, track the count
                        // of measurements that were added, in order to calculate the average
                        // later on.
                        sub_map
                            .entry(&entry.input_vars)
                            .and_modify(|(count, extrinsic_time, storage_root_time)| {
                                *count += 1;
                                *extrinsic_time += entry.extrinsic_time;
                                *storage_root_time += entry.storage_root_time;
                            })
                            .or_insert((1, entry.extrinsic_time, entry.storage_root_time));
                    })
                    .or_insert(HashMap::new());
            }
        }

        let mut table = StepIncrTable::new();
        // For each extrinsic ...
        for ((pallet, extrinsic), value) in db {
            let mut new_entry = StepIncrTableEntry::default();
            new_entry.pallet = pallet;
            new_entry.extrinsic = extrinsic;

            // ... and for each of its steps...
            for (input_vars, (count, extrinsic_time, storage_root_time)) in value {
                // ... calculate the average. The percentages are filled with zeroes
                // and get adjusted later on, since all averages have to be calculated
                // first.
                new_entry.step_incrs.push(StepIncr {
                    input_vars: input_vars,
                    avg_extrinsic_time: extrinsic_time.calc_average(Some(count)).round_by(4),
                    avg_storage_root_time: storage_root_time.calc_average(Some(count)).round_by(4),
                    extrinsic_incr_percentage: 0.0,
                    storage_root_incr_percentage: 0.0,
                })
            }

            // Get the smallest value of extrinsic time measurement.
            let extrinsic_base = new_entry
                .step_incrs
                .iter()
                .min_by(|x, y| {
                    x.avg_extrinsic_time
                        .partial_cmp(&y.avg_extrinsic_time)
                        // can occur if there's only one entry
                        .unwrap_or(Ordering::Equal)
                })
                .unwrap()
                .avg_extrinsic_time;

            // Get the smallest value of storage root measurement.
            let storage_root_base = new_entry
                .step_incrs
                .iter()
                .min_by(|x, y| {
                    x.avg_storage_root_time
                        .partial_cmp(&y.avg_storage_root_time)
                        // can occur if there's only one entry
                        .unwrap_or(Ordering::Equal)
                })
                .unwrap()
                .avg_storage_root_time;

            // Based on the smallest value, calculate the increase of each step in percentages.
            for entry in &mut new_entry.step_incrs {
                entry.extrinsic_incr_percentage =
                    ((entry.avg_extrinsic_time / extrinsic_base - 1.0) * 100.0).round_by(4);
                entry.storage_root_incr_percentage =
                    ((entry.avg_storage_root_time / storage_root_base - 1.0) * 100.0).round_by(4);
            }

            //percentage: ((avg_time / base - 1.0) * 100.0).round_by(4),

            table.push(new_entry);
        }

        table
    }
}
