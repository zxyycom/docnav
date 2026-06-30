use std::collections::BTreeMap;

use docnav_diagnostics::{
    typed_codes, AdapterConfigSourceDetails, DiagnosticRecordDraft, DiagnosticSource,
    FieldReasonDetails,
};
use docnav_typed_fields::{
    FieldIdentity, JsonValue, SchemaMetadataView, TypedValue, ValidationFailure,
};

use crate::{
    EntryPassthroughPolicy, OperationArgumentBinding, PassthroughValue,
    StandardParameterCatalogEntry, StandardParameterPath, StandardParameterSource,
    StandardParameterSourceInfo, StandardParameterSourceKind, StandardParameterSources,
};

#[derive(Clone, Debug, PartialEq)]
pub struct StandardParameterValidationIssue {
    pub identity: FieldIdentity,
    pub source: Option<StandardParameterSourceInfo>,
    pub failure: ValidationFailure,
}

impl StandardParameterValidationIssue {
    pub fn to_record_draft(&self, source: DiagnosticSource) -> DiagnosticRecordDraft {
        DiagnosticRecordDraft::new::<typed_codes::protocol::InvalidRequest>(
            format!(
                "standard parameter {} failed validation",
                self.identity.as_str()
            ),
            FieldReasonDetails::new(self.identity.as_str(), self.failure.to_string()),
            source,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StandardParameterConfigSourceIssue {
    pub source_level: String,
    pub path_origin: String,
    pub path: String,
    pub reason_code: String,
}

impl StandardParameterConfigSourceIssue {
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
            reason_code: reason_code.into(),
        }
    }

    pub fn message(&self) -> String {
        format!(
            "adapter config source failed: {} {} {} ({})",
            self.source_level, self.path_origin, self.path, self.reason_code
        )
    }

    pub fn to_record_draft(&self, source: DiagnosticSource) -> DiagnosticRecordDraft {
        let mut details = FieldReasonDetails::new("config", self.reason_code.clone());
        details.path = Some(self.path.clone());
        details.received = Some(self.path.clone());
        details.config_issues = Some(vec![AdapterConfigSourceDetails::new(
            &self.source_level,
            &self.path_origin,
            &self.path,
            &self.reason_code,
        )]);
        DiagnosticRecordDraft::new::<typed_codes::protocol::InvalidRequest>(
            self.message(),
            details,
            source,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum StandardParameterHandoff {
    Validation(StandardParameterValidationIssue),
    ConfigSource(StandardParameterConfigSourceIssue),
}

impl StandardParameterHandoff {
    pub fn validation(
        identity: FieldIdentity,
        source: Option<StandardParameterSourceInfo>,
        failure: ValidationFailure,
    ) -> Self {
        Self::Validation(StandardParameterValidationIssue {
            identity,
            source,
            failure,
        })
    }

    pub fn config_source(issue: StandardParameterConfigSourceIssue) -> Self {
        Self::ConfigSource(issue)
    }

    pub fn as_validation(&self) -> Option<&StandardParameterValidationIssue> {
        match self {
            Self::Validation(diagnostic) => Some(diagnostic),
            Self::ConfigSource(_) => None,
        }
    }

    pub fn as_config_source(&self) -> Option<&StandardParameterConfigSourceIssue> {
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
    pub arguments_path: StandardParameterPath,
    pub source: StandardParameterSourceInfo,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedStandardParameter {
    pub value: TypedValue,
    pub source: StandardParameterSourceInfo,
    pub operation_argument: Option<ResolvedOperationArgumentBinding>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StandardParameterResolution {
    values: BTreeMap<FieldIdentity, ResolvedStandardParameter>,
    diagnostics: Vec<StandardParameterHandoff>,
    passthrough: Vec<PassthroughValue>,
}

impl StandardParameterResolution {
    pub fn value(&self, identity: &FieldIdentity) -> Option<&ResolvedStandardParameter> {
        self.values.get(identity)
    }

    pub fn values(&self) -> &BTreeMap<FieldIdentity, ResolvedStandardParameter> {
        &self.values
    }

    pub fn diagnostics(&self) -> &[StandardParameterHandoff] {
        &self.diagnostics
    }

    pub fn passthrough(&self) -> &[PassthroughValue] {
        &self.passthrough
    }

    fn insert_value(
        &mut self,
        identity: FieldIdentity,
        value: TypedValue,
        source: StandardParameterSourceInfo,
        operation_argument: Option<OperationArgumentBinding>,
    ) {
        let operation_argument =
            operation_argument.map(|binding| ResolvedOperationArgumentBinding {
                arguments_path: binding.arguments_path,
                source: source.clone(),
            });
        self.values.insert(
            identity,
            ResolvedStandardParameter {
                value,
                source,
                operation_argument,
            },
        );
    }

    fn push_diagnostic(
        &mut self,
        identity: FieldIdentity,
        source: Option<StandardParameterSourceInfo>,
        failure: ValidationFailure,
    ) {
        self.diagnostics.push(StandardParameterHandoff::validation(
            identity, source, failure,
        ));
    }

    pub(crate) fn extend_diagnostics(
        &mut self,
        diagnostics: impl IntoIterator<Item = StandardParameterHandoff>,
    ) {
        self.diagnostics.extend(diagnostics);
    }
}

pub(crate) fn resolve_standard_parameters(
    entries: &[StandardParameterCatalogEntry],
    sources: StandardParameterSources,
    passthrough_policy: EntryPassthroughPolicy,
) -> StandardParameterResolution {
    let mut resolution = StandardParameterResolution::default();

    for entry in entries {
        resolve_entry(entry, &sources, &mut resolution);
    }

    resolution.passthrough = resolve_passthrough(&sources, passthrough_policy);
    resolution
}

fn resolve_entry(
    entry: &StandardParameterCatalogEntry,
    sources: &StandardParameterSources,
    resolution: &mut StandardParameterResolution,
) {
    if let Some((value, source)) = first_source_value(entry.identity(), sources) {
        resolve_raw_value(entry, value, source, resolution);
        return;
    }

    match entry.metadata.static_default_value() {
        Ok(Some(value)) => resolution.insert_value(
            entry.identity().clone(),
            value,
            StandardParameterSourceInfo::new(StandardParameterSourceKind::Default),
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
            Some(StandardParameterSourceInfo::new(
                StandardParameterSourceKind::Default,
            )),
            failure,
        ),
    }
}

fn resolve_raw_value(
    entry: &StandardParameterCatalogEntry,
    value: &JsonValue,
    source_kind: StandardParameterSourceKind,
    resolution: &mut StandardParameterResolution,
) {
    let source = StandardParameterSourceInfo::new(source_kind);
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
    sources: &'a StandardParameterSources,
) -> Option<(&'a JsonValue, StandardParameterSourceKind)> {
    [
        (
            sources.direct_input.value(identity),
            StandardParameterSourceKind::DirectInput,
        ),
        (
            sources.project_config.value(identity),
            StandardParameterSourceKind::ProjectConfig,
        ),
        (
            sources.user_config.value(identity),
            StandardParameterSourceKind::UserConfig,
        ),
        (
            sources.default.value(identity),
            StandardParameterSourceKind::Default,
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
    sources: &StandardParameterSources,
    policy: EntryPassthroughPolicy,
) -> Vec<PassthroughValue> {
    let mut passthrough = Vec::new();
    collect_passthrough(
        &mut passthrough,
        StandardParameterSourceKind::DirectInput,
        &sources.direct_input,
        policy,
    );
    collect_passthrough(
        &mut passthrough,
        StandardParameterSourceKind::ProjectConfig,
        &sources.project_config,
        policy,
    );
    collect_passthrough(
        &mut passthrough,
        StandardParameterSourceKind::UserConfig,
        &sources.user_config,
        policy,
    );
    passthrough
}

fn collect_passthrough(
    passthrough: &mut Vec<PassthroughValue>,
    source_kind: StandardParameterSourceKind,
    source: &StandardParameterSource,
    policy: EntryPassthroughPolicy,
) {
    if let Some(value) = source.processing_result() {
        passthrough.push(PassthroughValue {
            source: StandardParameterSourceInfo::new(source_kind),
            value: value.clone(),
            disposition: policy.disposition(),
        });
    }
}
