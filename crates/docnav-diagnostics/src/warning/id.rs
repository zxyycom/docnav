use std::fmt;

use serde::Serialize;

use crate::code::ReadableWarningDiagnosticCode;

pub const CLI_ARGV_IGNORED: WarningId =
    WarningId::from_static(ReadableWarningDiagnosticCode::CliArgvIgnored.warning_id());
pub const ADAPTER_CANDIDATE_FAILURE: WarningId =
    WarningId::from_static(ReadableWarningDiagnosticCode::AdapterCandidateFailure.warning_id());
pub const ADAPTER_CONFIG_SOURCE_SKIPPED: WarningId =
    WarningId::from_static(ReadableWarningDiagnosticCode::AdapterConfigSourceSkipped.warning_id());

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct WarningId(WarningIdRepr);

impl WarningId {
    pub const fn from_static(value: &'static str) -> Self {
        Self(WarningIdRepr::Static(value))
    }

    pub fn new(value: impl Into<String>) -> Result<Self, InvalidWarningId> {
        let value = value.into();
        validate_warning_id(&value)?;
        Ok(Self(WarningIdRepr::Owned(value)))
    }

    pub const fn cli_argv_ignored() -> Self {
        CLI_ARGV_IGNORED
    }

    pub const fn adapter_candidate_failure() -> Self {
        ADAPTER_CANDIDATE_FAILURE
    }

    pub const fn adapter_config_source_skipped() -> Self {
        ADAPTER_CONFIG_SOURCE_SKIPPED
    }

    pub fn as_str(&self) -> &str {
        match &self.0 {
            WarningIdRepr::Static(value) => value,
            WarningIdRepr::Owned(value) => value,
        }
    }
}

impl Serialize for WarningId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum WarningIdRepr {
    Static(&'static str),
    Owned(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidWarningId {
    value: String,
}

impl fmt::Display for InvalidWarningId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid warning id {:?}", self.value)
    }
}

impl std::error::Error for InvalidWarningId {}

fn validate_warning_id(value: &str) -> Result<(), InvalidWarningId> {
    let valid = !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_');
    if valid {
        Ok(())
    } else {
        Err(InvalidWarningId {
            value: value.to_owned(),
        })
    }
}
