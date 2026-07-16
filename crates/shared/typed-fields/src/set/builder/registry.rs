use std::collections::BTreeMap;

use crate::field::FieldDef;
use crate::metadata::{
    FieldDuplicateIdentityError, FieldIdentity, FieldPath, ProcessingMetadataView,
};
use crate::process_strategy::{ProcessingInputKind, ProcessingLocator};
use crate::processing::ProcessingId;

use super::super::super::{
    FieldDefSet, FieldDefSetBuildError, FieldDuplicateProcessingLocatorError,
    FieldDuplicateProcessingPathError,
};
use super::super::declaration::BuiltFieldDeclaration;

struct FieldIdentityLocation {
    declaration_path: Option<Vec<String>>,
    path: FieldPath,
}

struct FieldProcessingPathLocation {
    declaration_path: Option<Vec<String>>,
    identity: FieldIdentity,
}

#[derive(Default)]
struct ProcessingLocationRegistry {
    paths: BTreeMap<(ProcessingId, Vec<String>), FieldProcessingPathLocation>,
    source_locators:
        BTreeMap<(ProcessingId, SourceProcessingLocatorKey), FieldProcessingPathLocation>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum SourceProcessingLocatorKey {
    CliFlag(String),
    EnvVar(String),
}

impl ProcessingLocationRegistry {
    fn register(
        &mut self,
        metadata: ProcessingMetadataView,
        identity: &FieldIdentity,
        declaration_path: &Option<Vec<String>>,
    ) -> Result<(), FieldDefSetBuildError> {
        match &metadata.locator {
            ProcessingLocator::JsonPath(_) | ProcessingLocator::ConfigPath(_) => {
                self.register_path(&metadata, identity, declaration_path)
            }
            ProcessingLocator::CliFlag(flag) => self.register_source_locator(
                &metadata,
                SourceProcessingLocatorKey::CliFlag(flag.clone()),
                identity,
                declaration_path,
            ),
            ProcessingLocator::EnvVar(name) => self.register_source_locator(
                &metadata,
                SourceProcessingLocatorKey::EnvVar(name.clone()),
                identity,
                declaration_path,
            ),
            ProcessingLocator::RustField => Ok(()),
        }
    }

    fn register_path(
        &mut self,
        metadata: &ProcessingMetadataView,
        identity: &FieldIdentity,
        declaration_path: &Option<Vec<String>>,
    ) -> Result<(), FieldDefSetBuildError> {
        let key = (
            metadata.processing_id.clone(),
            metadata.path.raw_segments().to_vec(),
        );
        let current = field_processing_location(identity, declaration_path);
        let Some(previous) = self.paths.insert(key, current) else {
            return Ok(());
        };
        Err(FieldDuplicateProcessingPathError {
            processing_id: metadata.processing_id.clone(),
            path: metadata.path.clone(),
            previous_identity: previous.identity,
            previous_declaration_path: previous.declaration_path,
            current_identity: identity.clone(),
            current_declaration_path: declaration_path.clone(),
        }
        .into())
    }

    fn register_source_locator(
        &mut self,
        metadata: &ProcessingMetadataView,
        locator_key: SourceProcessingLocatorKey,
        identity: &FieldIdentity,
        declaration_path: &Option<Vec<String>>,
    ) -> Result<(), FieldDefSetBuildError> {
        let key = (metadata.processing_id.clone(), locator_key);
        let current = field_processing_location(identity, declaration_path);
        let Some(previous) = self.source_locators.insert(key, current) else {
            return Ok(());
        };
        Err(FieldDuplicateProcessingLocatorError {
            processing_id: metadata.processing_id.clone(),
            locator: metadata.locator.clone(),
            previous_identity: previous.identity,
            previous_declaration_path: previous.declaration_path,
            current_identity: identity.clone(),
            current_declaration_path: declaration_path.clone(),
        }
        .into())
    }
}

fn field_processing_location(
    identity: &FieldIdentity,
    declaration_path: &Option<Vec<String>>,
) -> FieldProcessingPathLocation {
    FieldProcessingPathLocation {
        declaration_path: declaration_path.clone(),
        identity: identity.clone(),
    }
}

#[derive(Default)]
pub(super) struct DefinitionRegistry {
    identities: BTreeMap<FieldIdentity, FieldIdentityLocation>,
    processing_locations: ProcessingLocationRegistry,
    fields: Vec<FieldDef>,
}

impl DefinitionRegistry {
    pub(super) fn register(
        &mut self,
        built: BuiltFieldDeclaration,
    ) -> Result<(), FieldDefSetBuildError> {
        let BuiltFieldDeclaration {
            declaration_path,
            definition,
        } = built;
        self.register_identity(&definition, &declaration_path)?;
        self.register_processing_locations(&definition, &declaration_path)?;
        self.fields.push(definition);
        Ok(())
    }

    fn register_identity(
        &mut self,
        definition: &FieldDef,
        declaration_path: &Option<Vec<String>>,
    ) -> Result<(), FieldDefSetBuildError> {
        let current = FieldIdentityLocation {
            declaration_path: declaration_path.clone(),
            path: definition.path.clone(),
        };
        let Some(previous) = self
            .identities
            .insert(definition.identity().clone(), current)
        else {
            return Ok(());
        };
        Err(FieldDuplicateIdentityError {
            field: definition.identity.clone(),
            path: definition.path.clone(),
            declaration_path: declaration_path.clone(),
            previous_path: previous.path,
            previous_declaration_path: previous.declaration_path,
        }
        .into())
    }

    fn register_processing_locations(
        &mut self,
        definition: &FieldDef,
        declaration_path: &Option<Vec<String>>,
    ) -> Result<(), FieldDefSetBuildError> {
        for (processing_id, _) in definition.processing_input_kinds() {
            let Some(metadata) = definition.processing_metadata(processing_id) else {
                continue;
            };
            self.processing_locations.register(
                metadata,
                definition.identity(),
                declaration_path,
            )?;
        }
        Ok(())
    }

    pub(super) fn finish(self) -> Result<FieldDefSet, FieldDefSetBuildError> {
        let processing_input_kinds = processing_input_kinds(&self.fields)?;
        Ok(FieldDefSet {
            fields: self.fields,
            processing_input_kinds,
        })
    }
}

fn processing_input_kinds(
    fields: &[FieldDef],
) -> Result<BTreeMap<ProcessingId, ProcessingInputKind>, FieldDefSetBuildError> {
    let mut input_kinds = BTreeMap::new();
    for field in fields {
        for (processing_id, input_kind) in field.processing_input_kinds() {
            if let Some(previous) = input_kinds.insert(processing_id.clone(), input_kind) {
                if previous != input_kind {
                    return Err(FieldDefSetBuildError::ProcessingInputKindConflict {
                        processing_id: processing_id.clone(),
                        previous,
                        current: input_kind,
                    });
                }
            }
        }
    }
    Ok(input_kinds)
}
