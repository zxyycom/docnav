use cli_config_resolution::{
    CandidateInvalidReason, DiagnosticReason, FieldDefSet, FieldPath, ProcessingId,
    ProcessingLocator, ResolutionDiagnostic, ResolutionResult, Resolver, Source, SourceCandidate,
    SourceId, SourceKind, SourceLocator,
};
use cli_config_resolution_serde::extract_config;
use docnav_diagnostics::{
    typed_codes, DiagnosticRecordDraft, DiagnosticSource, FieldReasonDetails,
};
use serde_json::Value;

use crate::{NavigationConfigSources, NavigationError};

use super::{input::DirectInput, CONFIG_PROCESSING, DIRECT_PROCESSING};

const CLI_PROCESSING: &str = "cli";
const EXPLICIT_SOURCE_ID: &str = "explicit";
const EXPLICIT_NATIVE_SOURCE_ID: &str = "explicit-native";
const PROJECT_SOURCE_ID: &str = "project";
const USER_SOURCE_ID: &str = "user";

const EXPLICIT_PRIORITY: i32 = 400;
const PROJECT_PRIORITY: i32 = 300;
const USER_PRIORITY: i32 = 200;

pub(super) fn resolve(
    fields: &FieldDefSet,
    direct_input: Option<&DirectInput>,
    config_sources: &NavigationConfigSources,
) -> Result<ResolutionResult, NavigationError> {
    let mut sources = Vec::new();
    if let Some(input) = direct_input {
        sources.extend(extract_direct_sources(fields, input)?);
    }
    if let Some(project) = config_sources.project.loaded.value() {
        sources.push(config_source(
            project,
            fields,
            PROJECT_SOURCE_ID,
            PROJECT_PRIORITY,
        )?);
    }
    if let Some(user) = config_sources.user.loaded.value() {
        sources.push(config_source(user, fields, USER_SOURCE_ID, USER_PRIORITY)?);
    }

    Resolver::resolve(fields, &sources)
        .map_err(|_| NavigationError::internal("cli-config-resolution-input-invalid"))
}

pub(super) fn source_label(source_id: &SourceId, source_kind: &SourceKind) -> &'static str {
    match source_id.as_str() {
        EXPLICIT_SOURCE_ID | EXPLICIT_NATIVE_SOURCE_ID => "explicit",
        PROJECT_SOURCE_ID => "project",
        USER_SOURCE_ID => "user",
        _ => match source_kind {
            SourceKind::Default => "built_in",
            SourceKind::Cli | SourceKind::Custom(_) => "explicit",
            SourceKind::Config => "config",
            SourceKind::Env => "env",
        },
    }
}

pub(super) fn diagnostic_record(
    diagnostic: &ResolutionDiagnostic,
    source: DiagnosticSource,
) -> DiagnosticRecordDraft {
    DiagnosticRecordDraft::new::<typed_codes::protocol::InvalidRequest>(
        format!(
            "parameter resolution {} failed validation",
            diagnostic.field.as_str()
        ),
        FieldReasonDetails::new(diagnostic.field.as_str(), diagnostic_reason(diagnostic)),
        source,
    )
}

fn extract_direct_sources(
    fields: &FieldDefSet,
    input: &DirectInput,
) -> Result<Vec<Source>, NavigationError> {
    let direct_candidates = extract_direct_candidates(fields, input.common())?;
    let native_candidates = extract_native_cli_candidates(fields, input)?;
    let mut sources = Vec::new();
    if !direct_candidates.is_empty() {
        sources.push(source(
            EXPLICIT_SOURCE_ID,
            SourceKind::Custom("docnav-direct".to_owned()),
            EXPLICIT_PRIORITY,
            direct_candidates,
        )?);
    }
    if !native_candidates.is_empty() {
        sources.push(source(
            EXPLICIT_NATIVE_SOURCE_ID,
            SourceKind::Cli,
            EXPLICIT_PRIORITY,
            native_candidates,
        )?);
    }
    Ok(sources)
}

fn extract_direct_candidates(
    fields: &FieldDefSet,
    root: &Value,
) -> Result<Vec<SourceCandidate>, NavigationError> {
    let mut candidates = Vec::new();
    for metadata in fields.processing_metadata(&ProcessingId::from(DIRECT_PROCESSING)) {
        let ProcessingLocator::JsonPath(path) = metadata.locator else {
            return Err(NavigationError::internal(
                "navigation-direct-processing-locator-invalid",
            ));
        };
        let Some(value) = value_at_path(root, &path) else {
            continue;
        };
        candidates.push(SourceCandidate::value(
            metadata.identity,
            SourceLocator::Custom(path.segments().join(".")),
            value.clone(),
        ));
    }
    Ok(candidates)
}

fn extract_native_cli_candidates(
    fields: &FieldDefSet,
    input: &DirectInput,
) -> Result<Vec<SourceCandidate>, NavigationError> {
    let mut candidates = Vec::new();
    for metadata in fields.processing_metadata(&ProcessingId::from(CLI_PROCESSING)) {
        let ProcessingLocator::CliFlag(flag) = metadata.locator else {
            return Err(NavigationError::internal(
                "navigation-cli-processing-locator-invalid",
            ));
        };
        let Some(value) = input.native_value(metadata.identity.as_str()) else {
            continue;
        };
        candidates.push(SourceCandidate::value(
            metadata.identity,
            SourceLocator::CliFlag(flag),
            value.clone(),
        ));
    }
    Ok(candidates)
}

fn config_source(
    root: &Value,
    fields: &FieldDefSet,
    id: &str,
    priority: i32,
) -> Result<Source, NavigationError> {
    extract_config(
        root,
        fields,
        &ProcessingId::from(CONFIG_PROCESSING),
        source_id(id)?,
        priority,
    )
    .map_err(|_| NavigationError::internal("navigation-config-source-extraction-failed"))
}

fn source(
    id: &str,
    kind: SourceKind,
    priority: i32,
    candidates: Vec<SourceCandidate>,
) -> Result<Source, NavigationError> {
    Source::new(source_id(id)?, kind, priority, candidates)
        .map_err(|_| NavigationError::internal("navigation-direct-source-build-failed"))
}

fn source_id(id: &str) -> Result<SourceId, NavigationError> {
    SourceId::new(id).map_err(|_| NavigationError::internal("cli-config-source-id-invalid"))
}

fn value_at_path<'a>(root: &'a Value, path: &FieldPath) -> Option<&'a Value> {
    let mut current = root;
    for segment in path.segments() {
        current = current.as_object()?.get(segment)?;
    }
    Some(current)
}

fn diagnostic_reason(diagnostic: &ResolutionDiagnostic) -> String {
    match &diagnostic.reason {
        DiagnosticReason::InvalidCandidate(reason) => match reason {
            CandidateInvalidReason::Decode(reason) => reason.clone(),
            CandidateInvalidReason::Shape { expected } => {
                format!("expected {expected:?} value")
            }
            CandidateInvalidReason::Validation(failure) => failure.to_string(),
        },
        DiagnosticReason::FinalValidation(failure) | DiagnosticReason::MissingRequired(failure) => {
            failure.to_string()
        }
        DiagnosticReason::MergeConflict(locators) => format!(
            "conflicting values from {}",
            locators
                .iter()
                .map(SourceLocator::as_key)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}
