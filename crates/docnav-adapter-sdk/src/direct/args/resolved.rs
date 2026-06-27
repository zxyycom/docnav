use docnav_protocol::{positive_result, Operation, PositiveInteger};
use docnav_standard_parameters::{
    PassthroughValue, StandardParameterHandoff, StandardParameterResolution,
    StandardParameterSourceKind, StandardParameterValidationIssue,
};
use docnav_typed_fields::{FieldIdentity, TypedValue};
use serde_json::{json, Map, Value};

use super::super::native_options::{NativeOptionDefault, NativeOptionSpec};
use super::super::warnings::DirectCliWarning;
use super::spec::flags;
use super::standard::{ID_LIMIT_CHARS, ID_OUTPUT, ID_PAGE, ID_PATH, ID_QUERY, ID_REF};

pub(super) fn resolved_page(
    operation: Operation,
    resolution: &StandardParameterResolution,
) -> Result<PositiveInteger, String> {
    if operation == Operation::Info {
        return Ok(positive_result(1).expect("static positive integer"));
    }

    required_positive_value(resolution, ID_PAGE, flags::PAGE)
}

pub(super) fn resolved_limit_chars(
    operation: Operation,
    resolution: &StandardParameterResolution,
    default_limit_chars: u32,
) -> Result<Value, String> {
    if operation == Operation::Info {
        return Ok(Value::from(default_limit_chars));
    }
    Ok(json!(required_positive_value(
        resolution,
        ID_LIMIT_CHARS,
        flags::LIMIT_CHARS
    )?
    .get()))
}

pub(super) fn merged_native_options(
    operation: Operation,
    resolution: &StandardParameterResolution,
    specs: &[NativeOptionSpec],
) -> Map<String, Value> {
    let mut options = default_native_option_values(operation, specs);
    extend_native_options_from_source(
        &mut options,
        resolution,
        StandardParameterSourceKind::UserConfig,
    );
    extend_native_options_from_source(
        &mut options,
        resolution,
        StandardParameterSourceKind::ProjectConfig,
    );
    extend_native_options_from_source(
        &mut options,
        resolution,
        StandardParameterSourceKind::DirectInput,
    );
    options
}

pub(super) fn collect_diagnostics(
    resolution: &StandardParameterResolution,
    warnings: &mut Vec<DirectCliWarning>,
) -> Result<(), String> {
    for diagnostic in resolution.diagnostics() {
        match diagnostic {
            StandardParameterHandoff::Validation(diagnostic) => {
                return Err(validation_message(diagnostic));
            }
            StandardParameterHandoff::Warning(warning) => warnings.push(warning.clone()),
        }
    }
    Ok(())
}

pub(super) fn required_string_value(
    resolution: &StandardParameterResolution,
    identity: &str,
) -> Result<String, String> {
    optional_string_value(resolution, identity)?
        .ok_or_else(|| format!("missing resolved standard parameter {identity}"))
}

pub(super) fn optional_string_value(
    resolution: &StandardParameterResolution,
    identity: &str,
) -> Result<Option<String>, String> {
    let Some(value) = resolution.value(&identity_key(identity)?) else {
        return Ok(None);
    };
    match &value.value {
        TypedValue::String(value) => Ok(Some(value.clone())),
        TypedValue::Null => Ok(None),
        _ => Err(format!("unexpected standard parameter type for {identity}")),
    }
}

pub(super) fn required_json_value(
    resolution: &StandardParameterResolution,
    identity: &str,
) -> Result<Value, String> {
    let value = resolution
        .value(&identity_key(identity)?)
        .ok_or_else(|| format!("missing resolved standard parameter {identity}"))?;
    Ok(typed_value_to_json(&value.value))
}

fn default_native_option_values(
    operation: Operation,
    specs: &[NativeOptionSpec],
) -> Map<String, Value> {
    let mut options = Map::new();
    for spec in specs.iter().filter(|spec| spec.supports(operation)) {
        let Some(default) = spec.default else {
            continue;
        };
        let value = match default {
            NativeOptionDefault::Integer(value) => Value::from(value),
        };
        options.insert(spec.option_key.to_owned(), value);
    }
    options
}

fn extend_native_options_from_source(
    options: &mut Map<String, Value>,
    resolution: &StandardParameterResolution,
    source: StandardParameterSourceKind,
) {
    let Some(passthrough) = passthrough_from_source(resolution, source) else {
        return;
    };
    let Value::Object(native_options) = &passthrough.value else {
        return;
    };
    options.extend(native_options.clone());
}

fn passthrough_from_source(
    resolution: &StandardParameterResolution,
    source: StandardParameterSourceKind,
) -> Option<&PassthroughValue> {
    resolution
        .passthrough()
        .iter()
        .find(|value| value.source.kind == source)
}

fn validation_message(diagnostic: &StandardParameterValidationIssue) -> String {
    match diagnostic.identity.as_str() {
        ID_LIMIT_CHARS => format!("{} must be a positive integer", flags::LIMIT_CHARS),
        ID_OUTPUT => format!("invalid {}", flags::OUTPUT),
        ID_PAGE => format!("{} must be a positive integer", flags::PAGE),
        ID_PATH => "path value must not be empty".to_owned(),
        ID_QUERY => format!("{} value must not be empty", flags::QUERY),
        ID_REF => format!("{} value must not be empty", flags::REF),
        _ => "standard parameter validation failed".to_owned(),
    }
}

fn required_positive_value(
    resolution: &StandardParameterResolution,
    identity: &str,
    flag: &str,
) -> Result<PositiveInteger, String> {
    let value = resolution
        .value(&identity_key(identity)?)
        .ok_or_else(|| format!("missing resolved standard parameter {identity}"))?;
    let TypedValue::Integer(value) = value.value else {
        return Err(format!("{flag} must be a positive integer"));
    };
    let value = u32::try_from(value)
        .ok()
        .and_then(std::num::NonZeroU32::new)
        .ok_or_else(|| format!("{flag} must be a positive integer"))?;
    Ok(value)
}

fn typed_value_to_json(value: &TypedValue) -> Value {
    match value {
        TypedValue::String(value) => Value::from(value.clone()),
        TypedValue::Integer(value) => Value::from(*value),
        TypedValue::Number(value) => Value::from(*value),
        TypedValue::Boolean(value) => Value::from(*value),
        TypedValue::Array(value) => Value::Array(value.clone()),
        TypedValue::Object(value) => Value::Object(value.clone()),
        TypedValue::Null => Value::Null,
    }
}

fn identity_key(identity: &str) -> Result<FieldIdentity, String> {
    FieldIdentity::new(identity)
        .map_err(|error| format!("invalid standard parameter identity: {error}"))
}
