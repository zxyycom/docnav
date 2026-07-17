use cli_config_resolution::ResolutionDiagnostic;
use docnav_adapter_contracts::{AdapterError, NativeOptionIssue};
use docnav_diagnostics::AdapterConfigSourceDetails;
use docnav_protocol::Operation;
use docnav_typed_fields::{
    FieldBoundKind, FieldConstraints, FieldDef, FieldNumericRange, FieldRange, SchemaMetadataView,
    ValueKind,
};
use serde_json::Value;

use crate::NavigationConfigSources;

use super::catalog::{DocumentParameterCatalog, DocumentParameterEntry};
use super::values::{diagnostic_source_label, resolution_reason_code};
use super::CONFIG_PROCESSING;

pub(super) struct UnsupportedOptionContext<'a> {
    pub source: &'static str,
    pub path_origin: Option<&'static str>,
    pub path: &'a str,
    pub owner: &'a str,
    pub config_field: Option<String>,
    pub operation: Option<Operation>,
    pub catalog: &'a DocumentParameterCatalog,
}

pub(super) fn native_option_validation_error(
    config_sources: &NavigationConfigSources,
    field: &FieldDef,
    entry: &DocumentParameterEntry,
    diagnostic: &ResolutionDiagnostic,
) -> AdapterError {
    let owner = entry
        .adapter_id()
        .expect("adapter option diagnostics require an adapter-scoped catalog entry");
    let config_path = option_config_path(field)
        .expect("adapter option catalog fields require a canonical config locator");
    let namespace = config_path
        .first()
        .expect("adapter option config path has a namespace");
    let key = config_path
        .last()
        .expect("adapter option config path has a key");
    let source = diagnostic_source_label(diagnostic).unwrap_or("explicit");
    let reason_code = resolution_reason_code(diagnostic);
    let metadata = field.schema_metadata();
    let expected = expected_value_description(&metadata);
    let issue = NativeOptionIssue {
        owner: owner.to_owned(),
        namespace: namespace.clone(),
        key: key.clone(),
        source: source.to_owned(),
        reason_code: reason_code.to_owned(),
        field: format!("arguments.options.{key}"),
        received: diagnostic.raw.as_ref().map(received_value),
        expected: Some(expected.clone()),
        type_variant: Some(value_kind_name(metadata.value_kind()).to_owned()),
        config_source: option_config_source_issue(
            config_sources,
            source,
            reason_code,
            &config_path,
        ),
    };
    AdapterError::native_option_invalid(
        "Native option value is invalid.",
        issue,
        [format!("Use {expected} for option {key}.")],
    )
}

pub(super) fn unsupported_option(
    context: UnsupportedOptionContext<'_>,
    key: &str,
    value: Value,
) -> AdapterError {
    let issue = NativeOptionIssue {
        owner: context.owner.to_owned(),
        namespace: "options".to_owned(),
        key: key.to_owned(),
        source: context.source.to_owned(),
        reason_code: "unsupported".to_owned(),
        field: format!("arguments.options.{key}"),
        received: Some(received_value(&value)),
        expected: Some(supported_option_keys(
            context.catalog,
            context.owner,
            context.operation,
        )),
        type_variant: None,
        config_source: context.path_origin.map(|path_origin| {
            let field = context
                .config_field
                .unwrap_or_else(|| format!("options.{key}"));
            AdapterConfigSourceDetails::new(
                context.source,
                path_origin,
                context.path,
                "unsupported",
            )
            .with_field(field)
        }),
    };
    AdapterError::native_option_invalid(
        "Native option is not supported by the selected adapter.",
        issue,
        [format!(
            "Remove option {key} from {} or select an adapter that supports it.",
            context.path
        )],
    )
}

pub(super) fn catalog_option_for_identity<'a>(
    catalog: &'a DocumentParameterCatalog,
    selected_adapter_id: &'a str,
    operation: Operation,
    identity: &str,
) -> Option<(&'a FieldDef, &'a DocumentParameterEntry)> {
    catalog
        .selected_operation_parameters(selected_adapter_id, operation)
        .find_map(|(field, entry, _)| {
            (entry.adapter_id() == Some(selected_adapter_id)
                && field.identity().as_str() == identity)
                .then_some((field, entry))
        })
}

pub(super) fn selected_adapter_supports_option(
    catalog: &DocumentParameterCatalog,
    selected_adapter_id: &str,
    operation: Operation,
    key: &str,
) -> bool {
    selected_option_keys(catalog, selected_adapter_id, operation).any(|candidate| candidate == key)
}

