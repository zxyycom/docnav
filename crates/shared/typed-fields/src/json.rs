use std::collections::BTreeSet;

use serde_json::{Map, Value};

use crate::process_strategy::ProcessingInputKind;
use crate::processing::{ProcessedExtraction, ProcessedValue, ProcessingId};
use crate::set::{FieldDefSet, FieldExtractionError, FieldValidationErrors};
use crate::JsonPassthroughProcessing;

#[derive(Clone, Copy, Debug)]
pub struct JsonFieldSet<'a> {
    fields: &'a FieldDefSet,
}

impl<'a> JsonFieldSet<'a> {
    pub fn new(fields: &'a FieldDefSet) -> Self {
        Self { fields }
    }

    pub fn validate(
        &self,
        processing_id: impl Into<ProcessingId>,
        root: &Value,
    ) -> Result<(), FieldExtractionError> {
        validate_processing_values(self.fields, processing_id.into(), root)
    }

    pub fn validate_with_passthrough(
        &self,
        processing_id: impl Into<ProcessingId>,
        root: &Value,
        passthrough_processing: Option<&JsonPassthroughProcessing<'_>>,
    ) -> ProcessedExtraction<Result<(), FieldExtractionError>, Value> {
        let processing_id = passthrough_processing
            .map(|processing| processing.id().clone())
            .unwrap_or_else(|| processing_id.into());
        let validation = validate_processing_values(self.fields, processing_id.clone(), root);
        let processing = match passthrough_processing {
            Some(processing) => processing.process(root.clone()),
            None => ProcessedValue::new(processing_id, root.clone()),
        };
        ProcessedExtraction::new(validation, processing)
    }

    pub fn unused_fields<I, S>(
        &self,
        processing_id: impl Into<ProcessingId>,
        root: &Value,
        object_path: I,
    ) -> Result<Value, FieldExtractionError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let processing_id = processing_id.into();
        self.fields
            .require_processing_input_kind(&processing_id, ProcessingInputKind::JsonValue)?;
        let object_path = object_path.into_iter().map(Into::into).collect::<Vec<_>>();
        let Some(Value::Object(object)) = value_at_owned_path(root, &object_path) else {
            return Ok(Value::Object(Map::new()));
        };
        let declared = self.declared_child_keys(&processing_id, &object_path);
        Ok(Value::Object(
            object
                .iter()
                .filter(|(key, _)| !declared.contains(key.as_str()))
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
        ))
    }

    fn declared_child_keys(
        &self,
        processing_id: &ProcessingId,
        object_path: &[String],
    ) -> BTreeSet<String> {
        self.fields
            .processing_metadata(processing_id)
            .into_iter()
            .filter_map(|metadata| direct_child_key(metadata.path.segments(), object_path))
            .collect()
    }
}

fn validate_processing_values(
    fields: &FieldDefSet,
    processing_id: ProcessingId,
    root: &Value,
) -> Result<(), FieldExtractionError> {
    fields.require_processing_input_kind(&processing_id, ProcessingInputKind::JsonValue)?;
    let mut errors = Vec::new();
    for definition in fields.fields() {
        if let Err(error) = definition.decode_json_process(&processing_id, root) {
            errors.push(error);
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(FieldExtractionError::Validation(
            FieldValidationErrors::new(errors),
        ))
    }
}

fn value_at_owned_path<'a>(root: &'a Value, path: &[String]) -> Option<&'a Value> {
    let mut current = root;
    for segment in path {
        let Value::Object(object) = current else {
            return None;
        };
        current = object.get(segment)?;
    }
    Some(current)
}

fn direct_child_key(path: Vec<&str>, object_path: &[String]) -> Option<String> {
    if path.len() <= object_path.len() {
        return None;
    }
    if object_path
        .iter()
        .zip(path.iter())
        .all(|(expected, actual)| expected.as_str() == *actual)
    {
        Some(path[object_path.len()].to_string())
    } else {
        None
    }
}
