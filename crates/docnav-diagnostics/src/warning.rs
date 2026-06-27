use std::fmt;
use std::io::{self, Write};

use serde::Serialize;
use serde_json::{Map, Value};

use crate::code::{
    typed_codes, DiagnosticCode, DiagnosticEffect, ReadableWarningDiagnosticCode,
    ReadableWarningDiagnosticMarker,
};
use crate::details::{
    AdapterCandidateDetails, AdapterConfigSourceDetails, CliArgvDetails, DiagnosticDetails,
};
use crate::record::{DiagnosticRecord, DiagnosticRecordDraft, DiagnosticSource};

pub const CLI_ARGV_IGNORED: ReadableWarningDiagnosticCode =
    ReadableWarningDiagnosticCode::CliArgvIgnored;
pub const ADAPTER_CANDIDATE_FAILURE: ReadableWarningDiagnosticCode =
    ReadableWarningDiagnosticCode::AdapterCandidateFailure;
pub const ADAPTER_CONFIG_SOURCE_SKIPPED: ReadableWarningDiagnosticCode =
    ReadableWarningDiagnosticCode::AdapterConfigSourceSkipped;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct WarningProjection {
    #[serde(rename = "id")]
    code: ReadableWarningDiagnosticCode,
    reason: String,
    effect: DiagnosticEffect,
    details: DiagnosticDetails,
}

impl WarningProjection {
    pub fn new<C>(
        reason: impl Into<String>,
        details: C::Details,
    ) -> Result<Self, EmptyWarningReason>
    where
        C: ReadableWarningDiagnosticMarker,
    {
        let reason = reason.into();
        if reason.is_empty() {
            return Err(EmptyWarningReason);
        }
        Ok(Self::from_typed_parts::<C>(reason, details))
    }

    fn from_typed_parts<C>(reason: String, details: C::Details) -> Self
    where
        C: ReadableWarningDiagnosticMarker,
    {
        let code = C::WARNING_CODE;
        Self {
            code,
            reason,
            effect: DiagnosticCode::from(code).default_effect(),
            details: details.into(),
        }
    }

    pub const fn code(&self) -> ReadableWarningDiagnosticCode {
        self.code
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }

    pub const fn effect(&self) -> DiagnosticEffect {
        self.effect
    }

    pub const fn details(&self) -> &DiagnosticDetails {
        &self.details
    }

    pub fn unknown_flag(token: &str) -> Self {
        Self::cli_argv_ignored(vec![token.to_owned()], "unknown CLI flag ignored")
    }

    pub fn extra_positional(token: &str) -> Self {
        Self::cli_argv_ignored(vec![token.to_owned()], "extra positional argument ignored")
    }

    pub fn unused_operation_flag(flag: &str, value: Option<&str>, command: &str) -> Self {
        let mut tokens = vec![flag.to_owned()];
        if let Some(value) = value {
            tokens.push(value.to_owned());
        }
        Self::cli_argv_ignored(tokens, format!("flag is not used by {command} command"))
    }

    pub fn adapter_candidate_failure(
        adapter_id: &str,
        stage: &str,
        code: &str,
        reason: &str,
        preselected: bool,
    ) -> Self {
        let reason = if preselected {
            format!("preselected adapter was not used: {reason}")
        } else {
            format!("adapter candidate was not used: {reason}")
        };
        Self::from_typed_parts::<typed_codes::readable_warning::AdapterCandidateFailure>(
            reason,
            AdapterCandidateDetails::new(
                adapter_id,
                stage,
                code,
                if preselected { Some(true) } else { None },
            ),
        )
    }

    pub fn adapter_config_source_skipped(
        source_level: &str,
        path_origin: &str,
        path: &str,
        reason_code: &str,
    ) -> Self {
        Self::from_typed_parts::<typed_codes::readable_warning::AdapterConfigSourceSkipped>(
            "adapter config source skipped".to_owned(),
            AdapterConfigSourceDetails::new(source_level, path_origin, path, reason_code),
        )
    }

    pub fn cli_argv_ignored(tokens: Vec<String>, reason: impl Into<String>) -> Self {
        Self::from_typed_parts::<typed_codes::readable_warning::CliArgvIgnored>(
            reason.into(),
            CliArgvDetails::new(tokens),
        )
    }