fn option_config_source_issue(
    config_sources: &NavigationConfigSources,
    source: &str,
    reason_code: &str,
    config_path: &[String],
) -> Option<AdapterConfigSourceDetails> {
    let config_source = match source {
        "project" => &config_sources.project,
        "user" => &config_sources.user,
        _ => return None,
    };
    Some(
        AdapterConfigSourceDetails::new(
            config_source.level.as_str(),
            config_source.origin.as_str(),
            &config_source.path,
            reason_code,
        )
        .with_field(config_path.join(".")),
    )
}

fn supported_option_keys(
    catalog: &DocumentParameterCatalog,
    owner: &str,
    operation: Option<Operation>,
) -> String {
    let keys = match operation {
        Some(operation) => selected_option_keys(catalog, owner, operation).collect::<Vec<_>>(),
        None => catalog
            .entries()
            .iter()
            .filter(|entry| entry.adapter_id() == Some(owner))
            .filter_map(|entry| catalog.fields().field(entry.identity()))
            .filter_map(option_key)
            .collect(),
    };
    if keys.is_empty() {
        "no native options".to_owned()
    } else {
        keys.join(", ")
    }
}

fn selected_option_keys<'a>(
    catalog: &'a DocumentParameterCatalog,
    selected_adapter_id: &'a str,
    operation: Operation,
) -> impl Iterator<Item = String> + 'a {
    catalog
        .selected_operation_parameters(selected_adapter_id, operation)
        .filter_map(move |(field, entry, _)| {
            (entry.adapter_id() == Some(selected_adapter_id))
                .then(|| option_key(field))
                .flatten()
        })
}

fn option_key(field: &FieldDef) -> Option<String> {
    option_config_path(field)?.last().cloned()
}

fn option_config_path(field: &FieldDef) -> Option<Vec<String>> {
    let processing_id =
        docnav_typed_fields::ProcessingId::new(CONFIG_PROCESSING).expect("processing id is valid");
    let path = field.processing_metadata(&processing_id)?.path;
    let segments = path
        .segments()
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    (segments.len() >= 3 && segments.first().is_some_and(|segment| segment == "options"))
        .then_some(segments)
}

fn received_value(value: &Value) -> String {
    value
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| value.to_string())
}

fn expected_value_description(metadata: &SchemaMetadataView<'_>) -> String {
    match metadata.value_kind() {
        ValueKind::String => "a string".to_owned(),
        ValueKind::Integer => integer_description(metadata.constraints()),
        ValueKind::Number => number_description(metadata.constraints()),
        ValueKind::Boolean => "a boolean".to_owned(),
        ValueKind::Array => "an array".to_owned(),
        ValueKind::Object => "an object".to_owned(),
        ValueKind::Json => "a JSON value".to_owned(),
    }
}

fn value_kind_name(kind: ValueKind) -> &'static str {
    match kind {
        ValueKind::String => "string",
        ValueKind::Integer => "integer",
        ValueKind::Number => "number",
        ValueKind::Boolean => "boolean",
        ValueKind::Array => "array",
        ValueKind::Object => "object",
        ValueKind::Json => "json",
    }
}

fn integer_description(constraints: &FieldConstraints) -> String {
    match constraints.numeric_range {
        FieldNumericRange::Integer(range) => {
            range_description("integer", range).unwrap_or_else(|| "an integer".to_owned())
        }
        _ => "an integer".to_owned(),
    }
}

fn number_description(constraints: &FieldConstraints) -> String {
    match constraints.numeric_range {
        FieldNumericRange::Number(range) => {
            range_description("number", range).unwrap_or_else(|| "a number".to_owned())
        }
        _ => "a number".to_owned(),
    }
}

fn range_description<T>(kind: &str, range: FieldRange<T>) -> Option<String>
where
    T: std::fmt::Display + Copy,
{
    match (range.minimum, range.maximum) {
        (Some(min), Some(max))
            if min.kind == FieldBoundKind::Closed && max.kind == FieldBoundKind::Closed =>
        {
            Some(format!("{kind} in range {}..{}", min.value, max.value))
        }
        (Some(min), None) if min.kind == FieldBoundKind::Closed => {
            Some(format!("{kind} >= {}", min.value))
        }
        (None, Some(max)) if max.kind == FieldBoundKind::Closed => {
            Some(format!("{kind} <= {}", max.value))
        }
        _ => None,
    }
}
