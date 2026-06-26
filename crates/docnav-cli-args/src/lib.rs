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
    let mut state = LooseArgScanState::default();
    while state.has_next(args) {
        scan_next_arg(args, config, &mut state)?;
    }
    Ok(state.finish())
}

#[derive(Default)]
struct LooseArgScanState {
    retained_args: Vec<String>,
    ignored: Vec<IgnoredArg>,
    positional_count: usize,
    index: usize,
}

impl LooseArgScanState {
    fn has_next(&self, args: &[String]) -> bool {
        self.index < args.len()
    }

    fn finish(self) -> LooseArgScanResult {
        LooseArgScanResult {
            retained_args: self.retained_args,
            ignored: self.ignored,
        }
    }
}

fn scan_next_arg(
    args: &[String],
    config: &LooseArgScan<'_>,
    state: &mut LooseArgScanState,
) -> Result<(), MissingValue> {
    let token = &args[state.index];
    let (flag_token, _inline_value) = split_equals(token);

    if is_known_switch_flag(config, token) {
        state.retained_args.push(token.clone());
        state.index += 1;
    } else if let Some(flag) = known_value_flag(config.known_value_flags, flag_token) {
        scan_value_flag_arg(args, config, state, flag, flag_token)?;
    } else if is_long_flag(token) {
        state.ignored.push(IgnoredArg::UnknownFlag {
            token: token.clone(),
        });
        state.index += 1;
    } else {
        scan_positional_arg(args, config, state);
    }

    Ok(())
}

fn is_known_switch_flag(config: &LooseArgScan<'_>, token: &str) -> bool {
    config.known_switch_flags.contains(&token)
}

fn scan_value_flag_arg(
    args: &[String],
    config: &LooseArgScan<'_>,
    state: &mut LooseArgScanState,
    flag: KnownValueFlag<'_>,
    flag_token: &str,
) -> Result<(), MissingValue> {
    if flag.used {
        push_retained_value_arg(&mut state.retained_args, args, &mut state.index, flag_token)
    } else {
        let token = args[state.index].clone();
        let value = ignored_value(args, &mut state.index, flag_token)?;
        state.ignored.push(IgnoredArg::UnusedValueFlag {
            flag: token,
            value,
            command: config.command.to_owned(),
        });
        Ok(())
    }
}

fn scan_positional_arg(args: &[String], config: &LooseArgScan<'_>, state: &mut LooseArgScanState) {
    let token = &args[state.index];
    if state.positional_count < config.positional_limit {
        state.retained_args.push(token.clone());
    } else {
        state.ignored.push(IgnoredArg::ExtraPositional {
            token: token.clone(),
        });
    }
    state.positional_count += 1;
    state.index += 1;
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
mod tests;