    pub fn to_record_draft(&self, source: DiagnosticSource) -> DiagnosticRecordDraft {
        match self.code {
            ReadableWarningDiagnosticCode::CliArgvIgnored => {
                let DiagnosticDetails::CliArgv { tokens } = self.details.clone() else {
                    unreachable_warning_projection(self.code);
                };
                DiagnosticRecordDraft::new::<typed_codes::readable_warning::CliArgvIgnored>(
                    self.reason.clone(),
                    CliArgvDetails::new(tokens),
                    source,
                )
            }
            ReadableWarningDiagnosticCode::AdapterCandidateFailure => {
                let DiagnosticDetails::AdapterCandidate {
                    adapter_id,
                    stage,
                    code,
                    preselected,
                } = self.details.clone()
                else {
                    unreachable_warning_projection(self.code);
                };
                DiagnosticRecordDraft::new::<typed_codes::readable_warning::AdapterCandidateFailure>(
                    self.reason.clone(),
                    AdapterCandidateDetails::new(adapter_id, stage, code, preselected),
                    source,
                )
            }
            ReadableWarningDiagnosticCode::AdapterConfigSourceSkipped => {
                let DiagnosticDetails::AdapterConfigSource {
                    source_level,
                    path_origin,
                    path,
                    reason_code,
                } = self.details.clone()
                else {
                    unreachable_warning_projection(self.code);
                };
                DiagnosticRecordDraft::new::<
                    typed_codes::readable_warning::AdapterConfigSourceSkipped,
                >(
                    self.reason.clone(),
                    AdapterConfigSourceDetails::new(source_level, path_origin, path, reason_code),
                    source,
                )
            }
        }
    }

    pub fn from_record(record: &DiagnosticRecord) -> Option<Self> {
        let DiagnosticCode::ReadableWarning(code) = record.code() else {
            return None;
        };
        match code {
            ReadableWarningDiagnosticCode::CliArgvIgnored => {
                Self::from_record_typed::<typed_codes::readable_warning::CliArgvIgnored>(record)
            }
            ReadableWarningDiagnosticCode::AdapterCandidateFailure => Self::from_record_typed::<
                typed_codes::readable_warning::AdapterCandidateFailure,
            >(record),
            ReadableWarningDiagnosticCode::AdapterConfigSourceSkipped => Self::from_record_typed::<
                typed_codes::readable_warning::AdapterConfigSourceSkipped,
            >(record),
        }
    }

    fn from_record_typed<C>(record: &DiagnosticRecord) -> Option<Self>
    where
        C: ReadableWarningDiagnosticMarker,
    {
        let details = serde_json::from_value::<C::Details>(record.details().to_value()).ok()?;
        Self::new::<C>(record.summary(), details).ok()
    }
}

fn unreachable_warning_projection(code: ReadableWarningDiagnosticCode) -> ! {
    unreachable!("warning projection details are constructed by typed marker for {code:?}")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmptyWarningReason;

impl fmt::Display for EmptyWarningReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("warning reason must not be empty")
    }
}

impl std::error::Error for EmptyWarningReason {}

pub fn warning_text_line(warning: &WarningProjection) -> Result<String, serde_json::Error> {
    let details = serde_json::to_string(&warning.details)?;
    Ok(format!(
        "warning: id={}, effect={}, reason={}, details={}",
        warning.code.warning_id(),
        diagnostic_effect_as_str(warning.effect),
        warning.reason.replace(['\r', '\n'], " "),
        details
    ))
}

pub fn attach_warnings_to_value<T: Serialize>(mut value: Value, warnings: &[T]) -> Value {
    if warnings.is_empty() {
        return value;
    }

    let warnings = serde_json::to_value(warnings).unwrap_or_else(|_| Value::Array(Vec::new()));
    match &mut value {
        Value::Object(object) => {
            object.insert("warnings".to_owned(), warnings);
            value
        }
        _ => {
            let mut object = Map::new();
            object.insert("value".to_owned(), value);
            object.insert("warnings".to_owned(), warnings);
            Value::Object(object)
        }
    }
}

pub fn write_warning_text_lines<W: Write>(
    warnings: &[WarningProjection],
    writer: &mut W,
) -> io::Result<()> {
    for warning in warnings {
        writeln!(
            writer,
            "{}",
            warning_text_line(warning).map_err(io::Error::other)?
        )?;
    }
    Ok(())
}

fn diagnostic_effect_as_str(effect: DiagnosticEffect) -> &'static str {
    match effect {
        DiagnosticEffect::OperationContinued => "operation_continued",
        DiagnosticEffect::CandidateSkipped => "candidate_skipped",
        DiagnosticEffect::InputRejected => "input_rejected",
        DiagnosticEffect::DocumentFailed => "document_failed",
        DiagnosticEffect::AdapterBoundaryFailed => "adapter_boundary_failed",
        DiagnosticEffect::InternalFailure => "internal_failure",
    }
}
