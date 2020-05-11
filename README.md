# Substrate benchmark reviewer

The `bench-review` utility creates overview tables of the substrate runtime benchmarks, allowing for further inspection and adjustments of benchmarking results.

## Install

`cargo install --git https://github.com/w3f/substrate-benchmark-reviewer.git`

## Usage

```bash
$ bench-review [SUBCOMMAND] [PATH] [--csv] [--skip-warnings]
```

`bench-review` will print warnings if it reads files which are invalid. Those warnings can be suppressed with the `--skip-warnings` flag. A `--csv` flag is also supported.

### per-extrinsic
Calculates the average extrinsic and storage root execution times **of each extrinsic**. Additionally, each extrinsic displays the ratio of the extrinsic execution time between the fastest benchmarking result and its own, including the increase in percentage.

```bash
$ bench-review per-extrinsic /path/to/results

+-----------+---------------------------+----------------+---------------+----------------+----------------+
|  Pallet   |         Extrinsic         | Avg. Extrinsic | Avg. Storage  | Extrinsic Time | Extrinsic Time |
|           |                           |      Time      |   Root Time   |  Ratio (1:x)   |    Increase    |
+-----------+---------------------------+----------------+---------------+----------------+----------------+
| staking   | set_validator_count       | 4320.8182      | 30835.0455    | 1              | 0              |
+-----------+---------------------------+----------------+---------------+----------------+----------------+
| utility   | as_sub                    | 4495.0182      | 3525.3364     | 1.0403         | 4.0316         |
+-----------+---------------------------+----------------+---------------+----------------+----------------+
| staking   | force_new_era_always      | 4558.65        | 31691.4       | 1.055          | 5.5043         |
+-----------+---------------------------+----------------+---------------+----------------+----------------+
| staking   | force_new_era             | 4624.2         | 33759         | 1.0702         | 7.0214         |
+-----------+---------------------------+----------------+---------------+----------------+----------------+
| ...       | ...                       | ...            | ...           | ...            | ...            |
+-----------+---------------------------+----------------+---------------+----------------+----------------+
```

### per-step
The benchmarks can contain multiple executions of the same input variables ("repeats"). This review calculates the average extrinsic and storage root execution time **of each step** and displays the ratio of the extrinsic execution time between the fastest result (from the same extrinsic) and its own, including the increase in percentage. This review reveals which inputs significantly increase execution time.

```bash
$ bench-review per-step /path/to/results

+-----------+---------------------------+---------------+-----------------+----------------+----------------+----------------+-------------------+
|  Pallet   |         Extrinsic         |   Variables   | Avg. Extrinsic  |  Avg. Storage  | Extrinsic Time | Extrinsic Time | Storage Root Time |
|           |                           |               |      Time       |   Root Time    |  Ratio (1:x)   |    Increase    |     Increase      |
+-----------+---------------------------+---------------+-----------------+----------------+----------------+----------------+-------------------+
| balances  | set_balance               | 1, 1000       | 103719          | 74726.5        | 1.1081         | 10.814         | 17.2484           |
+-----------+---------------------------+---------------+-----------------+----------------+----------------+----------------+-------------------+
| balances  | set_balance               | 1000, 992     | 98291.6         | 69878.7        | 1.0502         | 5.0153         | 9.642             |
+-----------+---------------------------+---------------+-----------------+----------------+----------------+----------------+-------------------+
| balances  | set_balance               | 1000, 695     | 98031.2         | 70534.7        | 1.0474         | 4.7371         | 10.6713           |
+-----------+---------------------------+---------------+-----------------+----------------+----------------+----------------+-------------------+
| balances  | set_balance               | 199, 1000     | 97551           | 69578.4        | 1.0422         | 4.224          | 9.1708            |
+-----------+---------------------------+---------------+-----------------+----------------+----------------+----------------+-------------------+
| ...       | ...                       | ...           | ...             | ...            | ...            | ...            | ...               |
+-----------+---------------------------+---------------+-----------------+----------------+----------------+----------------+-------------------+
```

## TODO

- Also add ratio for storage root time.
- *per-step* -> should probably also contain variable names ("u", "r", etc.).
- A "timeline" review: can compare *per-extrinsic* and *per-step* between days, revealing that some changes in the code have increased (or decreased) execution time.
- Document source code some more.
- Automate "timeline", build a notification service in case something looks off.
