#[derive(Debug)]
pub struct OverviewTable<'a> {
    inner: Vec<TableEntry<'a>>,
}

#[derive(Debug)]
pub(crate) struct TableEntry<'a> {
    pub pallet: &'a str,
    pub extrinsic: &'a str,
    pub avg_extrinsic_time: f64,
    pub avg_storage_root_time: f64,
    pub ratio: f64,
    pub percentage: f64,
}

impl<'a> OverviewTable<'a> {
    pub fn new() -> Self {
        OverviewTable { inner: Vec::new() }
    }
    pub(crate) fn push(&mut self, entry: TableEntry<'a>) {
        self.inner.push(entry);
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
    pub fn list(&self) -> Vec<(&str, &str, f64, f64, f64, f64)> {
        self.inner
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

#[derive(Debug)]
pub struct StepOverviewTable<'a> {
    inner: Vec<StepTableEntry<'a>>
}

#[derive(Debug, Default)]
pub(crate) struct StepTableEntry<'a> {
    pub pallet: &'a str,
    pub extrinsic: &'a str,
    pub steps: Vec<SingleStep<'a>>,
}

#[derive(Debug)]
pub(crate) struct SingleStep<'a> {
    pub input_vars: &'a Vec<u64>,
    pub avg_extrinsic_time: f64,
    pub avg_storage_root_time: f64,
    pub percentage: f64,
}

impl<'a> StepOverviewTable<'a> {
    pub fn new() -> Self {
        StepOverviewTable { inner: Vec::new() }
    }
    pub(crate) fn push(&mut self, entry: StepTableEntry<'a>) {
        self.inner.push(entry);
    }
}
