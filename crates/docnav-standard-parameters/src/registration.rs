use std::collections::BTreeMap;
use std::fmt;

use docnav_typed_fields::{ExtractionStrategyId, FieldIdentity, SchemaMetadataView};

use crate::StandardParameterPath;

#[derive(Clone, Debug, PartialEq)]
pub struct StandardParameterBinding {
    pub strategy_id: ExtractionStrategyId,
    pub path: StandardParameterPath,
}

impl StandardParameterBinding {
    pub fn new(strategy_id: impl Into<ExtractionStrategyId>, path: StandardParameterPath) -> Self {
        Self {
            strategy_id: strategy_id.into(),
            path,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OperationArgumentBinding {
    pub arguments_path: StandardParameterPath,
}

impl OperationArgumentBinding {
    pub fn new(arguments_path: StandardParameterPath) -> Self {
        Self { arguments_path }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StandardParameterRegistration {
    pub metadata: SchemaMetadataView,
    pub direct_input: Option<StandardParameterBinding>,
    pub config: Option<StandardParameterBinding>,
    pub operation_argument: Option<OperationArgumentBinding>,
}

impl StandardParameterRegistration {
    pub fn new(metadata: SchemaMetadataView) -> Self {
        Self {
            metadata,
            direct_input: None,
            config: None,
            operation_argument: None,
        }
    }

    pub fn with_direct_input_binding(mut self, binding: StandardParameterBinding) -> Self {
        self.direct_input = Some(binding);
        self
    }

    pub fn without_direct_input_binding(mut self) -> Self {
        self.direct_input = None;
        self
    }

    pub fn with_config_binding(mut self, binding: StandardParameterBinding) -> Self {
        self.config = Some(binding);
        self
    }

    pub fn without_config_binding(mut self) -> Self {
        self.config = None;
        self
    }

    pub fn with_operation_argument(mut self, binding: OperationArgumentBinding) -> Self {
        self.operation_argument = Some(binding);
        self
    }

    pub fn identity(&self) -> &FieldIdentity {
        &self.metadata.identity
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StandardParameterRegistrationSet {
    registrations: Vec<StandardParameterRegistration>,
}

impl StandardParameterRegistrationSet {
    pub fn new(
        registrations: Vec<StandardParameterRegistration>,
    ) -> Result<Self, StandardParameterRegistrationSetError> {
        validate_registrations(&registrations)?;
        Ok(Self { registrations })
    }

    pub fn as_slice(&self) -> &[StandardParameterRegistration] {
        &self.registrations
    }

    pub fn into_vec(self) -> Vec<StandardParameterRegistration> {
        self.registrations
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardParameterRegistrationConflictKind {
    Identity,
    DirectInputPath,
    ConfigPath,
    OperationArgumentPath,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StandardParameterRegistrationSetError {
    pub kind: StandardParameterRegistrationConflictKind,
    pub identity: FieldIdentity,
    pub previous_identity: FieldIdentity,
    pub path: Option<StandardParameterPath>,
}

impl fmt::Display for StandardParameterRegistrationSetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.path {
            Some(path) => write!(
                formatter,
                "{:?} conflict for {} at {} (previously used by {})",
                self.kind,
                self.identity.as_str(),
                path.segments().join("."),
                self.previous_identity.as_str()
            ),
            None => write!(
                formatter,
                "{:?} conflict for {} (previously used by {})",
                self.kind,
                self.identity.as_str(),
                self.previous_identity.as_str()
            ),
        }
    }
}

impl std::error::Error for StandardParameterRegistrationSetError {}

fn validate_registrations(
    registrations: &[StandardParameterRegistration],
) -> Result<(), StandardParameterRegistrationSetError> {
    let mut identities = BTreeMap::new();
    let mut direct_paths = BTreeMap::new();
    let mut config_paths = BTreeMap::new();
    let mut operation_paths = BTreeMap::new();

    for registration in registrations {
        let identity = registration.identity().clone();
        if let Some(previous) = identities.insert(identity.clone(), identity.clone()) {
            return Err(StandardParameterRegistrationSetError {
                kind: StandardParameterRegistrationConflictKind::Identity,
                identity,
                previous_identity: previous,
                path: None,
            });
        }
        check_path_conflict(
            &mut direct_paths,
            StandardParameterRegistrationConflictKind::DirectInputPath,
            &identity,
            registration
                .direct_input
                .as_ref()
                .map(|binding| &binding.path),
        )?;
        check_path_conflict(
            &mut config_paths,
            StandardParameterRegistrationConflictKind::ConfigPath,
            &identity,
            registration.config.as_ref().map(|binding| &binding.path),
        )?;
        check_path_conflict(
            &mut operation_paths,
            StandardParameterRegistrationConflictKind::OperationArgumentPath,
            &identity,
            registration
                .operation_argument
                .as_ref()
                .map(|binding| &binding.arguments_path),
        )?;
    }

    Ok(())
}

fn check_path_conflict(
    paths: &mut BTreeMap<Vec<String>, FieldIdentity>,
    kind: StandardParameterRegistrationConflictKind,
    identity: &FieldIdentity,
    path: Option<&StandardParameterPath>,
) -> Result<(), StandardParameterRegistrationSetError> {
    let Some(path) = path else {
        return Ok(());
    };
    let key = path.key();
    if let Some(previous) = paths.insert(key, identity.clone()) {
        return Err(StandardParameterRegistrationSetError {
            kind,
            identity: identity.clone(),
            previous_identity: previous,
            path: Some(path.clone()),
        });
    }
    Ok(())
}
