use docnav_protocol::Operation;
use serde_json::Value;

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

impl NativeOptionSpec {
    pub(super) fn supports(&self, operation: Operation) -> bool {
        self.operations.contains(&operation)
    }

    pub(super) fn parse_value(&self, value: &str) -> Result<Value, String> {
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

fn integer_range_error(flag: &str, min: u64, max: u64) -> String {
    format!("{flag} must be an integer from {min} to {max}")
}
