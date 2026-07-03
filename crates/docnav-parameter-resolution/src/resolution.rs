use std::collections::BTreeMap;

use docnav_diagnostics::{
    typed_codes, AdapterConfigSourceDetails, DiagnosticRecordDraft, DiagnosticSource,
    FieldReasonDetails,
};
use docnav_typed_fields::{
    FieldIdentity, JsonValue, SchemaMetadataView, TypedValue, ValidationFailure,
};

use crate::{
    EntryPassthroughPolicy, OperationArgumentBinding, ParameterCatalogEntry, ParameterPath,
    ParameterSource, ParameterSourceInfo, ParameterSourceKind, ParameterSources, PassthroughValue,
};

#[derive(Clone, Debug, PartialEq)]
pub struct ParameterValidationIssue {
    pub identity: FieldIdentity,
    pub source: Option<ParameterSourceInfo>,
    pub failure: ValidationFailure,
}

impl ParameterValidationIssue {
    pub fn to_record_draft(&self, source: DiagnosticSource) -> DiagnosticRecordDraft {
        DiagnosticRecordDraft::new::<typed_codes::protocol::InvalidRequest>(
            format!(
                "parameter resolution {} failed validation",
                self.identity.as_str()
            ),
            FieldReasonDetails::new(self.identity.as_str(), self.failure.to_string()),
            source,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParameterConfigSourceIssue {
    pub source_level: String,
    pub path_origin: String,
    pub path: String,
    pub field: Option<String>,
    pub reason_code: String,
}

impl ParameterConfigSourceIssue {
    pub fn new(
        source_level: impl Into<String>,
        path_origin: impl Into<String>,
        path: impl Into<String>,
        reason_code: impl Into<String>,
    ) -> Self {
        Self {
            source_level: source_level.into(),
            path_origin: path_origin.into(),
            path: path.into(),
            field: None,
            reason_code: reason_code.into(),
        }
    }

    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }

    pub fn message(&self) -> String {
        let field = self
            .field
            .as_ref()
            .map_or(String::new(), |field| format!(" field {field}"));
        format!(
            "adapter config source failed: {} {} {}{} ({})",
            self.source_level, self.path_origin, self.path, field, self.reason_code
        )
    }

    pub fn to_record_draft(&self, source: DiagnosticSource) -> DiagnosticRecordDraft {
        let field = self.field.as_deref().unwrap_or("config");
        let mut details = FieldReasonDetails::new(field, self.reason_code.clone());
        details.path = Some(self.path.clone());
        details.received = Some(self.field.clone().unwrap_or_else(|| self.path.clone()));
        let mut issue = AdapterConfigSourceDetails::new(
            &self.source_level,
            &self.path_origin,
            &self.path,
            &self.reason_code,
        );
        if let Some(field) = &self.field {
            issue = issue.with_field(field);
        }
        details.config_issues = Some(vec![issue]);
        DiagnosticRecordDraft::new::<typed_codes::protocol::InvalidRequest>(
            self.message(),
            details,
            source,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParameterResolutionHandoff {
    Validation(ParameterValidationIssue),
    ConfigSource(ParameterConfigSourceIssue),
}

impl ParameterResolutionHandoff {
    pub fn validation(
        identity: FieldIdentity,
        source: Option<ParameterSourceInfo>,
        failure: ValidationFailure,
    ) -> Self {
        Self::Validation(ParameterValidationIssue {
            identity,
            source,
            failure,
        })
    }

    pub fn config_source(issue: ParameterConfigSourceIssue) -> Self {
        Self::ConfigSource(issue)
    }

    pub fn as_validation(&self) -> Option<&ParameterValidationIssue> {
        match self {
            Self::Validation(diagnostic) => Some(diagnostic),
            Self::ConfigSource(_) => None,
        }
    }

    pub fn as_config_source(&self) -> Option<&ParameterConfigSourceIssue> {
        match self {
            Self::Validation(_) => None,
            Self::ConfigSource(issue) => Some(issue),
        }
    }

    pub fn to_record_draft(&self, source: DiagnosticSource) -> DiagnosticRecordDraft {
        match self {
            Self::Validation(diagnostic) => diagnostic.to_record_draft(source),
            Self::ConfigSource(issue) => issue.to_record_draft(source),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedOperationArgumentBinding {
    pub arguments_path: ParameterPath,
    pub source: ParameterSourceInfo,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedParameter {
    pub value: TypedValue,
    pub source: ParameterSourceInfo,
    pub operation_argument: Option<ResolvedOperationArgumentBinding>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParameterResolution {
    values: BTreeMap<FieldIdentity, ResolvedParameter>,
    diagnostics: Vec<ParameterResolutionHandoff>,
    passthrough: Vec<PassthroughValue>,
}

impl ParameterResolution {
    pub fn value(&self, identity: &FieldIdentity) -> Option<&ResolvedParameter> {
        self.values.get(identity)
    }

    pub fn values(&self) -> &BTreeMap<FieldIdentity, ResolvedParameter> {
        &self.values
    }

    pub fn diagnostics(&self) -> &[ParameterResolutionHandoff] {
        &self.diagnostics
    }

    pub fn passthrough(&self) -> &[PassthroughValue] {
        &self.passthrough
    }

    fn insert_value(
        &mut self,
        identity: FieldIdentity,
        value: TypedValue,
        source: ParameterSourceInfo,
        operation_argument: Option<OperationArgumentBinding>,
    ) {
        let operation_argument =
            operation_argument.map(|binding| ResolvedOperationArgumentBinding {
                arguments_path: binding.arguments_path,
                source: source.clone(),
            });
        self.values.insert(
            identity,
            ResolvedParameter {
                value,
                source,
                operation_argument,
            },
        );
    }

    fn push_diagnostic(
        &mut self,
        identity: FieldIdentity,
        source: Option<ParameterSourceInfo>,
        failure: ValidationFailure,
    ) {
        self.diagnostics
            .push(ParameterResolutionHandoff::validation(
                identity, source, failure,
            ));
    }

    pub(crate) fn extend_diagnostics(
        &mut self,
        diagnostics: impl IntoIterator<Item = ParameterResolutionHandoff>,
    ) {
        self.diagnostics.extend(diagnostics);
    }
}

pub(crate) fn resolve_parameters(
    entries: &[ParameterCatalogEntry],
    sources: ParameterSources,
    passthrough_policy: EntryPassthroughPolicy,
) -> ParameterResolution {
    let mut resolution = ParameterResolution::default();

    for entry in entries {
        resolve_entry(entry, &sources, &mut resolution);
    }

    resolution.passthrough = resolve_passthrough(&sources, passthrough_policy);
    resolution
}

fn resolve_entry(
    entry: &ParameterCatalogEntry,
    sources: &ParameterSources,
    resolution: &mut ParameterResolution,
) {
    if let Some((value, source)) = first_source_value(entry.identity(), sources) {
        resolve_raw_value(entry, value, source, resolution);
        return;
    }

    match entry.metadata.static_default_value() {
        Ok(Some(value)) => resolution.insert_value(
            entry.identity().clone(),
            value,
            ParameterSourceInfo::new(ParameterSourceKind::Default),
            entry.operation_argument.clone(),
        ),
        Ok(None) if entry.metadata.is_required() => resolution.push_diagnostic(
            entry.identity().clone(),
            None,
            missing_required_failure(&entry.metadata),
        ),
        Ok(None) => {}
        Err(failure) => resolution.push_diagnostic(
            entry.identity().clone(),
            Some(ParameterSourceInfo::new(ParameterSourceKind::Default)),
            failure,
        ),
    }
}

fn resolve_raw_value(
    entry: &ParameterCatalogEntry,
    value: &JsonValue,
    source_kind: ParameterSourceKind,
    resolution: &mut ParameterResolution,
) {
    let source = ParameterSourceInfo::new(source_kind);
    match entry.metadata.validate_optional_value(Some(value)) {
        Ok(Some(value)) => resolution.insert_value(
            entry.identity().clone(),
            value,
            source,
            entry.operation_argument.clone(),
        ),
        Ok(None) => {}
        Err(failure) => resolution.push_diagnostic(entry.identity().clone(), Some(source), failure),
    }
}

fn first_source_value<'a>(
    identity: &FieldIdentity,
    sources: &'a ParameterSources,
) -> Option<(&'a JsonValue, ParameterSourceKind)> {
    [
        (
            sources.direct_input.value(identity),
            ParameterSourceKind::DirectInput,
        ),
        (
            sources.project_config.value(identity),
            ParameterSourceKind::ProjectConfig,
        ),
        (
            sources.user_config.value(identity),
            ParameterSourceKind::UserConfig,
        ),
        (
            sources.default.value(identity),
            ParameterSourceKind::Default,
        ),
    ]
    .into_iter()
    .find_map(|(value, source)| value.map(|value| (value, source)))
}

fn missing_required_failure(metadata: &SchemaMetadataView) -> ValidationFailure {
    metadata
        .validate_optional_value(None)
        .expect_err("required metadata without a value must fail validation")
}

fn resolve_passthrough(
    sources: &ParameterSources,
    policy: EntryPassthroughPolicy,
) -> Vec<PassthroughValue> {
    let mut passthrough = Vec::new();
    collect_passthrough(
        &mut passthrough,
        ParameterSourceKind::DirectInput,
        &sources.direct_input,
        policy,
    );
    collect_passthrough(
        &mut passthrough,
        ParameterSourceKind::ProjectConfig,
        &sources.project_config,
        policy,
    );
    collect_passthrough(
        &mut passthrough,
        ParameterSourceKind::UserConfig,
        &sources.user_config,
        policy,
    );
    passthrough
}

fn collect_passthrough(
    passthrough: &mut Vec<PassthroughValue>,
    source_kind: ParameterSourceKind,
    source: &ParameterSource,
    policy: EntryPassthroughPolicy,
) {
    if let Some(value) = source.processing_result() {
        passthrough.push(PassthroughValue {
            source: ParameterSourceInfo::new(source_kind),
            value: value.clone(),
            disposition: policy.disposition(),
        });
    }
}
