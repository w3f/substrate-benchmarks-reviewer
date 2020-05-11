use std::cmp::Ordering;
use std::io::stdout;

#[derive(Debug)]
pub struct PerExtrTable<'a> {
    entries: Vec<PerExtrTableEntry<'a>>,
}

#[derive(Debug)]
pub(crate) struct PerExtrTableEntry<'a> {
    pub pallet: &'a str,
    pub extrinsic: &'a str,
    pub avg_extrinsic_time: f64,
    pub avg_storage_root_time: f64,
    pub ratio: f64,
    pub percentage: f64,
}

impl<'a> PerExtrTable<'a> {
    pub fn new() -> Self {
        PerExtrTable {
            entries: Vec::new(),
        }
    }
    pub(crate) fn push(&mut self, entry: PerExtrTableEntry<'a>) {
        self.entries.push(entry);
    }
    pub fn sort_by_ratio(&mut self) {
        self.entries
            .sort_by(|a, b| a.ratio.partial_cmp(&b.ratio).unwrap_or(Ordering::Equal));
    }
    /// Returns a list of the entries.
    ///
    /// Data ordered as:
    /// - pallet
    /// - extrinsic
    /// - average extrinsic time
    /// - average storage root time
    /// - ratio
    /// - percentage
    ///
    /// # Example output:
    /// ```
    /// vec![
    ///     ("identity", "add_registrar", 1.0, 0.0),
    ///     ("treasury", "tip_new", 1.8363, 83.6271),
    ///     ("balances", "transfer", 2.4501, 145.0108),
    /// ];
    /// ```
    pub fn raw_list(&self) -> Vec<(&str, &str, f64, f64, f64, f64)> {
        self.entries
            .iter()
            .map(|e| {
                (
                    e.pallet,
                    e.extrinsic,
                    e.avg_extrinsic_time,
                    e.avg_storage_root_time,
                    e.ratio,
                    e.percentage,
                )
            })
            .collect()
    }
    fn build_table(&self) -> prettytable::Table {
        let mut table = prettytable::Table::new();

        // Header
        table.add_row(row![
            bc =>
            "Pallet",
            "Extrinsic",
            "Avg. Extrinsic\nTime",
            "Avg. Storage\nRoot Time",
            "Extrinsic Time\nRatio (1:x)",
            "Extrinsic Time\nIncrease"
        ]);

        // Body
        for entry in self.raw_list() {
            table.add_row(row![entry.0, entry.1, entry.2, entry.3, entry.4, entry.5,]);
        }

        table
    }
    pub fn print(&self) {
        self.build_table().printstd();
    }
    pub fn print_csv(&self) {
        self.build_table().to_csv(stdout()).unwrap();
    }
}

#[derive(Debug)]
pub struct StepIncrTable<'a> {
    entries: Vec<StepIncrTableEntry<'a>>,
}

#[derive(Debug, Default)]
pub(crate) struct StepIncrTableEntry<'a> {
    pub pallet: &'a str,
    pub extrinsic: &'a str,
    pub step_incrs: Vec<StepIncr<'a>>,
}

#[derive(Debug)]
pub(crate) struct StepIncr<'a> {
    pub input_vars: &'a Vec<u64>,
    pub avg_extrinsic_time: f64,
    pub avg_storage_root_time: f64,
    pub ratio: f64,
    pub extrinsic_incr_percentage: f64,
    pub storage_root_incr_percentage: f64,
}

impl<'a> StepIncrTable<'a> {
    pub fn new() -> Self {
        StepIncrTable {
            entries: Vec::new(),
        }
    }
    pub(crate) fn push(&mut self, entry: StepIncrTableEntry<'a>) {
        self.entries.push(entry);
    }
    pub fn sort_by_extrinsic_incr_percentage(&mut self) {
        // Sort by increase percentages for each extrinsic
        for entry in &mut self.entries {
            entry.step_incrs.sort_by(|a, b| {
                b.extrinsic_incr_percentage
                    .partial_cmp(&a.extrinsic_incr_percentage)
                    .unwrap_or(Ordering::Equal)
            });
        }

        // Additionally, sort by pallet name
        self.entries.sort_by(|a, b| a.pallet.cmp(b.pallet));
    }
    /// Returns a list of the entries.
    ///
    /// Data ordered as:
    /// - pallet
    /// - extrinsic
    /// - input variables
    /// - average extrinsic time
    /// - average storage root time
    /// - percentage increase of extrinsic time compared to the lowest
    /// - percentage increase of storage root time compared to the lowest
    ///
    /// # Example output:
    /// ```
    /// vec![
    ///     ("democracy", "propose", &vec![19], 135927.0, 106922.0, 1.0796, 7.9605, 27.1982),
    ///     ("democracy", "propose", &vec![10], 130751.0, 89704.8, 1.0385, 3.8495, 6.7160),
    ///     ("democracy", "propose", &vec![1], 125904.3333, 84059.3333, 1.0, 0.0, 0.0),
    /// ];
    /// ```
    pub fn raw_list(&self) -> Vec<(&str, &str, &[u64], f64, f64, f64, f64, f64)> {
        self.entries
            .iter()
            .map(|e| {
                e.step_incrs
                    .iter()
                    .map(|s| {
                        (
                            e.pallet,
                            e.extrinsic,
                            s.input_vars.as_slice(),
                            s.avg_extrinsic_time,
                            s.avg_storage_root_time,
                            s.ratio,
                            s.extrinsic_incr_percentage,
                            s.storage_root_incr_percentage,
                        )
                    })
                    .collect::<Vec<(&str, &str, &[u64], f64, f64, f64, f64, f64)>>()
            })
            .flatten()
            .collect()
    }
    fn build_table(&self) -> prettytable::Table {
        fn display_slice(slice: &[u64]) -> String {
            let mut s = String::new();

            for i in slice {
                s.push_str(&format!("{}, ", i));
            }

            s.pop(); // remove whitespace
            s.pop(); // remove comma
            s
        }

        let mut table = prettytable::Table::new();

        // Header
        table.add_row(row![
            bc =>
            "Pallet",
            "Extrinsic",
            "Variables",
            "Avg. Extrinsic\nTime",
            "Avg. Storage\nRoot Time",
            "Extrinsic Time\nRatio (1:x)",
            "Extrinsic Time\nIncrease",
            "Storage Root Time\nIncrease"
        ]);

        // Body
        for entry in self.raw_list() {
            table.add_row(row![
                entry.0,
                entry.1,
                display_slice(entry.2),
                entry.3,
                entry.4,
                entry.5,
                entry.6,
                entry.7,
            ]);
        }

        table
    }
    pub fn print(&self) {
        self.build_table().printstd();
    }
    pub fn print_csv(&self) {
        self.build_table().to_csv(stdout()).unwrap();
    }
}
