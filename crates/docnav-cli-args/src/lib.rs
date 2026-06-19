#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KnownValueFlag<'a> {
    pub flag: &'a str,
    pub used: bool,
}

impl<'a> KnownValueFlag<'a> {
    pub const fn used(flag: &'a str) -> Self {
        Self { flag, used: true }
    }

    pub const fn unused(flag: &'a str) -> Self {
        Self { flag, used: false }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LooseArgScan<'a> {
    pub command: &'a str,
    pub positional_limit: usize,
    pub known_value_flags: &'a [KnownValueFlag<'a>],
    pub known_switch_flags: &'a [&'a str],
}

impl<'a> LooseArgScan<'a> {
    pub const fn new(
        command: &'a str,
        positional_limit: usize,
        known_value_flags: &'a [KnownValueFlag<'a>],
    ) -> Self {
        Self {
            command,
            positional_limit,
            known_value_flags,
            known_switch_flags: &[],
        }
    }

    pub const fn with_known_switch_flags(mut self, known_switch_flags: &'a [&'a str]) -> Self {
        self.known_switch_flags = known_switch_flags;
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LooseArgScanResult {
    pub retained_args: Vec<String>,
    pub ignored: Vec<IgnoredArg>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IgnoredArg {
    UnknownFlag {
        token: String,
    },
    ExtraPositional {
        token: String,
    },
    UnusedValueFlag {
        flag: String,
        value: Option<String>,
        command: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MissingValue {
    flag: String,
}

impl MissingValue {
    pub fn flag(&self) -> &str {
        &self.flag
    }
}

pub fn scan_loose_args(
    args: &[String],
    config: &LooseArgScan<'_>,
) -> Result<LooseArgScanResult, MissingValue> {
    let mut retained_args = Vec::new();
    let mut ignored = Vec::new();
    let mut positional_count = 0;
    let mut index = 0;

    while index < args.len() {
        let token = &args[index];
        let (flag_token, inline_value) = split_equals(token);
        if config
            .known_switch_flags
            .iter()
            .any(|known| *known == token)
        {
            retained_args.push(token.clone());
            index += 1;
        } else if let Some(flag) = known_value_flag(config.known_value_flags, flag_token) {
            if flag.used {
                push_retained_value_arg(&mut retained_args, args, &mut index, flag_token)?;
            } else {
                let value = ignored_value(args, &mut index, flag_token)?;
                ignored.push(IgnoredArg::UnusedValueFlag {
                    flag: token.clone(),
                    value,
                    command: config.command.to_owned(),
                });
            }
        } else if is_long_flag(token) {
            ignored.push(IgnoredArg::UnknownFlag {
                token: token.clone(),
            });
            index += 1;
        } else {
            if positional_count < config.positional_limit {
                retained_args.push(token.clone());
            } else {
                ignored.push(IgnoredArg::ExtraPositional {
                    token: token.clone(),
                });
            }
            positional_count += 1;
            index += 1;
        }

        let _ = inline_value;
    }

    Ok(LooseArgScanResult {
        retained_args,
        ignored,
    })
}

fn known_value_flag<'a>(
    flags: &'a [KnownValueFlag<'a>],
    token: &str,
) -> Option<KnownValueFlag<'a>> {
    flags.iter().copied().find(|flag| flag.flag == token)
}

fn push_retained_value_arg(
    retained_args: &mut Vec<String>,
    args: &[String],
    index: &mut usize,
    flag: &str,
) -> Result<(), MissingValue> {
    let token = &args[*index];
    if token.split_once('=').is_some() {
        retained_args.push(token.clone());
        *index += 1;
        return Ok(());
    }

    let value = args
        .get(*index + 1)
        .ok_or_else(|| missing_value(flag))?
        .clone();
    retained_args.push(token.clone());
    retained_args.push(value);
    *index += 2;
    Ok(())
}

fn ignored_value(
    args: &[String],
    index: &mut usize,
    flag: &str,
) -> Result<Option<String>, MissingValue> {
    let token = &args[*index];
    if let Some((_flag, value)) = token.split_once('=') {
        *index += 1;
        return Ok(Some(value.to_owned()));
    }

    let value = args
        .get(*index + 1)
        .ok_or_else(|| missing_value(flag))?
        .clone();
    *index += 2;
    Ok(Some(value))
}

fn missing_value(flag: &str) -> MissingValue {
    MissingValue {
        flag: flag.to_owned(),
    }
}

fn is_long_flag(value: &str) -> bool {
    value.starts_with("--")
}

fn split_equals(token: &str) -> (&str, Option<&str>) {
    token
        .split_once('=')
        .map_or((token, None), |(flag, value)| (flag, Some(value)))
}

#[cfg(test)]
mod tests {
    // @case WB-CLIARGS-COMPAT-001
    use super::*;

    #[test]
    fn unknown_flag_does_not_consume_following_positional() {
        let result = scan(&["--future", "doc.md"], 1, &[]).unwrap();

        assert_eq!(result.retained_args, ["doc.md"]);
        assert_eq!(
            result.ignored,
            [IgnoredArg::UnknownFlag {
                token: "--future".to_owned()
            }]
        );
    }

    #[test]
    fn used_value_flag_is_retained_and_consumes_value() {
        let flags = [KnownValueFlag::used("--ref")];
        let result = scan(&["doc.md", "--ref", "--future-value"], 1, &flags).unwrap();

        assert_eq!(result.retained_args, ["doc.md", "--ref", "--future-value"]);
        assert!(result.ignored.is_empty());
    }

    #[test]
    fn unused_value_flag_records_fact_without_validating_value() {
        let flags = [KnownValueFlag::unused("--page")];
        let result = scan(&["doc.md", "--page", "nope"], 1, &flags).unwrap();

        assert_eq!(result.retained_args, ["doc.md"]);
        assert_eq!(
            result.ignored,
            [IgnoredArg::UnusedValueFlag {
                flag: "--page".to_owned(),
                value: Some("nope".to_owned()),
                command: "info".to_owned()
            }]
        );
    }

    #[test]
    fn unused_value_flag_requires_a_value() {
        let flags = [KnownValueFlag::unused("--page")];
        let error = scan(&["doc.md", "--page"], 1, &flags).unwrap_err();

        assert_eq!(error.flag(), "--page");
    }

    #[test]
    fn switch_flags_are_retained_without_consuming_value() {
        let config = LooseArgScan::new("config get", 1, &[]).with_known_switch_flags(&["--user"]);
        let result = scan_loose_args(&args(&["--user", "key"]), &config).unwrap();

        assert_eq!(result.retained_args, ["--user", "key"]);
        assert!(result.ignored.is_empty());
    }

    fn scan(
        values: &[&str],
        positional_limit: usize,
        known_value_flags: &[KnownValueFlag<'_>],
    ) -> Result<LooseArgScanResult, MissingValue> {
        let config = LooseArgScan::new("info", positional_limit, known_value_flags);
        scan_loose_args(&args(values), &config)
    }

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }
}
