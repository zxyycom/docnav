use std::collections::BTreeMap;

use docnav_diagnostics::{
    DiagnosticDetails, DiagnosticRecordDraft, DiagnosticSource, ProtocolDiagnosticCode, Warning,
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
pub struct StandardParameterValidationDiagnostic {
    pub identity: FieldIdentity,
    pub source: Option<StandardParameterSourceInfo>,
    pub failure: ValidationFailure,
}

impl StandardParameterValidationDiagnostic {
    pub fn to_record_draft(&self, source: DiagnosticSource) -> DiagnosticRecordDraft {
        DiagnosticRecordDraft::new(
            ProtocolDiagnosticCode::InvalidRequest,
            format!(
                "standard parameter {} failed validation",
                self.identity.as_str()
            ),
            DiagnosticDetails::FieldReason {
                field: self.identity.as_str().to_owned(),
                reason: self.failure.to_string(),
            },
            source,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum StandardParameterDiagnostic {
    Validation(StandardParameterValidationDiagnostic),
    Warning(Warning),
}

impl StandardParameterDiagnostic {
    pub fn validation(
        identity: FieldIdentity,
        source: Option<StandardParameterSourceInfo>,
        failure: ValidationFailure,
    ) -> Self {
        Self::Validation(StandardParameterValidationDiagnostic {
            identity,
            source,
            failure,
        })
    }

    pub fn warning(warning: Warning) -> Self {
        Self::Warning(warning)
    }

    pub fn as_validation(&self) -> Option<&StandardParameterValidationDiagnostic> {
        match self {
            Self::Validation(diagnostic) => Some(diagnostic),
            Self::Warning(_) => None,
        }
    }

    pub fn as_warning(&self) -> Option<&Warning> {
        match self {
            Self::Validation(_) => None,
            Self::Warning(warning) => Some(warning),
        }
    }

    pub fn to_record_draft(&self, source: DiagnosticSource) -> Option<DiagnosticRecordDraft> {
        match self {
            Self::Validation(diagnostic) => Some(diagnostic.to_record_draft(source)),
            Self::Warning(warning) => warning.to_record_draft(source),
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
    diagnostics: Vec<StandardParameterDiagnostic>,
    passthrough: Vec<PassthroughValue>,
}

impl StandardParameterResolution {
    pub fn value(&self, identity: &FieldIdentity) -> Option<&ResolvedStandardParameter> {
        self.values.get(identity)
    }

    pub fn values(&self) -> &BTreeMap<FieldIdentity, ResolvedStandardParameter> {
        &self.values
    }

    pub fn diagnostics(&self) -> &[StandardParameterDiagnostic] {
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
        self.diagnostics
            .push(StandardParameterDiagnostic::validation(
                identity, source, failure,
            ));
    }

    pub(crate) fn extend_diagnostics(
        &mut self,
        diagnostics: impl IntoIterator<Item = StandardParameterDiagnostic>,
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
