pub mod file_collector;
mod parser;
pub mod tables;

pub use file_collector::{FileCollector, FileContent};
use tables::{RatioTable, RatioTableEntry, StepIncr, StepIncrTable, StepIncrTableEntry};

use std::cmp::Ordering;
use failure::Error;

#[macro_use]
extern crate failure;

#[derive(Debug, Default)]
pub struct ExtrinsicResult {
    pallet: String,
    extrinsic: String,
    steps: usize,
    repeats: usize,
    input_var_names: Vec<String>,
    steps_repeats: Vec<StepRepeatEntry>,
}

#[derive(Debug, Default)]
struct StepRepeatEntry {
    input_vars: Vec<u64>,
    extrinsic_time: u64,
    storage_root_time: u64,
}

/// Convenience trait. Round based on the specified
/// number of digits.
///
/// # Example
/// ```ignore
/// let num: f64 = 15.123456;
/// assert_eq!(15.1235, num.round_by(4));
/// ```
trait RoundBy {
    fn round_by(&self, by: i32) -> Self;
}

impl RoundBy for f64 {
    fn round_by(&self, by: i32) -> Self {
        let precision = 10.0_f64.powi(by);
        (self * precision).round() / precision
    }
}

/// Convenience trait. Calculate average of `count` items.
///
/// # Example
/// ```ignore
/// let total: u64 = 20;
/// assert_eq!(5.0, total.calc_average(4));
/// ```
trait CalculateAverage {
    fn calc_average(&self, count: usize) -> f64;
}

impl CalculateAverage for u64 {
    fn calc_average(&self, count: usize) -> f64 {
        *self as f64 / count as f64
    }
}

impl ExtrinsicResult {
    // TODO: Ensure each generated average uses the same amount
    // of entries as the data it gets compared to.
    // TODO: Ensure length is never 0.
    fn average_extrinsic_time(&self) -> f64 {
        self.steps_repeats
            .iter()
            .map(|e| e.extrinsic_time)
            .fold(0, |acc, num| acc + num)
            .calc_average(self.steps_repeats.len())
    }
    // TODO: Ensure each generated average uses the same amount
    // of entries as the data it gets compared to.
    // TODO: Ensure length is never 0.
    fn average_storage_root_time(&self) -> f64 {
        self.steps_repeats
            .iter()
            .map(|e| e.storage_root_time)
            .fold(0, |acc, num| acc + num)
            .calc_average(self.steps_repeats.len())
    }
}

#[derive(Debug, Fail)]
enum ExtrinsicCollectionError {
    #[fail(display = "collection does not contain any results")]
    EmptyResults,
}

use ExtrinsicCollectionError::*;

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
    pub fn generate_overview_table(&self) -> Result<RatioTable, Error> {
        // find base (lowest value)
        // TODO: Handle unwrap
        let base = self
            .results
            .iter()
            .min_by(|a, b| {
                a.average_extrinsic_time()
                    .partial_cmp(&b.average_extrinsic_time())
                    // can occur if there's only one entry
                    .unwrap_or(Ordering::Equal)
            })
            .ok_or(EmptyResults)?
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

        Ok(table)
    }
    pub fn generate_step_table(&self) -> Result<StepIncrTable, Error> {
        use std::collections::HashMap;

        // Signature: (pallet, extrinsic) -> ((input vars) -> (count, extrinsic time, storage root time))
        let mut db: HashMap<(&str, &str), HashMap<&Vec<u64>, (usize, u64, u64)>> = HashMap::new();

        // For each extrinsic result...
        for result in &self.results {
            // ... and for each of its steps/repeats...
            for step in &result.steps_repeats {
                // ... create an entry...
                db.entry((&result.pallet, &result.extrinsic))
                    .and_modify(|sub_map| {
                        // ... and add the measured times of each repeat to the current value,
                        // identified by the step (input vars). Additionally, track the count
                        // of measurements that were added, in order to calculate the average
                        // later on.
                        sub_map
                            .entry(&step.input_vars)
                            .and_modify(|(count, extrinsic_time, storage_root_time)| {
                                *count += 1;
                                *extrinsic_time += step.extrinsic_time;
                                *storage_root_time += step.storage_root_time;
                            })
                            .or_insert((1, step.extrinsic_time, step.storage_root_time));
                    })
                    .or_insert_with(|| {
                        let mut hm = HashMap::new();
                        hm.insert(
                            &step.input_vars,
                            (1, step.extrinsic_time, step.storage_root_time),
                        );
                        hm
                    });
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
                    avg_extrinsic_time: extrinsic_time.calc_average(count).round_by(4),
                    avg_storage_root_time: storage_root_time.calc_average(count).round_by(4),
                    extrinsic_incr_percentage: 0.0,
                    storage_root_incr_percentage: 0.0,
                })
            }

            // Get the smallest value of extrinsic time measurement.
            let extrinsic_base = new_entry
                .step_incrs
                .iter()
                .min_by(|a, b| {
                    a.avg_extrinsic_time
                        .partial_cmp(&b.avg_extrinsic_time)
                        // can occur if there's only one entry
                        .unwrap_or(Ordering::Equal)
                })
                .ok_or(EmptyResults)?
                .avg_extrinsic_time;

            // Get the smallest value of storage root measurement.
            let storage_root_base = new_entry
                .step_incrs
                .iter()
                .min_by(|a, b| {
                    a.avg_storage_root_time
                        .partial_cmp(&b.avg_storage_root_time)
                        // can occur if there's only one entry
                        .unwrap_or(Ordering::Equal)
                })
                .ok_or(EmptyResults)?
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

        Ok(table)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_average() {
        let total = 20;
        assert_eq!(5.0, total.calc_average(4));
        assert_eq!(4.0, total.calc_average(5));
        assert_eq!(0.4, total.calc_average(50));
    }
}
