use super::{file_collector::FileContent, ExtrinsicResult, StepRepeatEntry};

use failure::Error;

#[derive(Debug, Fail)]
enum AnalyserError {
    #[fail(display = "header value of the benchmark result is missing")]
    MissingHeader,
    #[fail(display = "header value of the benchmark result is invalid")]
    InvalidHeader,
}

use self::AnalyserError::*;

/// Parses the header of the result file. This function has slightly stricter requirements.
///
/// Example:
/// ```txt
/// Pallet: "balances", Extrinsic: "set_balance", Lowest values: [], Highest values: [], Steps: [10], Repeat: 10
/// u,e,extrinsic_time,storage_root_time
/// ```
#[rustfmt::skip]
pub(crate) fn parse_header(content: &FileContent) -> Result<ExtrinsicResult, Error> {
    let mut extrinsic_result = ExtrinsicResult::default();

    let lines: Vec<&str> = content.0.lines().take(2).collect();

    // Parse the first line
    {
        let parts: Vec<&str> = lines
            .get(0)
            .ok_or(MissingHeader)?
            .split_whitespace()
            .collect();

        // Length is checked here, so directly indexing
        // the vector after this is safe.
        check(|| parts.len() == 14)?;

        // Parse pallet name
        extrinsic_result.pallet =
            check_requirements(parts[0], parts[1], "Pallet:", "\"", "\",")?;

        // Parse extrinsic name
        extrinsic_result.extrinsic =
            check_requirements(parts[2], parts[3], "Extrinsic:", "\"", "\",")?;

        // Parse steps amount
        extrinsic_result.steps =
            check_requirements(parts[10], parts[11], "Steps:", "[", "],")?
                .parse::<usize>()
                .map_err(|_| InvalidHeader)?;

        // Parse repeat amount. The amount does not have brackets around it,
        // probably skipped by accident. Generally not an issue, just a
        // small inconsistency.
        extrinsic_result.repeats =
            check_requirements(parts[12], parts[13], "Repeat:", "", "")?
                .parse::<usize>()
                .map_err(|_| InvalidHeader)?;
    }

    // Parse second line
    {
        let parts: Vec<&str> = lines
            .get(1)
            .ok_or(MissingHeader)?
            .split(",")
            .collect();

        check(|| parts.len() > 2)?;
        check(|| parts.contains(&"extrinsic_time"))?;
        check(|| parts.contains(&"storage_root_time"))?;

        let mut offset = 0;
        for part in &parts {
            if part == &"extrinsic_time" {
                break;
            }

            // E.g. part = `u`
            check(|| part.len() == 1)?;

            offset += 1;
        }

        parts
            .iter()
            .take(offset)
            .for_each(|var| {
                extrinsic_result.input_var_names.push(var.to_string())
            });
    }

    Ok(extrinsic_result)
}

pub(crate) fn parse_body(
    content: &FileContent,
    expected_len: usize,
) -> Result<Vec<StepRepeatEntry>, Error> {
    let mut coll = Vec::new();
    let lines: Vec<&str> = content.0.lines().skip(2).collect();

    for line in lines {
        let parts: Vec<&str> = line.split(",").collect();

        // Must have the expected length:
        // -> variables + "extrinsic_time" + "storage_root_time"
        if parts.len() != expected_len || parts.len() < 2 {
            break;
        }

        // All parts must be numeric
        if parts.iter().all(|p| !p.parse::<usize>().is_ok()) {
            break;
        }

        let mut repeat_entry = StepRepeatEntry::default();

        // Fill in the data. The length and conversion validity is checked above,
        // so directly indexing and unwrapping is safe here.
        let temp: Vec<&&str> = parts.iter().rev().take(2).collect();
        repeat_entry.storage_root_time = temp[0].parse::<u64>().unwrap();
        repeat_entry.extrinsic_time = temp[1].parse::<u64>().unwrap();
        repeat_entry.input_vars = parts
            .iter()
            .take(expected_len - 2)
            .map(|p| p.parse::<u64>().unwrap())
            .collect();

        coll.push(repeat_entry);
    }

    Ok(coll)
}

/// Checks the requirements of the header (benchmark description) key and value.
/// Returns the parsed value. Parameters:
/// - input key
/// - input value
/// - key must equal ...
/// - value starts with ... (gets removed)
/// - value ends with ...
fn check_requirements(
    input_key: &str,
    input_val: &str,
    key_name: &str,
    val_start: &str,
    val_end: &str,
) -> Result<String, Error> {
    // E.g. ... == "Pallet:"
    check(|| input_key == key_name)?;

    // E.g. ... starts with `"` and ends with `",`
    check(|| input_val.starts_with(val_start) && input_val.ends_with(val_end))?;

    // E.g. from `"balances",` -> `balances`
    Ok(String::from(input_val)
        .replace(val_start, "")
        .replace(val_end, "")
        .replace(",", "")) // Remove any tangling comma
}

