use docnav_protocol::{positive_result, Operation, Options, PositiveInteger};
use serde_json::Value;

use super::output::DirectOutputMode;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeOptionValueSpec {
    IntegerRange { min: u64, max: u64 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeOptionDefault {
    Integer(u64),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeOptionSpec {
    pub flag: &'static str,
    pub option_key: &'static str,
    pub operations: &'static [Operation],
    pub value: NativeOptionValueSpec,
    pub default: Option<NativeOptionDefault>,
}

#[derive(Clone, Debug)]
pub(crate) struct DirectOperationOptions {
    pub path: String,
    pub page: PositiveInteger,
    pub limit_chars: PositiveInteger,
    pub output: DirectOutputMode,
    pub ref_id: Option<String>,
    pub query: Option<String>,
    native_options: Options,
}

impl DirectOperationOptions {
    pub(crate) fn protocol_options(&self) -> Option<Options> {
        if self.native_options.is_empty() {
            None
        } else {
            Some(self.native_options.clone())
        }
    }
}

pub(crate) fn parse_protocol_only_output(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Ok(());
    }
    if args.len() == 2 && args[0] == "--output" && args[1] == "protocol-json" {
        return Ok(());
    }
    Err("only --output protocol-json is supported for this command".to_owned())
}

pub(crate) fn parse_probe(args: &[String]) -> Result<String, String> {
    let Some(path) = args.first() else {
        return Err("probe requires <path>".to_owned());
    };
    if is_flag(path) {
        return Err("probe requires <path>".to_owned());
    }
    parse_protocol_only_output(&args[1..])?;
    Ok(path.clone())
}

pub(crate) fn parse_operation_options(
    operation: Operation,
    args: &[String],
    default_limit_chars: u32,
    native_options: &[NativeOptionSpec],
) -> Result<DirectOperationOptions, String> {
    let Some(path) = args.first() else {
        return Err(format!("{operation} requires <path>"));
    };
    if is_flag(path) {
        return Err(format!("{operation} requires <path>"));
    }

    let mut options = DirectOperationOptions {
        path: path.clone(),
        page: positive_result(1).expect("static positive integer"),
        limit_chars: positive_result(default_limit_chars).expect("static positive integer"),
        output: DirectOutputMode::Text,
        ref_id: None,
        query: None,
        native_options: default_native_options(operation, native_options),
    };

    let mut index = 1;
    while index < args.len() {
        let flag = args[index].as_str();
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag {
            "--page" if operation != Operation::Info => {
                options.page = parse_positive(value, "--page")?;
            }
            "--limit-chars" if operation != Operation::Info => {
                options.limit_chars = parse_positive(value, "--limit-chars")?;
            }
            "--ref" if operation == Operation::Read => {
                if value.is_empty() {
                    return Err("--ref must not be empty".to_owned());
                }
                options.ref_id = Some(value.clone());
            }
            "--query" if operation == Operation::Find => {
                if value.is_empty() {
                    return Err("--query must not be empty".to_owned());
                }
                options.query = Some(value.clone());
            }
            "--output" => {
                options.output = parse_output(value)?;
            }
            _ => {
                let Some(spec) = native_options
                    .iter()
                    .find(|spec| spec.flag == flag && spec.supports(operation))
                else {
                    return Err(format!("unknown or unsupported flag {flag}"));
                };
                options
                    .native_options
                    .insert(spec.option_key.to_owned(), spec.parse_value(value)?);
            }
        }
        index += 2;
    }

    Ok(options)
}

impl NativeOptionSpec {
    fn supports(&self, operation: Operation) -> bool {
        self.operations.contains(&operation)
    }

    fn parse_value(&self, value: &str) -> Result<Value, String> {
        match self.value {
            NativeOptionValueSpec::IntegerRange { min, max } => {
                let parsed = value
                    .parse::<u64>()
                    .map_err(|_| integer_range_error(self.flag, min, max))?;
                if parsed < min || parsed > max {
                    return Err(integer_range_error(self.flag, min, max));
                }
                Ok(Value::from(parsed))
            }
        }
    }
}

fn default_native_options(operation: Operation, specs: &[NativeOptionSpec]) -> Options {
    let mut options = Options::new();
    for spec in specs.iter().filter(|spec| spec.supports(operation)) {
        let Some(default) = spec.default else {
            continue;
        };
        let value = match default {
            NativeOptionDefault::Integer(value) => Value::from(value),
        };
        options.insert(spec.option_key.to_owned(), value);
    }
    options
}

fn parse_output(value: &str) -> Result<DirectOutputMode, String> {
    match value {
        "text" => Ok(DirectOutputMode::Text),
        "readable-json" => Ok(DirectOutputMode::ReadableJson),
        "protocol-json" => Ok(DirectOutputMode::ProtocolJson),
        _ => Err(format!("invalid --output {value:?}")),
    }
}

fn parse_positive(value: &str, flag: &str) -> Result<PositiveInteger, String> {
    let raw = value
        .parse::<u32>()
        .map_err(|_| format!("{flag} must be a positive integer"))?;
    positive_result(raw).map_err(|_| format!("{flag} must be a positive integer"))
}

fn integer_range_error(flag: &str, min: u64, max: u64) -> String {
    format!("{flag} must be an integer from {min} to {max}")
}

fn is_flag(value: &str) -> bool {
    value.starts_with("--")
}
