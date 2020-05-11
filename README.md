# Substrate benchmark reviewer

The `bench-review` utility creates overview tables of the substrate runtime benchmarks, allowing for further inspection and adjustments of benchmarking results.

## Install

`cargo install --git https://github.com/w3f/substrate-benchmark-reviewer.git`

## Usage

Three reviews are supported:

- *ratio*: Calculates the average extrinsic and storage root execution times and displays those in a table. Additionally, each extrinsic displays the ratio of the extrinsic execution time between the fastest benchmark result and its own, including the increase in percentage.
- *step*: The benchmarks can contain multiple execution of the same input variables ("repeats"). This review calculates the average extrinsic and storage root execution time of each step and displays the increase from the fastest result to its own in percentage. This review reveals which inputs significantly increase execution time.

## TODO

- Also add ratio for storage root time.
- *step* -> should probably also contain variable names ("u", "r", etc.).
- A "timeline" review: can compare *ratio* and *step* between days, revealing that some changes in the code have increased (or decreased) execution time.
- Document source code some more.
- Automate "timeline", build a notification service in case something looks off.