fn check<F>(func: F) -> Result<(), Error>
where
    F: Fn() -> bool,
{
    if !func() {
        return Err(InvalidHeader.into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FileContent;

    #[test]
    fn test_parse_header() {
        let test_data = [
            (
                // Input (str1 + str2)
                (
                    r#"Pallet: "balances", Extrinsic: "set_balance", Lowest values: [], Highest values: [], Steps: [10], Repeat: 10"#,
                    r#"u,e,extrinsic_time,storage_root_time"#,
                ),
                // -> output to be tested
                ("balances", "set_balance", 10, 10, vec!["u", "e"]),
            ),
            (
                (
                    r#"Pallet: "democracy", Extrinsic: "delegate", Lowest values: [], Highest values: [], Steps: [10], Repeat: 10"#,
                    r#"r,extrinsic_time,storage_root_time"#,
                ),
                ("democracy", "delegate", 10, 10, vec!["r"]),
            ),
            (
                (
                    r#"Pallet: "democracy", Extrinsic: "proxy_undelegate", Lowest values: [], Highest values: [], Steps: [20], Repeat: 20"#,
                    r#"r,extrinsic_time,storage_root_time"#,
                ),
                ("democracy", "proxy_undelegate", 20, 20, vec!["r"]),
            ),
            (
                (
                    r#"Pallet: "identity", Extrinsic: "cancel_request", Lowest values: [], Highest values: [], Steps: [20], Repeat: 20"#,
                    r#"r,x,extrinsic_time,storage_root_time"#,
                ),
                ("identity", "cancel_request", 20, 20, vec!["r", "x"]),
            ),
        ];

        for ((str1, str2), output) in &test_data {
            let content = FileContent(format!("{}\n{}", str1, str2));
            let res = parse_header(&content).unwrap();
            assert_eq!(res.pallet, output.0);
            assert_eq!(res.extrinsic, output.1);
            assert_eq!(res.steps, output.2);
            assert_eq!(res.repeats, output.3);

            let mut counter = 0;
            for var in &output.4 {
                assert_eq!(&res.input_var_names[counter], var);
                counter += 1;
            }
        }
    }

    #[test]
    fn test_parse_body() {
        let test_data = [
            (
                ("header1 ... (skipped)\n\
                    header2 ... (skpped)\n\
                    1,100,409567,85176\n\
                    1,100,404202,95485\n\
                    1,100,436160,89604\n\
                    1,100,443911,106889\n\
                    1,100,441193,93017\n\
                    1,100,419524,94390"),
                vec![
                    vec![1, 100, 409567, 85176],
                    vec![1, 100, 404202, 95485],
                    vec![1, 100, 436160, 89604],
                    vec![1, 100, 443911, 106889],
                    vec![1, 100, 441193, 93017],
                    vec![1, 100, 419524, 94390],
                ],
            ),
            (
                ("header1 ... (skipped)\n\
                    header2 ... (skpped)\n\
                    20,100,416389,88954\n\
                    20,100,411389,88545\n\
                    20,100,410049,88203\n\
                    20,1,127985,70742\n\
                    20,1,133206,72225\n\
                    20,1,140801,71239\n\
                    Median Slopes Analysis\n\
                    ========\n\
                    \n\
                    Model:\n\
                    Time ~=    188.8\n\
                        + r    25.97\n\
                                µs\n\
                    \n\
                    Min Squares Analysis\n\
                    ========"),
                vec![
                    vec![20, 100, 416389, 88954],
                    vec![20, 100, 411389, 88545],
                    vec![20, 100, 410049, 88203],
                    vec![20, 1, 127985, 70742],
                    vec![20, 1, 133206, 72225],
                    vec![20, 1, 140801, 71239],
                ],
            ),
            (
                ("header1 ... (skipped)\n\
                    header2 ... (skpped)\n\
                    50,87726,90501\n\
                    50,229949,111762\n\
                    60,96437,87721\n\
                    60,87618,83273\n\
                    60,97325,232905\n\
                    60,117408,116346"),
                vec![
                    vec![50, 87726, 90501],
                    vec![50, 229949, 111762],
                    vec![60, 96437, 87721],
                    vec![60, 87618, 83273],
                    vec![60, 97325, 232905],
                    vec![60, 117408, 116346],
                ],
            ),
        ];

        for (content, output) in &test_data {
            let content = FileContent(String::from(*content));
            let expected_len = output[0].len();
            let res = parse_body(&content, expected_len).unwrap();

            let mut counter = 0;
            for entry in res {
                // println!("{:?}", entry);

                // Hint: the other two values are `extrinsic_time`
                // and `storage_root_time`
                assert_eq!(entry.input_vars.len(), expected_len - 2);

                let current = &output[counter];
                for i in 0..expected_len - 2 {
                    assert_eq!(entry.input_vars[i], current[i]);
                }

                assert_eq!(entry.extrinsic_time, current[expected_len - 2]);
                assert_eq!(entry.storage_root_time, current[expected_len - 1]);
                counter += 1;
            }
        }
    }
}
