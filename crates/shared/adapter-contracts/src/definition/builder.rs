use std::collections::BTreeSet;

use docnav_protocol::{Manifest, Operation};

use crate::{Adapter, AdapterOptionSpec, UnstructuredFullReadCapabilities};

use super::{
    AdapterDefinition, AdapterDefinitionError, AdapterOperationHandlers, FullReadCapabilityGroup,
    REQUIRED_OPERATIONS,
};

pub struct AdapterDefinitionBuilder<'a> {
    adapter: Option<&'a dyn Adapter>,
    id: String,
    manifest: Option<Manifest>,
    operation_handlers: Vec<Operation>,
    duplicate_operation_handlers: Vec<Operation>,
    native_options: Vec<AdapterOptionSpec>,
    full_read: Option<FullReadCapabilityGroup>,
    duplicate_full_read: bool,
}

impl<'a> AdapterDefinitionBuilder<'a> {
    pub(super) fn new(id: impl Into<String>) -> Self {
        Self {
            adapter: None,
            id: id.into(),
            manifest: None,
            operation_handlers: Vec::new(),
            duplicate_operation_handlers: Vec::new(),
            native_options: Vec::new(),
            full_read: None,
            duplicate_full_read: false,
        }
    }

    pub fn adapter(mut self, adapter: &'a dyn Adapter) -> Self {
        self.adapter = Some(adapter);
        self
    }

    pub fn manifest(mut self, manifest: Manifest) -> Self {
        self.manifest = Some(manifest);
        self
    }

    pub fn operation_handler(mut self, operation: Operation) -> Self {
        if self.operation_handlers.contains(&operation) {
            self.duplicate_operation_handlers.push(operation);
        }
        self.operation_handlers.push(operation);
        self
    }

    pub fn required_operation_handlers(mut self) -> Self {
        for operation in REQUIRED_OPERATIONS {
            self = self.operation_handler(operation);
        }
        self
    }

    pub fn native_option(mut self, option: AdapterOptionSpec) -> Self {
        self.native_options.push(option);
        self
    }

    pub fn native_options<I>(mut self, options: I) -> Self
    where
        I: IntoIterator<Item = AdapterOptionSpec>,
    {
        self.native_options.extend(options);
        self
    }

    pub fn full_read_capability_group(
        mut self,
        capabilities: UnstructuredFullReadCapabilities,
    ) -> Self {
        if self.full_read.is_some() {
            self.duplicate_full_read = true;
        }
        self.full_read = Some(FullReadCapabilityGroup::new(capabilities));
        self
    }

    pub fn build(self) -> Result<AdapterDefinition<'a>, AdapterDefinitionError> {
        let adapter = self
            .adapter
            .ok_or_else(|| AdapterDefinitionError::MissingAdapter {
                id: self.id.clone(),
            })?;
        let manifest = self
            .manifest
            .ok_or_else(|| AdapterDefinitionError::MissingManifest {
                id: self.id.clone(),
            })?;

        if manifest.adapter.id != self.id {
            return Err(AdapterDefinitionError::ManifestIdMismatch {
                id: self.id.clone(),
                manifest_id: manifest.adapter.id,
            });
        }

        if let Some(operation) = self.duplicate_operation_handlers.first() {
            return Err(AdapterDefinitionError::DuplicateOperationHandler {
                id: self.id.clone(),
                operation: *operation,
            });
        }

        let operations = unique_operations(self.operation_handlers);
        let missing = REQUIRED_OPERATIONS
            .into_iter()
            .filter(|operation| !operations.contains(operation))
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            return Err(AdapterDefinitionError::MissingRequiredHandlers {
                id: self.id.clone(),
                operations: missing,
            });
        }

        validate_native_options(&self.id, &self.native_options)?;
        validate_full_read(&self.id, self.duplicate_full_read, self.full_read.as_ref())?;

        Ok(AdapterDefinition {
            adapter,
            id: self.id,
            manifest,
            operation_handlers: AdapterOperationHandlers::new(operations),
            native_options: self.native_options,
            full_read: self.full_read,
        })
    }
}

fn unique_operations(operations: Vec<Operation>) -> Vec<Operation> {
    let mut seen = BTreeSet::new();
    operations
        .into_iter()
        .filter(|operation| seen.insert(*operation))
        .collect()
}

fn validate_native_options(
    id: &str,
    native_options: &[AdapterOptionSpec],
) -> Result<(), AdapterDefinitionError> {
    let mut identities = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for option in native_options {
        option.validate_declaration().map_err(|error| {
            AdapterDefinitionError::InvalidNativeOption {
                id: id.to_owned(),
                option: option.identity.clone(),
                reason: error.to_string(),
            }
        })?;
        if !identities.insert(option.identity.clone()) {
            return Err(AdapterDefinitionError::DuplicateNativeOptionDeclaration {
                id: id.to_owned(),
                option: option.identity.clone(),
            });
        }
        let key = (
            option.owner.clone(),
            option.namespace().to_owned(),
            option.key().to_owned(),
        );
        if !paths.insert(key.clone()) {
            return Err(AdapterDefinitionError::DuplicateNativeOptionPath {
                id: id.to_owned(),
                owner: key.0,
                namespace: key.1,
                key: key.2,
            });
        }
    }
    Ok(())
}

fn validate_full_read(
    id: &str,
    duplicate_full_read: bool,
    full_read: Option<&FullReadCapabilityGroup>,
) -> Result<(), AdapterDefinitionError> {
    if duplicate_full_read {
        return Err(AdapterDefinitionError::DuplicateCapabilityGroup {
            id: id.to_owned(),
            capability: "full_read",
        });
    }
    let Some(full_read) = full_read else {
        return Ok(());
    };

    let capabilities = full_read.capabilities();
    let has_any_hook = capabilities.content_hook
        || capabilities.result_facts_hook
        || !capabilities.cost_measurement_units.is_empty();
    if !has_any_hook {
        return Err(AdapterDefinitionError::UnsupportedCapabilityCombination {
            id: id.to_owned(),
            capability: "full_read",
            reason: "full-read capability group must declare at least one hook or cost unit",
        });
    }

    let mut units = BTreeSet::new();
    for unit in &capabilities.cost_measurement_units {
        if unit.is_empty() || !units.insert(unit) {
            return Err(AdapterDefinitionError::UnsupportedCapabilityCombination {
                id: id.to_owned(),
                capability: "full_read",
                reason: "full-read cost measurement units must be non-empty and unique",
            });
        }
    }
    Ok(())
}
