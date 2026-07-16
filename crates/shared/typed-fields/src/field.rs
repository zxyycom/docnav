use std::collections::BTreeMap;
use std::marker::PhantomData;

use serde_json::Value;

pub(crate) mod constraints;

use crate::metadata::{
    BuildError, DefaultMetadata, FieldConstraints, FieldIdentity, FieldPath, MergeStrategy,
    ProcessingMetadataView, SchemaMetadataView, TypedValue, ValidationFailure, ValueKind,
};
use crate::process_strategy::{BuiltProcessStrategy, ProcessStrategy, ProcessingInputKind};
use crate::{processing::ProcessingId, validation::FieldValidation, value::FieldValue};
use constraints::{
    validate_length_range, validate_numeric_range, validate_regex_pattern, value_at_path,
};

#[derive(Clone, Debug)]
pub struct FieldDef {
    pub(crate) identity: FieldIdentity,
    pub(crate) path: FieldPath,
    value_kind: ValueKind,
    constraints: FieldConstraints,
    default: DefaultMetadata,
    merge_strategy: MergeStrategy,
    processes: BTreeMap<ProcessingId, BuiltProcessStrategy>,
}

impl FieldDef {
    pub fn builder(identity: impl Into<String>) -> FieldDefBuilder {
        FieldDefBuilder::new(identity)
    }

    pub fn identity(&self) -> &FieldIdentity {
        &self.identity
    }

    pub fn schema_metadata(&self) -> SchemaMetadataView {
        self.schema_metadata_with_path(self.path.clone())
    }

    pub fn processing_metadata(
        &self,
        processing_id: &ProcessingId,
    ) -> Option<ProcessingMetadataView> {
        let process = self.processes.get(processing_id)?;
        let path = process
            .json_path()
            .cloned()
            .unwrap_or_else(|| identity_path(&self.identity));
        Some(ProcessingMetadataView {
            identity: self.identity.clone(),
            processing_id: processing_id.clone(),
            path,
            input_kind: process.input_kind(),
            locator: process.locator(),
            value_kind: self.value_kind,
            constraints: self.constraints.clone(),
            default: self.default.clone(),
            merge_strategy: self.merge_strategy,
            cli: process.cli_metadata().cloned(),
        })
    }

    pub fn merge_strategy(&self) -> MergeStrategy {
        self.merge_strategy
    }

    pub fn validate_value(&self, value: &Value) -> Result<TypedValue, ValidationFailure> {
        self.schema_metadata().validate_value(value)
    }

    fn schema_metadata_with_path(&self, path: FieldPath) -> SchemaMetadataView {
        SchemaMetadataView {
            identity: self.identity.clone(),
            path,
            value_kind: self.value_kind,
            constraints: self.constraints.clone(),
            default: self.default.clone(),
            merge_strategy: self.merge_strategy,
        }
    }

    pub(crate) fn static_default_value(&self) -> Option<TypedValue> {
        self.schema_metadata()
            .static_default_value()
            .expect("static default metadata is validated during field build")
    }

    pub(crate) fn apply_declaration_presence(&mut self, required: bool, nullable: bool) {
        self.constraints.required = required;
        self.constraints.nullable = nullable;
    }

    pub(crate) fn processing_input_kinds(
        &self,
    ) -> impl Iterator<Item = (&ProcessingId, ProcessingInputKind)> {
        self.processes
            .iter()
            .map(|(processing_id, process)| (processing_id, process.input_kind()))
    }

    pub(crate) fn decode_json_process(
        &self,
        processing_id: &ProcessingId,
        root: &Value,
    ) -> Result<Option<TypedValue>, ValidationFailure> {
        let Some(process) = self.processes.get(processing_id) else {
            return self.schema_metadata().validate_optional_value(None);
        };
        self.validate_json_process_value(process, root, false)
    }

    pub(crate) fn decode_json_process_with_static_default(
        &self,
        processing_id: &ProcessingId,
        root: &Value,
    ) -> Result<Option<TypedValue>, ValidationFailure> {
        let Some(process) = self.processes.get(processing_id) else {
            return self
                .schema_metadata()
                .validate_optional_value_with_static_default(None);
        };
        self.validate_json_process_value(process, root, true)
    }

    fn validate_json_process_value(
        &self,
        process: &BuiltProcessStrategy,
        root: &Value,
        use_static_default: bool,
    ) -> Result<Option<TypedValue>, ValidationFailure> {
        let Some(path) = process.json_path() else {
            return self.schema_metadata().validate_optional_value(None);
        };
        let metadata = self.schema_metadata_with_path(path.clone());
        if use_static_default {
            metadata.validate_optional_value_with_static_default(value_at_path(root, path))
        } else {
            metadata.validate_optional_value(value_at_path(root, path))
        }
    }
}

#[derive(Debug)]
pub struct FieldDefBuilder<T = ()> {
    identity: String,
    processes: Vec<(String, ProcessStrategy)>,
    validation: Option<FieldValidation<T>>,
    default: Result<DefaultMetadata, BuildError>,
    merge_strategy: MergeStrategy,
    typed: PhantomData<T>,
}

impl FieldDefBuilder<()> {
    fn new(identity: impl Into<String>) -> Self {
        Self {
            identity: identity.into(),
            processes: Vec::new(),
            validation: None,
            default: Ok(DefaultMetadata::None),
            merge_strategy: MergeStrategy::Replace,
            typed: PhantomData,
        }
    }
}

