use std::collections::BTreeSet;

use serde_json::{Map, Value};

use super::{FieldDefSet, FieldExtractionError, JsonExtractionDefaults};
use crate::processing::{ProcessedExtraction, ProcessedValue, ProcessingId};
use crate::JsonPassthroughProcessing;

impl FieldDefSet {
    pub fn validate_json_with_passthrough(
        &self,
        processing_id: impl Into<ProcessingId>,
        root: &Value,
        passthrough_processing: Option<&JsonPassthroughProcessing<'_>>,
    ) -> ProcessedExtraction<Result<(), FieldExtractionError>, Value> {
        let processed = self.__extract_json_values_with_passthrough(
            processing_id,
            root,
            passthrough_processing,
        );
        let (extraction, processing) = processed.into_parts();
        ProcessedExtraction::new(extraction.map(|_| ()), processing)
    }

    pub fn unused_json_fields<I, S>(
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
        self.require_json_processing(&processing_id)?;
        let object_path = object_path.into_iter().map(Into::into).collect::<Vec<_>>();
        let Some(Value::Object(object)) = value_at_owned_path(root, &object_path) else {
            return Ok(Value::Object(Map::new()));
        };
        let declared = self.declared_json_child_keys(&processing_id, &object_path);
        Ok(Value::Object(
            object
                .iter()
                .filter(|(key, _)| !declared.contains(key.as_str()))
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
        ))
    }

    #[doc(hidden)]
    pub fn __extract_json_values_with_passthrough(
        &self,
        processing_id: impl Into<ProcessingId>,
        root: &Value,
        passthrough_processing: Option<&JsonPassthroughProcessing<'_>>,
    ) -> ProcessedExtraction<Result<super::FieldValues, FieldExtractionError>, Value> {
        let processing_id = passthrough_processing
            .map(|processing| processing.id().clone())
            .unwrap_or_else(|| processing_id.into());
        let values = self.require_json_processing(&processing_id).and_then(|()| {
            self.extract_json_processing_values(
                &processing_id,
                root,
                JsonExtractionDefaults::Absent,
            )
        });
        let processing = match passthrough_processing {
            Some(processing) => processing.process(root.clone()),
            None => ProcessedValue::new(processing_id, root.clone()),
        };
        ProcessedExtraction::new(values, processing)
    }

    fn declared_json_child_keys(
        &self,
        processing_id: &ProcessingId,
        object_path: &[String],
    ) -> BTreeSet<String> {
        self.processing_metadata(processing_id)
            .into_iter()
            .filter_map(|metadata| direct_child_key(metadata.path.segments(), object_path))
            .collect()
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
