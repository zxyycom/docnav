use std::collections::BTreeSet;

use serde_json::{Map, Value};

use crate::process_strategy::ProcessingInputKind;
use crate::processing::{ProcessedExtraction, ProcessedValue, ProcessingBuild, ProcessingId};
use crate::set::{FieldDefSet, FieldExtractionError, FieldValidationErrors, FieldValues};
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
        extract_values(self.fields, processing_id.into(), root).map(|_| ())
    }

    pub fn validate_with_passthrough(
        &self,
        processing_id: impl Into<ProcessingId>,
        root: &Value,
        passthrough_processing: Option<&JsonPassthroughProcessing<'_>>,
    ) -> ProcessedExtraction<Result<(), FieldExtractionError>, Value> {
        let processed = extract_values_with_passthrough(
            self.fields,
            processing_id,
            root,
            passthrough_processing,
        );
        let (values, processing) = processed.into_parts();
        ProcessedExtraction::new(values.map(|_| ()), processing)
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

pub(crate) fn extract_values(
    fields: &FieldDefSet,
    processing_id: ProcessingId,
    root: &Value,
) -> Result<FieldValues, FieldExtractionError> {
    fields.require_processing_input_kind(&processing_id, ProcessingInputKind::JsonValue)?;
    extract_processing_values(fields, &processing_id, root, JsonExtractionDefaults::Absent)
}

pub(crate) fn extract_values_with_static_defaults(
    fields: &FieldDefSet,
    processing_id: ProcessingId,
    root: &Value,
) -> Result<FieldValues, FieldExtractionError> {
    fields.require_processing_input_kind(&processing_id, ProcessingInputKind::JsonValue)?;
    extract_processing_values(fields, &processing_id, root, JsonExtractionDefaults::Static)
}

pub(crate) fn process_values<O>(
    fields: &FieldDefSet,
    processing: &ProcessingBuild<'_, Value, O>,
    root: &Value,
) -> ProcessedExtraction<Result<FieldValues, FieldExtractionError>, O> {
    process_values_with_defaults(fields, processing, root, JsonExtractionDefaults::Absent)
}

pub(crate) fn process_values_with_static_defaults<O>(
    fields: &FieldDefSet,
    processing: &ProcessingBuild<'_, Value, O>,
    root: &Value,
) -> ProcessedExtraction<Result<FieldValues, FieldExtractionError>, O> {
    process_values_with_defaults(fields, processing, root, JsonExtractionDefaults::Static)
}

pub(crate) fn extract_values_with_passthrough(
    fields: &FieldDefSet,
    processing_id: impl Into<ProcessingId>,
    root: &Value,
    passthrough_processing: Option<&JsonPassthroughProcessing<'_>>,
) -> ProcessedExtraction<Result<FieldValues, FieldExtractionError>, Value> {
    let processing_id = passthrough_processing
        .map(|processing| processing.id().clone())
        .unwrap_or_else(|| processing_id.into());
    let values = fields
        .require_processing_input_kind(&processing_id, ProcessingInputKind::JsonValue)
        .and_then(|()| {
            extract_processing_values(fields, &processing_id, root, JsonExtractionDefaults::Absent)
        });
    let processing = match passthrough_processing {
        Some(processing) => processing.process(root.clone()),
        None => ProcessedValue::new(processing_id, root.clone()),
    };
    ProcessedExtraction::new(values, processing)
}

fn process_values_with_defaults<O>(
    fields: &FieldDefSet,
    processing: &ProcessingBuild<'_, Value, O>,
    root: &Value,
    defaults: JsonExtractionDefaults,
) -> ProcessedExtraction<Result<FieldValues, FieldExtractionError>, O> {
    let values = fields
        .require_processing_input_kind(processing.id(), ProcessingInputKind::JsonValue)
        .and_then(|()| extract_processing_values(fields, processing.id(), root, defaults));
    ProcessedExtraction::new(values, processing.process(root.clone()))
}

fn extract_processing_values(
    fields: &FieldDefSet,
    processing_id: &ProcessingId,
    root: &Value,
    defaults: JsonExtractionDefaults,
) -> Result<FieldValues, FieldExtractionError> {
    let mut values = Vec::with_capacity(fields.fields().len());
    let mut errors = Vec::new();
    for definition in fields.fields() {
        let decoded = match defaults {
            JsonExtractionDefaults::Absent => definition.decode_json_process(processing_id, root),
            JsonExtractionDefaults::Static => {
                definition.decode_json_process_with_static_default(processing_id, root)
            }
        };
        match decoded {
            Ok(value) => values.push(value),
            Err(error) => errors.push(error),
        }
    }
    if errors.is_empty() {
        Ok(FieldValues { values })
    } else {
        Err(FieldExtractionError::Validation(
            FieldValidationErrors::new(errors),
        ))
    }
}

#[derive(Clone, Copy)]
enum JsonExtractionDefaults {
    Absent,
    Static,
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

#[doc(hidden)]
pub mod __private {
    use crate::{
        FieldDefSet, FieldExtractionError, JsonPassthroughProcessing, JsonValue,
        ProcessedExtraction, ProcessingBuild, ProcessingId,
    };

    use super::FieldValues;

    pub fn extract_values(
        fields: &FieldDefSet,
        processing_id: ProcessingId,
        root: &JsonValue,
    ) -> Result<FieldValues, FieldExtractionError> {
        super::extract_values(fields, processing_id, root)
    }

    pub fn extract_values_with_static_defaults(
        fields: &FieldDefSet,
        processing_id: ProcessingId,
        root: &JsonValue,
    ) -> Result<FieldValues, FieldExtractionError> {
        super::extract_values_with_static_defaults(fields, processing_id, root)
    }

    pub fn process_values<O>(
        fields: &FieldDefSet,
        processing: &ProcessingBuild<'_, JsonValue, O>,
        root: &JsonValue,
    ) -> ProcessedExtraction<Result<FieldValues, FieldExtractionError>, O> {
        super::process_values(fields, processing, root)
    }

    pub fn process_values_with_static_defaults<O>(
        fields: &FieldDefSet,
        processing: &ProcessingBuild<'_, JsonValue, O>,
        root: &JsonValue,
    ) -> ProcessedExtraction<Result<FieldValues, FieldExtractionError>, O> {
        super::process_values_with_static_defaults(fields, processing, root)
    }

    pub fn extract_values_with_passthrough(
        fields: &FieldDefSet,
        processing_id: impl Into<ProcessingId>,
        root: &JsonValue,
        passthrough_processing: Option<&JsonPassthroughProcessing<'_>>,
    ) -> ProcessedExtraction<Result<FieldValues, FieldExtractionError>, JsonValue> {
        super::extract_values_with_passthrough(fields, processing_id, root, passthrough_processing)
    }
}