impl<T> Clone for FieldDefBuilder<T> {
    fn clone(&self) -> Self {
        Self {
            identity: self.identity.clone(),
            processes: self.processes.clone(),
            validation: self.validation.clone(),
            default: self.default.clone(),
            merge_strategy: self.merge_strategy,
            typed: PhantomData,
        }
    }
}

impl<T> FieldDefBuilder<T> {
    pub fn process(mut self, processing_id: impl Into<String>, process: ProcessStrategy) -> Self {
        self.processes.push((processing_id.into(), process));
        self
    }

    /// Updates an existing processing strategy or adds it when it is not yet declared.
    pub fn set_process(
        mut self,
        processing_id: impl Into<String>,
        process: ProcessStrategy,
    ) -> Self {
        let id = processing_id.into();
        let existing = self.processes.iter_mut().find(|(key, _)| key == &id);
        match existing {
            Some((_, existing)) => *existing = process,
            None => self.processes.push((id, process)),
        }
        self
    }

    pub fn validation<U>(self, validation: FieldValidation<U>) -> FieldDefBuilder<U> {
        FieldDefBuilder {
            identity: self.identity,
            processes: self.processes,
            validation: Some(validation),
            default: self.default,
            merge_strategy: self.merge_strategy,
            typed: PhantomData,
        }
    }

    pub fn merge(mut self, merge_strategy: MergeStrategy) -> Self {
        self.merge_strategy = merge_strategy;
        self
    }

    pub fn default_static(mut self, value: impl Into<T>) -> Self
    where
        T: FieldValue,
    {
        let value = value.into();
        self.default = value.try_into_json_value().map(DefaultMetadata::Static);
        self
    }

    pub(crate) fn build(self) -> Result<FieldDef, BuildError> {
        let definition = self.into_definition()?;
        definition.validate_enum_metadata()?;
        definition.validate_default_metadata()?;
        Ok(definition)
    }

    fn into_definition(self) -> Result<FieldDef, BuildError> {
        let identity = FieldIdentity::new(self.identity)?;
        let processes = build_processes(self.processes)?;
        let path = metadata_path(&identity, &processes)?;
        let validation = self.validation.ok_or(BuildError::MissingValidation)?;
        let (value_kind, mut constraints) = validation.into_parts();
        constraints.nullable = !constraints.required;
        for process in processes.values() {
            process.validate_cli_metadata(value_kind)?;
        }
        validate_merge_strategy(value_kind, self.merge_strategy)?;
        validate_numeric_range(&constraints)?;
        validate_length_range(&constraints)?;
        validate_regex_pattern(&constraints)?;
        let default = self.default?;

        Ok(FieldDef {
            identity,
            path,
            value_kind,
            constraints,
            default,
            merge_strategy: self.merge_strategy,
            processes,
        })
    }
}

fn build_processes(
    processes: Vec<(String, ProcessStrategy)>,
) -> Result<BTreeMap<ProcessingId, BuiltProcessStrategy>, BuildError> {
    if processes.is_empty() {
        return Err(BuildError::MissingProcessingStrategy);
    }
    let mut declared = BTreeMap::new();
    for (processing_id, process) in processes {
        let processing_id =
            ProcessingId::new(processing_id).map_err(|_| BuildError::EmptyProcessingId)?;
        if declared.insert(processing_id.clone(), process).is_some() {
            return Err(BuildError::DuplicateProcessingId { processing_id });
        }
    }
    declared
        .into_iter()
        .map(|(processing_id, process)| Ok((processing_id, process.build()?)))
        .collect()
}

fn metadata_path(
    identity: &FieldIdentity,
    processes: &BTreeMap<ProcessingId, BuiltProcessStrategy>,
) -> Result<FieldPath, BuildError> {
    if let Some(path) = processes.values().find_map(BuiltProcessStrategy::json_path) {
        return Ok(path.clone());
    }
    FieldPath::new(identity.as_str().split('.'))
}

fn identity_path(identity: &FieldIdentity) -> FieldPath {
    FieldPath::new(identity.as_str().split('.'))
        .expect("field identity is non-empty and split produces non-empty path segments")
}

impl FieldDef {
    fn validate_enum_metadata(&self) -> Result<(), BuildError> {
        let Some(enum_values) = &self.constraints.enum_values else {
            return Ok(());
        };
        if enum_values.is_empty() {
            return Err(BuildError::EmptyEnumValues);
        }
        for value in enum_values {
            self.validate_value(value)
                .map_err(BuildError::InvalidEnumValue)?;
        }
        Ok(())
    }

    fn validate_default_metadata(&self) -> Result<(), BuildError> {
        if let DefaultMetadata::Static(value) = &self.default {
            self.validate_value(value)
                .map_err(BuildError::InvalidDefault)?;
        }
        Ok(())
    }
}

fn validate_merge_strategy(
    value_kind: ValueKind,
    merge_strategy: MergeStrategy,
) -> Result<(), BuildError> {
    let compatible = match merge_strategy {
        MergeStrategy::Replace | MergeStrategy::DenyConflict => true,
        MergeStrategy::Append => value_kind == ValueKind::Array,
        MergeStrategy::MapMerge => value_kind == ValueKind::Object,
    };
    if compatible {
        Ok(())
    } else {
        Err(BuildError::IncompatibleMergeStrategy {
            value_kind,
            merge_strategy,
        })
    }
}
