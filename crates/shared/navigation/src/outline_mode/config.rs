use regex::Regex;
use serde_json::{Map, Value};

use crate::error::ConfigFieldError;
use crate::{NavigationCommand, NavigationConfigSource, NavigationConfigSources, NavigationError};

const OUTLINE_MODE_STRUCTURED: &str = "structured";
const OUTLINE_MODE_UNSTRUCTURED_FULL: &str = "unstructured_full";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RuleMode {
    Structured,
    UnstructuredFull,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ModeRule {
    pub pattern: String,
    pub mode: RuleMode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CostThreshold {
    pub adapter: String,
    pub unit: String,
    pub value: u64,
}

pub(super) fn validate_outline_config_sources(
    command: &NavigationCommand,
    config_sources: &NavigationConfigSources,
) -> Result<(), NavigationError> {
    if command.operation != docnav_protocol::Operation::Outline {
        return Ok(());
    }

    for source in ordered_config_sources(config_sources) {
        validate_outline_config_source(source)?;
    }
    Ok(())
}

pub(super) fn validate_outline_config_source(
    source: &NavigationConfigSource,
) -> Result<(), NavigationError> {
    let Some(outline) = outline_config(source)? else {
        return Ok(());
    };
    let _ = mode_rules(source, outline)?;
    let _ = thresholds(source, outline)?;
    Ok(())
}

pub(super) fn mode_rules(
    source: &NavigationConfigSource,
    outline: &Map<String, Value>,
) -> Result<Vec<ModeRule>, NavigationError> {
    let Some(value) = outline.get("mode_rules") else {
        return Ok(Vec::new());
    };
    let Some(items) = value.as_array() else {
        return Err(invalid_config_field(
            source,
            "outline.mode_rules",
            "invalid_config_array",
            "Use an array for outline.mode_rules.",
        ));
    };

    items
        .iter()
        .enumerate()
        .map(|(index, item)| mode_rule(source, item, index))
        .collect()
}

pub(super) fn thresholds(
    source: &NavigationConfigSource,
    outline: &Map<String, Value>,
) -> Result<Vec<CostThreshold>, NavigationError> {
    let Some(auto_full_read) = outline.get("auto_full_read") else {
        return Ok(Vec::new());
    };
    let Some(auto_full_read) = auto_full_read.as_object() else {
        return Err(NavigationError::config_invalid_object(
            source.level.as_str(),
            source.origin.as_str(),
            &source.path,
            "outline.auto_full_read",
        ));
    };
    let Some(value) = auto_full_read.get("thresholds") else {
        return Ok(Vec::new());
    };
    let Some(items) = value.as_array() else {
        return Err(invalid_config_field(
            source,
            "outline.auto_full_read.thresholds",
            "invalid_config_array",
            "Use an array for outline.auto_full_read.thresholds.",
        ));
    };

    items
        .iter()
        .enumerate()
        .map(|(index, item)| threshold(source, item, index))
        .collect()
}

pub(super) fn compile_path_pattern(
    source: &NavigationConfigSource,
    field: &str,
    pattern: &str,
) -> Result<Regex, NavigationError> {
    Regex::new(&format!("^(?:{pattern})$")).map_err(|_| {
        invalid_config_field(
            source,
            field,
            "invalid_path_pattern",
            "Use a valid regex path pattern supported by the regex matcher.",
        )
    })
}

pub(super) fn outline_config(
    source: &NavigationConfigSource,
) -> Result<Option<&Map<String, Value>>, NavigationError> {
    let Some(value) = source.loaded.value() else {
        return Ok(None);
    };
    let Some(root) = value.as_object() else {
        return Ok(None);
    };
    let Some(outline) = root.get("outline") else {
        return Ok(None);
    };
    outline.as_object().map(Some).ok_or_else(|| {
        NavigationError::config_invalid_object(
            source.level.as_str(),
            source.origin.as_str(),
            &source.path,
            "outline",
        )
    })
}

pub(super) fn ordered_config_sources(
    config_sources: &NavigationConfigSources,
) -> [&NavigationConfigSource; 2] {
    [&config_sources.user, &config_sources.project]
}

fn mode_rule(
    source: &NavigationConfigSource,
    value: &Value,
    index: usize,
) -> Result<ModeRule, NavigationError> {
    let field = |name: &str| format!("outline.mode_rules[{index}].{name}");
    let Some(object) = value.as_object() else {
        return Err(NavigationError::config_invalid_object(
            source.level.as_str(),
            source.origin.as_str(),
            &source.path,
            &format!("outline.mode_rules[{index}]"),
        ));
    };
    let path = required_string(source, object, &field("path"))?;
    compile_path_pattern(source, &field("path"), &path)?;
    let mode = match required_string(source, object, &field("mode"))?.as_str() {
        OUTLINE_MODE_STRUCTURED => RuleMode::Structured,
        OUTLINE_MODE_UNSTRUCTURED_FULL => RuleMode::UnstructuredFull,
        _ => {
            return Err(NavigationError::config_invalid_field(
                ConfigFieldError::invalid(
                    source,
                    field("mode"),
                    "invalid_outline_mode",
                    "Use structured or unstructured_full for outline.mode_rules[].mode.",
                )
                .with_accepted(vec![
                    OUTLINE_MODE_STRUCTURED.to_owned(),
                    OUTLINE_MODE_UNSTRUCTURED_FULL.to_owned(),
                ]),
            ));
        }
    };

    Ok(ModeRule {
        pattern: path,
        mode,
    })
}

fn threshold(
    source: &NavigationConfigSource,
    value: &Value,
    index: usize,
) -> Result<CostThreshold, NavigationError> {
    let field = |name: &str| format!("outline.auto_full_read.thresholds[{index}].{name}");
    let Some(object) = value.as_object() else {
        return Err(NavigationError::config_invalid_object(
            source.level.as_str(),
            source.origin.as_str(),
            &source.path,
            &format!("outline.auto_full_read.thresholds[{index}]"),
        ));
    };
    let adapter = required_string(source, object, &field("adapter"))?;
    let unit = required_string(source, object, &field("unit"))?;
    let value = required_positive_u64(source, object, &field("value"))?;
    Ok(CostThreshold {
        adapter,
        unit,
        value,
    })
}

fn required_string(
    source: &NavigationConfigSource,
    object: &Map<String, Value>,
    field: &str,
) -> Result<String, NavigationError> {
    let key = field.rsplit('.').next().unwrap_or(field);
    let Some(value) = object.get(key) else {
        return Err(NavigationError::config_missing_field(
            source.level.as_str(),
            source.origin.as_str(),
            &source.path,
            field,
        ));
    };
    value
        .as_str()
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            invalid_config_field(
                source,
                field,
                "invalid_config_string",
                "Use a non-empty string value.",
            )
        })
}

fn required_positive_u64(
    source: &NavigationConfigSource,
    object: &Map<String, Value>,
    field: &str,
) -> Result<u64, NavigationError> {
    let key = field.rsplit('.').next().unwrap_or(field);
    let Some(value) = object.get(key) else {
        return Err(NavigationError::config_missing_field(
            source.level.as_str(),
            source.origin.as_str(),
            &source.path,
            field,
        ));
    };
    value.as_u64().filter(|value| *value > 0).ok_or_else(|| {
        invalid_config_field(
            source,
            field,
            "invalid_positive_integer",
            "Use a positive integer value.",
        )
    })
}

fn invalid_config_field(
    source: &NavigationConfigSource,
    field: &str,
    reason_code: &'static str,
    guidance: &str,
) -> NavigationError {
    NavigationError::config_invalid_field(ConfigFieldError::invalid(
        source,
        field,
        reason_code,
        guidance,
    ))
}
