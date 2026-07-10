use std::collections::{BTreeMap, BTreeSet};

use cli_config_resolution::{
    CandidateState, DefaultMetadata as GenericDefaultMetadata, DiagnosticReason, FieldConstraints,
    FieldContract, FieldIdentity as GenericFieldIdentity, FieldSet, ReceivedValueKind,
    ResolutionDiagnostic, Resolver, SourceCandidate, SourceCollection, SourceId, SourceKind,
    SourceLocator, SourceSpec, Value as GenericValue, ValueKind as GenericValueKind,
};
use docnav_diagnostics::{
    typed_codes, DiagnosticRecordDraft, DiagnosticSource, FieldReasonDetails,
};
use docnav_typed_fields::{
    ActualValueKind, DefaultMetadata, FieldBound, FieldBoundKind, FieldIdentity, FieldNumericBound,
    FieldNumericRange, FieldPath, ProcessingId, ProcessingMetadataView, SchemaMetadataView,
    TypedValue, ValidationFailure, ValidationReason, ValueKind,
};
use serde_json::Value;

use crate::{config_source::NavigationConfigSourceIssue, NavigationConfigSources, NavigationError};

use super::{CONFIG_PROCESSING, DIRECT_PROCESSING};

const EXPLICIT_SOURCE_ID: &str = "explicit";
const PROJECT_SOURCE_ID: &str = "project";
const USER_SOURCE_ID: &str = "user";
const BUILT_IN_SOURCE_ID: &str = "built_in";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ParameterSourceKind {
    DirectInput,
    ProjectConfig,
    UserConfig,
    Default,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ParameterSourceInfo {
    pub(super) kind: ParameterSourceKind,
}

impl ParameterSourceInfo {
    const fn new(kind: ParameterSourceKind) -> Self {
        Self { kind }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ParameterValidationIssue {
    pub(super) identity: FieldIdentity,
    pub(super) source: Option<ParameterSourceInfo>,
    pub(super) failure: ValidationFailure,
}

impl ParameterValidationIssue {
    pub(super) fn to_record_draft(&self, source: DiagnosticSource) -> DiagnosticRecordDraft {
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
pub(super) enum ParameterResolutionHandoff {
    Validation(ParameterValidationIssue),
    ConfigSource(NavigationConfigSourceIssue),
}

impl ParameterResolutionHandoff {
    pub(super) fn to_record_draft(&self, source: DiagnosticSource) -> DiagnosticRecordDraft {
        match self {
            Self::Validation(diagnostic) => diagnostic.to_record_draft(source),
            Self::ConfigSource(issue) => issue.to_record_draft(source),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ResolvedParameter {
    pub(super) value: TypedValue,
    pub(super) source: ParameterSourceInfo,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct ParameterResolution {
    values: BTreeMap<FieldIdentity, ResolvedParameter>,
    diagnostics: Vec<ParameterResolutionHandoff>,
}

impl ParameterResolution {
    pub(super) fn value(&self, identity: &FieldIdentity) -> Option<&ResolvedParameter> {
        self.values.get(identity)
    }

    pub(super) fn values(&self) -> &BTreeMap<FieldIdentity, ResolvedParameter> {
        &self.values
    }

    pub(super) fn diagnostics(&self) -> &[ParameterResolutionHandoff] {
        &self.diagnostics
    }
}

pub(super) fn resolve(
    fields: &docnav_typed_fields::FieldDefSet,
    direct_input: Option<&Value>,
    config_sources: &NavigationConfigSources,
) -> Result<ParameterResolution, NavigationError> {
    let metadata = FieldMetadata::from_fields(fields)?;
    let field_set = generic_field_set(&metadata)?;
    let sources = generic_sources()?;
    let mut candidates = Vec::new();

    if let Some(input) = direct_input {
        collect_processing_candidates(
            &mut candidates,
            &metadata.direct,
            input,
            source_spec(&sources, EXPLICIT_SOURCE_ID)?,
            ParameterSourceKind::DirectInput,
        )?;
    }
    if let Some(project) = config_sources.project.loaded.value() {
        collect_processing_candidates(
            &mut candidates,
            &metadata.config,
            project,
            source_spec(&sources, PROJECT_SOURCE_ID)?,
            ParameterSourceKind::ProjectConfig,
        )?;
    }
    if let Some(user) = config_sources.user.loaded.value() {
        collect_processing_candidates(
            &mut candidates,
            &metadata.config,
            user,
            source_spec(&sources, USER_SOURCE_ID)?,
            ParameterSourceKind::UserConfig,
        )?;
    }
    collect_default_candidates(
        &mut candidates,
        &metadata.schema,
        source_spec(&sources, BUILT_IN_SOURCE_ID)?,
    )?;

    let generic = Resolver::resolve(&field_set, &sources, candidates);
    let diagnostics = filtered_diagnostics(&generic, &sources);
    let diagnostic_fields = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.field.as_str().to_owned())
        .collect::<BTreeSet<_>>();

    let mut resolution = ParameterResolution::default();
    for (identity, field_resolution) in generic.fields() {
        if diagnostic_fields.contains(identity.as_str()) {
            continue;
        }
        let Some(value) = field_resolution.value() else {
            continue;
        };
        let Some(schema) = metadata.schema(identity.as_str()) else {
            continue;
        };
        let json_value = generic_value_to_json(value);
        match schema.validate_value(&json_value) {
            Ok(typed) => {
                let source = field_resolution
                    .trace()
                    .selected
                    .as_ref()
                    .and_then(|candidate| source_info_from_id(candidate.source_id.as_str()))
                    .unwrap_or_else(|| ParameterSourceInfo::new(ParameterSourceKind::Default));
                resolution.values.insert(
                    typed_identity(identity)?,
                    ResolvedParameter {
                        value: typed,
                        source,
                    },
                );
            }
            Err(failure) => {
                resolution
                    .diagnostics
                    .push(ParameterResolutionHandoff::Validation(
                        ParameterValidationIssue {
                            identity: typed_identity(identity)?,
                            source: field_resolution.trace().selected.as_ref().and_then(
                                |candidate| source_info_from_id(candidate.source_id.as_str()),
                            ),
                            failure,
                        },
                    ));
            }
        }
    }

    let converted_diagnostics = diagnostics
        .iter()
        .map(|diagnostic| {
            parameter_diagnostic(diagnostic, metadata.schema(diagnostic.field.as_str()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    resolution
        .diagnostics
        .extend(converted_diagnostics.into_iter().flatten());
    order_validation_diagnostics(&mut resolution.diagnostics, &metadata.schema);
    resolution.diagnostics.extend(
        config_sources
            .project
            .loaded
            .diagnostics()
            .iter()
            .cloned()
            .map(ParameterResolutionHandoff::ConfigSource),
    );
    resolution.diagnostics.extend(
        config_sources
            .user
            .loaded
            .diagnostics()
            .iter()
            .cloned()
            .map(ParameterResolutionHandoff::ConfigSource),
    );
    Ok(resolution)
}

fn order_validation_diagnostics(
    diagnostics: &mut [ParameterResolutionHandoff],
    schema: &[SchemaMetadataView],
) {
    diagnostics.sort_by_key(|diagnostic| match diagnostic {
        ParameterResolutionHandoff::Validation(issue) => schema
            .iter()
            .position(|metadata| metadata.identity == issue.identity)
            .unwrap_or(usize::MAX),
        ParameterResolutionHandoff::ConfigSource(_) => usize::MAX,
    });
}

fn parameter_diagnostic(
    diagnostic: &ResolutionDiagnostic,
    schema: Option<&SchemaMetadataView>,
) -> Result<Option<ParameterResolutionHandoff>, NavigationError> {
    let Some(schema) = schema else {
        return Ok(None);
    };
    Ok(Some(ParameterResolutionHandoff::Validation(
        ParameterValidationIssue {
            identity: typed_identity(&diagnostic.field)?,
            source: diagnostic
                .source_id
                .as_ref()
                .and_then(|source| source_info_from_id(source.as_str())),
            failure: ValidationFailure {
                field: typed_identity(&diagnostic.field)?,
                path: schema.path.clone(),
                reason: validation_reason(diagnostic, schema),
            },
        },
    )))
}

fn validation_reason(
    diagnostic: &ResolutionDiagnostic,
    schema: &SchemaMetadataView,
) -> ValidationReason {
    match &diagnostic.reason {
        DiagnosticReason::ValidationFailed(reason) => generic_validation_reason(reason, schema),
        DiagnosticReason::MissingRequired => ValidationReason::MissingRequired,
        DiagnosticReason::SourceInvalid { .. }
        | DiagnosticReason::MergeConflict(_)
        | DiagnosticReason::AmbiguousPriority { .. } => ValidationReason::WrongType {
            expected: schema.value_kind,
            actual: actual_kind(diagnostic.received_kind),
        },
    }
}

fn generic_validation_reason(
    reason: &cli_config_resolution::ValidationReason,
    schema: &SchemaMetadataView,
) -> ValidationReason {
    match reason {
        cli_config_resolution::ValidationReason::MissingRequired => {
            ValidationReason::MissingRequired
        }
        cli_config_resolution::ValidationReason::NullNotAllowed => ValidationReason::WrongType {
            expected: schema.value_kind,
            actual: ActualValueKind::Null,
        },
        cli_config_resolution::ValidationReason::WrongType { actual, .. } => {
            ValidationReason::WrongType {
                expected: schema.value_kind,
                actual: actual_kind(Some(*actual)),
            }
        }
        cli_config_resolution::ValidationReason::DisallowedValue { allowed } => {
            ValidationReason::DisallowedEnumValue {
                allowed: allowed.iter().map(generic_value_to_json).collect(),
            }
        }
        cli_config_resolution::ValidationReason::BelowMinimum { minimum } => {
            ValidationReason::BelowMinimum {
                minimum: numeric_bound(schema.value_kind, *minimum),
            }
        }
        cli_config_resolution::ValidationReason::AboveMaximum { maximum } => {
            ValidationReason::AboveMaximum {
                maximum: numeric_bound(schema.value_kind, *maximum),
            }
        }
        cli_config_resolution::ValidationReason::BelowMinimumLength { minimum } => {
            ValidationReason::BelowMinimumLength {
                minimum: FieldBound::closed(u64::try_from(*minimum).unwrap_or(u64::MAX)),
            }
        }
        cli_config_resolution::ValidationReason::AboveMaximumLength { maximum } => {
            ValidationReason::AboveMaximumLength {
                maximum: FieldBound::closed(u64::try_from(*maximum).unwrap_or(u64::MAX)),
            }
        }
    }
}

fn numeric_bound(value_kind: ValueKind, value: f64) -> FieldNumericBound {
    if value_kind == ValueKind::Integer {
        FieldNumericBound::Integer(FieldBound::closed(value as i64))
    } else {
        FieldNumericBound::Number(FieldBound::closed(value))
    }
}

fn actual_kind(kind: Option<ReceivedValueKind>) -> ActualValueKind {
    match kind {
        Some(ReceivedValueKind::String) => ActualValueKind::String,
        Some(ReceivedValueKind::Integer) => ActualValueKind::Integer,
        Some(ReceivedValueKind::Number) => ActualValueKind::Number,
        Some(ReceivedValueKind::Boolean) => ActualValueKind::Boolean,
        Some(ReceivedValueKind::List) => ActualValueKind::Array,
        Some(ReceivedValueKind::Map) => ActualValueKind::Object,
        Some(ReceivedValueKind::Null) | None => ActualValueKind::Null,
    }
}

fn filtered_diagnostics(
    generic: &cli_config_resolution::ResolutionResult,
    sources: &SourceCollection,
) -> Vec<ResolutionDiagnostic> {
    generic
        .diagnostics()
        .iter()
        .filter(|diagnostic| {
            let Some(diagnostic_source) = diagnostic.source_id.as_ref() else {
                return true;
            };
            let Some(field_resolution) = generic.fields().get(&diagnostic.field) else {
                return true;
            };
            let Some(selected) = field_resolution.trace().selected.as_ref() else {
                return true;
            };
            source_priority(sources, &selected.source_id)
                <= source_priority(sources, diagnostic_source)
        })
        .cloned()
        .collect()
}

fn source_priority(sources: &SourceCollection, source_id: &SourceId) -> i32 {
    sources
        .get(source_id)
        .map_or(i32::MIN, SourceSpec::priority)
}

fn source_info_from_id(source_id: &str) -> Option<ParameterSourceInfo> {
    let kind = match source_id {
        EXPLICIT_SOURCE_ID => ParameterSourceKind::DirectInput,
        PROJECT_SOURCE_ID => ParameterSourceKind::ProjectConfig,
        USER_SOURCE_ID => ParameterSourceKind::UserConfig,
        BUILT_IN_SOURCE_ID => ParameterSourceKind::Default,
        _ => return None,
    };
    Some(ParameterSourceInfo::new(kind))
}

struct FieldMetadata {
    schema: Vec<SchemaMetadataView>,
    direct: Vec<ProcessingMetadataView>,
    config: Vec<ProcessingMetadataView>,
}

impl FieldMetadata {
    fn from_fields(fields: &docnav_typed_fields::FieldDefSet) -> Result<Self, NavigationError> {
        Ok(Self {
            schema: fields.schema_metadata(),
            direct: fields.processing_metadata(&ProcessingId::from(DIRECT_PROCESSING)),
            config: fields.processing_metadata(&ProcessingId::from(CONFIG_PROCESSING)),
        })
    }

    fn schema(&self, identity: &str) -> Option<&SchemaMetadataView> {
        self.schema
            .iter()
            .find(|metadata| metadata.identity.as_str() == identity)
    }
}

fn generic_field_set(metadata: &FieldMetadata) -> Result<FieldSet, NavigationError> {
    let mut builder = FieldSet::builder();
    for schema in &metadata.schema {
        let mut contract = FieldContract::builder(
            schema.identity.as_str(),
            generic_value_kind(schema.value_kind),
        )
        .constraints(generic_constraints(schema))
        .default(generic_default(schema)?);
        contract = contract.projection(cli_config_resolution::FieldProjectionDeclaration::default(
            schema.identity.as_str(),
        ));
        builder = builder.add_declaration(contract);
    }
    builder
        .build()
        .map_err(|_| NavigationError::internal("cli-config-field-set-build-failed"))
}

fn generic_constraints(schema: &SchemaMetadataView) -> FieldConstraints {
    let mut constraints = FieldConstraints {
        required: schema.constraints.required,
        nullable: schema.constraints.nullable || schema.value_kind == ValueKind::Json,
        allowed_values: schema
            .constraints
            .enum_values
            .clone()
            .unwrap_or_default()
            .iter()
            .map(json_to_generic_value)
            .collect(),
        min_number: None,
        max_number: None,
        min_length: None,
        max_length: None,
    };
    match schema.constraints.numeric_range {
        FieldNumericRange::None => {}
        FieldNumericRange::Integer(range) => {
            constraints.min_number = range.minimum.map(integer_minimum);
            constraints.max_number = range.maximum.map(integer_maximum);
        }
        FieldNumericRange::Number(range) => {
            constraints.min_number = range.minimum.map(|bound| bound.value);
            constraints.max_number = range.maximum.map(|bound| bound.value);
        }
    }
    if let Some(length) = schema.constraints.length_range {
        constraints.min_length = length
            .minimum
            .map(|bound| usize::try_from(bound.value).unwrap_or(usize::MAX));
        constraints.max_length = length
            .maximum
            .map(|bound| usize::try_from(bound.value).unwrap_or(usize::MAX));
    }
    constraints
}

fn integer_minimum(bound: FieldBound<i64>) -> f64 {
    let value = if bound.kind == FieldBoundKind::Open {
        bound.value.saturating_add(1)
    } else {
        bound.value
    };
    value as f64
}

fn integer_maximum(bound: FieldBound<i64>) -> f64 {
    let value = if bound.kind == FieldBoundKind::Open {
        bound.value.saturating_sub(1)
    } else {
        bound.value
    };
    value as f64
}

fn generic_default(schema: &SchemaMetadataView) -> Result<GenericDefaultMetadata, NavigationError> {
    match &schema.default {
        DefaultMetadata::None => Ok(GenericDefaultMetadata::None),
        DefaultMetadata::Static(value) => {
            Ok(GenericDefaultMetadata::Static(json_to_generic_value(value)))
        }
    }
}

fn generic_value_kind(value_kind: ValueKind) -> GenericValueKind {
    match value_kind {
        ValueKind::String => GenericValueKind::String,
        ValueKind::Integer => GenericValueKind::Integer,
        ValueKind::Number => GenericValueKind::Number,
        ValueKind::Boolean => GenericValueKind::Boolean,
        ValueKind::Array => GenericValueKind::List,
        ValueKind::Object => GenericValueKind::Map,
        ValueKind::Json => GenericValueKind::Any,
    }
}

fn collect_processing_candidates(
    candidates: &mut Vec<SourceCandidate>,
    metadata: &[ProcessingMetadataView],
    root: &Value,
    source: &SourceSpec,
    source_kind: ParameterSourceKind,
) -> Result<(), NavigationError> {
    for field in metadata {
        let Some(value) = value_at_path(root, &field.path) else {
            continue;
        };
        let state = candidate_state(field, value);
        let locator = match source_kind {
            ParameterSourceKind::DirectInput => {
                SourceLocator::CliFlag(field.path.segments().join("."))
            }
            ParameterSourceKind::ProjectConfig | ParameterSourceKind::UserConfig => {
                SourceLocator::ConfigPath(config_path(&field.path)?)
            }
            ParameterSourceKind::Default => {
                SourceLocator::Default(field.identity.as_str().to_owned())
            }
        };
        candidates.push(SourceCandidate::new(
            generic_identity(&field.identity)?,
            source,
            locator,
            state,
        ));
    }
    Ok(())
}

fn candidate_state(field: &ProcessingMetadataView, value: &Value) -> CandidateState {
    if value.is_null() && !field.constraints.required && field.value_kind != ValueKind::Json {
        CandidateState::ExplicitAbsent
    } else {
        CandidateState::Present(json_to_generic_value(value))
    }
}

fn collect_default_candidates(
    candidates: &mut Vec<SourceCandidate>,
    metadata: &[SchemaMetadataView],
    source: &SourceSpec,
) -> Result<(), NavigationError> {
    for field in metadata {
        let DefaultMetadata::Static(value) = &field.default else {
            continue;
        };
        candidates.push(SourceCandidate::new(
            generic_identity(&field.identity)?,
            source,
            SourceLocator::Default(field.identity.as_str().to_owned()),
            CandidateState::DefaultFallback {
                value: json_to_generic_value(value),
                dynamic: false,
            },
        ));
    }
    Ok(())
}

fn generic_sources() -> Result<SourceCollection, NavigationError> {
    SourceCollection::new(vec![
        SourceSpec::new(source_id(EXPLICIT_SOURCE_ID)?, SourceKind::Cli, 400),
        SourceSpec::new(source_id(PROJECT_SOURCE_ID)?, SourceKind::Config, 300),
        SourceSpec::new(source_id(USER_SOURCE_ID)?, SourceKind::Config, 200),
        SourceSpec::new(source_id(BUILT_IN_SOURCE_ID)?, SourceKind::Default, 100),
    ])
    .map_err(|_| NavigationError::internal("cli-config-source-collection-build-failed"))
}

fn source_spec<'a>(
    sources: &'a SourceCollection,
    id: &str,
) -> Result<&'a SourceSpec, NavigationError> {
    let id = source_id(id)?;
    sources
        .get(&id)
        .ok_or_else(|| NavigationError::internal("cli-config-source-missing"))
}

fn source_id(id: &str) -> Result<SourceId, NavigationError> {
    SourceId::new(id).map_err(|_| NavigationError::internal("cli-config-source-id-invalid"))
}

fn generic_identity(identity: &FieldIdentity) -> Result<GenericFieldIdentity, NavigationError> {
    GenericFieldIdentity::new(identity.as_str())
        .map_err(|_| NavigationError::internal("cli-config-field-identity-invalid"))
}

fn typed_identity(identity: &GenericFieldIdentity) -> Result<FieldIdentity, NavigationError> {
    FieldIdentity::new(identity.as_str())
        .map_err(|_| NavigationError::internal("navigation-field-identity-invalid"))
}

fn config_path(path: &FieldPath) -> Result<cli_config_resolution::ConfigPath, NavigationError> {
    cli_config_resolution::ConfigPath::new(path.segments())
        .map_err(|_| NavigationError::internal("cli-config-path-invalid"))
}

fn value_at_path<'a>(root: &'a Value, path: &FieldPath) -> Option<&'a Value> {
    let mut current = root;
    for segment in path.segments() {
        current = current.get(segment)?;
    }
    Some(current)
}

fn json_to_generic_value(value: &Value) -> GenericValue {
    match value {
        Value::String(value) => GenericValue::String(value.clone()),
        Value::Number(value) => value
            .as_i64()
            .map(GenericValue::Integer)
            .or_else(|| value.as_f64().map(GenericValue::Number))
            .unwrap_or(GenericValue::Null),
        Value::Bool(value) => GenericValue::Boolean(*value),
        Value::Array(values) => {
            GenericValue::List(values.iter().map(json_to_generic_value).collect())
        }
        Value::Object(values) => GenericValue::Map(
            values
                .iter()
                .map(|(key, value)| (key.clone(), json_to_generic_value(value)))
                .collect(),
        ),
        Value::Null => GenericValue::Null,
    }
}

fn generic_value_to_json(value: &GenericValue) -> Value {
    match value {
        GenericValue::String(value) => Value::String(value.clone()),
        GenericValue::Integer(value) => Value::Number(serde_json::Number::from(*value)),
        GenericValue::Number(value) => serde_json::Number::from_f64(*value)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        GenericValue::Boolean(value) => Value::Bool(*value),
        GenericValue::List(values) => {
            Value::Array(values.iter().map(generic_value_to_json).collect())
        }
        GenericValue::Map(values) => Value::Object(
            values
                .iter()
                .map(|(key, value)| (key.clone(), generic_value_to_json(value)))
                .collect(),
        ),
        GenericValue::Null => Value::Null,
    }
}
