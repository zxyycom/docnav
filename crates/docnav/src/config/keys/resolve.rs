use serde_json::json;

use crate::cli::OutputMode;
use crate::error::AppResult;
use crate::registry::AdapterRegistry;

use super::super::model::{
    ConfigContext, ResolvedValue, DEFAULT_LIMIT, DEFAULT_OUTPUT, DEFAULT_PAGINATION_ENABLED,
};
use super::registered_native_option_key;
use super::validation::{parse_output, validate_positive_key};

pub fn resolve_adapter(explicit: Option<&str>, context: &ConfigContext) -> ResolvedValue {
    if let Some(adapter) = explicit {
        return ResolvedValue::explicit(json!(adapter));
    }
    if let Some(adapter) = &context.project_config.defaults.adapter {
        return ResolvedValue::project(json!(adapter));
    }
    if let Some(adapter) = &context.user_config.defaults.adapter {
        return ResolvedValue::user(json!(adapter));
    }
    ResolvedValue::unset()
}

pub fn resolve_pagination_enabled(context: &ConfigContext) -> AppResult<ResolvedValue> {
    if let Some(enabled) = context.project_config.defaults.pagination.enabled {
        return Ok(ResolvedValue::project(json!(enabled)));
    }
    if let Some(enabled) = context.user_config.defaults.pagination.enabled {
        return Ok(ResolvedValue::user(json!(enabled)));
    }
    Ok(ResolvedValue::built_in(json!(DEFAULT_PAGINATION_ENABLED)))
}

pub fn resolve_limit(
    explicit: Option<docnav_protocol::PositiveInteger>,
    context: &ConfigContext,
) -> AppResult<ResolvedValue> {
    if let Some(limit) = explicit {
        return Ok(ResolvedValue::explicit(json!(limit.get())));
    }
    if let Some(limit) = context.project_config.defaults.pagination.limit {
        validate_positive_key("defaults.pagination.limit", limit)?;
        return Ok(ResolvedValue::project(json!(limit)));
    }
    if let Some(limit) = context.user_config.defaults.pagination.limit {
        validate_positive_key("defaults.pagination.limit", limit)?;
        return Ok(ResolvedValue::user(json!(limit)));
    }
    Ok(ResolvedValue::built_in(json!(DEFAULT_LIMIT)))
}

pub fn resolve_output(
    explicit: Option<OutputMode>,
    context: &ConfigContext,
) -> AppResult<ResolvedValue> {
    if let Some(output) = explicit {
        return Ok(ResolvedValue::explicit(json!(output.as_str())));
    }
    if let Some(output) = &context.project_config.defaults.output {
        let output = parse_output(
            "defaults.output",
            output,
            context.project.project_config_path(),
        )?;
        return Ok(ResolvedValue::project(json!(output.as_str())));
    }
    if let Some(output) = &context.user_config.defaults.output {
        let output = parse_output(
            "defaults.output",
            output,
            context.project.user_config_path(),
        )?;
        return Ok(ResolvedValue::user(json!(output.as_str())));
    }
    Ok(ResolvedValue::built_in(json!(DEFAULT_OUTPUT)))
}

pub(in crate::config) fn effective_key_value(
    key: &str,
    context: &ConfigContext,
    registry: &AdapterRegistry,
) -> AppResult<ResolvedValue> {
    match key {
        "defaults.adapter" => Ok(resolve_adapter(None, context)),
        "defaults.pagination.enabled" => resolve_pagination_enabled(context),
        "defaults.pagination.limit" => resolve_limit(None, context),
        "defaults.output" => resolve_output(None, context),
        "invocation_log.enabled" => Ok(resolve_invocation_log_enabled(context)),
        "invocation_log.path" => Ok(resolve_invocation_log_path(context)),
        "invocation_log.content_capture.enabled" => {
            Ok(resolve_invocation_log_content_capture_enabled(context))
        }
        "invocation_log.content_capture.root" => {
            Ok(resolve_invocation_log_content_capture_root(context))
        }
        _ => {
            let option_key = registered_native_option_key(key, registry)?;
            Ok(resolve_native_option(option_key, context))
        }
    }
}

fn resolve_native_option(key: &str, context: &ConfigContext) -> ResolvedValue {
    if let Some(value) = context.project_config.options.value_for_key(key) {
        return ResolvedValue::project(value.clone());
    }
    if let Some(value) = context.user_config.options.value_for_key(key) {
        return ResolvedValue::user(value.clone());
    }
    ResolvedValue::unset()
}

fn resolve_invocation_log_enabled(context: &ConfigContext) -> ResolvedValue {
    if let Some(value) = context.project_config.invocation_log.enabled {
        return ResolvedValue::project(json!(value));
    }
    if let Some(value) = context.user_config.invocation_log.enabled {
        return ResolvedValue::user(json!(value));
    }
    ResolvedValue::unset()
}

fn resolve_invocation_log_path(context: &ConfigContext) -> ResolvedValue {
    if let Some(value) = &context.project_config.invocation_log.path {
        return ResolvedValue::project(json!(value));
    }
    if let Some(value) = &context.user_config.invocation_log.path {
        return ResolvedValue::user(json!(value));
    }
    ResolvedValue::unset()
}

fn resolve_invocation_log_content_capture_enabled(context: &ConfigContext) -> ResolvedValue {
    if let Some(value) = context
        .project_config
        .invocation_log
        .content_capture
        .enabled
    {
        return ResolvedValue::project(json!(value));
    }
    if let Some(value) = context.user_config.invocation_log.content_capture.enabled {
        return ResolvedValue::user(json!(value));
    }
    ResolvedValue::unset()
}

fn resolve_invocation_log_content_capture_root(context: &ConfigContext) -> ResolvedValue {
    if let Some(value) = &context.project_config.invocation_log.content_capture.root {
        return ResolvedValue::project(json!(value));
    }
    if let Some(value) = &context.user_config.invocation_log.content_capture.root {
        return ResolvedValue::user(json!(value));
    }
    ResolvedValue::unset()
}
