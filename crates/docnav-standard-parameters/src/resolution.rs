use std::collections::{btree_map::Entry, BTreeMap};

use docnav_typed_fields::{
    FieldIdentity, JsonValue, SchemaMetadataView, TypedValue, ValidationFailure,
};

use crate::{
    EntryPassthroughPolicy, OperationArgumentBinding, PassthroughValue, StandardParameterPath,
    StandardParameterRegistration, StandardParameterSource, StandardParameterSourceInfo,
    StandardParameterSourceKind, StandardParameterSources,
};

#[derive(Clone, Debug, PartialEq)]
pub struct StandardParameterDiagnostic {
    pub identity: FieldIdentity,
    pub source: Option<StandardParameterSourceInfo>,
    pub failure: ValidationFailure,
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
        self.diagnostics.push(StandardParameterDiagnostic {
            identity,
            source,
            failure,
        });
    }
}

pub fn resolve_standard_parameters(
    registrations: &[StandardParameterRegistration],
    sources: StandardParameterSources,
    passthrough_policy: EntryPassthroughPolicy,
) -> StandardParameterResolution {
    let mut resolution = StandardParameterResolution::default();

    for registration in registrations {
        resolve_registration(registration, &sources, &mut resolution);
    }

    resolution.passthrough = resolve_passthrough(&sources, passthrough_policy);
    resolution
}

fn resolve_registration(
    registration: &StandardParameterRegistration,
    sources: &StandardParameterSources,
    resolution: &mut StandardParameterResolution,
) {
    if let Some((value, source)) = first_source_value(registration.identity(), sources) {
        resolve_raw_value(registration, value, source, resolution);
        return;
    }

    match registration.metadata.static_default_value() {
        Ok(Some(value)) => resolution.insert_value(
            registration.identity().clone(),
            value,
            StandardParameterSourceInfo::new(StandardParameterSourceKind::Default),
            registration.operation_argument.clone(),
        ),
        Ok(None) if registration.metadata.is_required() => resolution.push_diagnostic(
            registration.identity().clone(),
            None,
            missing_required_failure(&registration.metadata),
        ),
        Ok(None) => {}
        Err(failure) => resolution.push_diagnostic(
            registration.identity().clone(),
            Some(StandardParameterSourceInfo::new(
                StandardParameterSourceKind::Default,
            )),
            failure,
        ),
    }
}

fn resolve_raw_value(
    registration: &StandardParameterRegistration,
    value: &JsonValue,
    source_kind: StandardParameterSourceKind,
    resolution: &mut StandardParameterResolution,
) {
    let source = StandardParameterSourceInfo::new(source_kind);
    match registration.metadata.validate_value(value) {
        Ok(value) => resolution.insert_value(
            registration.identity().clone(),
            value,
            source,
            registration.operation_argument.clone(),
        ),
        Err(failure) => {
            resolution.push_diagnostic(registration.identity().clone(), Some(source), failure)
        }
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
    let mut passthrough = BTreeMap::new();
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
    collect_passthrough(
        &mut passthrough,
        StandardParameterSourceKind::Default,
        &sources.default,
        policy,
    );
    passthrough.into_values().collect()
}

fn collect_passthrough(
    passthrough: &mut BTreeMap<Vec<String>, PassthroughValue>,
    source_kind: StandardParameterSourceKind,
    source: &StandardParameterSource,
    policy: EntryPassthroughPolicy,
) {
    for input in source.passthrough() {
        match passthrough.entry(input.path.key()) {
            Entry::Vacant(entry) => {
                entry.insert(PassthroughValue {
                    source: StandardParameterSourceInfo::new(source_kind),
                    path: input.path.clone(),
                    value: input.value.clone(),
                    disposition: policy.disposition(),
                });
            }
            Entry::Occupied(_) => {}
        }
    }
}
