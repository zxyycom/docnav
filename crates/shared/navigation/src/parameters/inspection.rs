use cli_config_resolution::ResolutionResult;
use docnav_diagnostics::{DiagnosticRecordDraft, DiagnosticSource};
use docnav_typed_fields::{FieldDefSet, ProcessingId};
use serde_json::{json, Value};

use crate::{
    config_source::NavigationConfigSourceIssue, NavigationConfigSource, NavigationConfigSources,
    NavigationError,
};

use super::{
    catalog::DocumentParameterCatalog, config, fields, resolution, resolve_with_fields, values,
    CONFIG_PROCESSING,
};

pub(crate) fn inspect_config_sources(
    config_sources: &NavigationConfigSources,
    catalog: &DocumentParameterCatalog,
) -> Result<Value, NavigationError> {
    let routing_fields = fields::adapter_routing_fields()
        .map_err(|_| NavigationError::internal("config-routing-fields-build-failed"))?;
    let invocation_log_fields = fields::invocation_log_fields()
        .map_err(|_| NavigationError::internal("config-invocation-log-fields-build-failed"))?;
    let routing_resolution = resolve_with_fields(
        &routing_fields,
        None,
        None,
        config_sources,
        "config-inspection-routing",
    )?;
    let catalog_resolution = resolve_with_fields(
        catalog.fields(),
        None,
        None,
        config_sources,
        "config-inspection-catalog",
    )?;
    let invocation_log_resolution = resolve_with_fields(
        &invocation_log_fields,
        None,
        None,
        config_sources,
        "config-inspection-invocation-log",
    )?;
    let scalar_fields = [&routing_fields, catalog.fields(), &invocation_log_fields];
    let resolved_fields = [
        (&routing_fields, &routing_resolution),
        (catalog.fields(), &catalog_resolution),
        (&invocation_log_fields, &invocation_log_resolution),
    ];
    let resolutions = [
        &routing_resolution,
        &catalog_resolution,
        &invocation_log_resolution,
    ];

    Ok(json!({
        "sources": [
            inspect_source(&config_sources.project, catalog),
            inspect_source(&config_sources.user, catalog),
        ],
        "config_source_projection": config_source_projection(&scalar_fields),
        "parameter_facts": parameter_facts(&resolved_fields),
        "parameter_diagnostics": parameter_diagnostics(&resolutions, config_sources),
    }))
}

fn inspect_source(source: &NavigationConfigSource, catalog: &DocumentParameterCatalog) -> Value {
    let diagnostics = source_diagnostics(source, catalog);
    let load_state = load_state(source);
    json!({
        "scope": source.level.as_str(),
        "path": source.path,
        "origin": source.origin.as_str(),
        "exists": std::path::Path::new(&source.path).exists(),
        "load_state": load_state,
        "summary": source.loaded.value().map(source_summary).unwrap_or_else(|| {
            json!({
                "top_level_fields": [],
                "field_count": 0
            })
        }),
        "diagnostics": diagnostics,
    })
}

fn source_diagnostics(
    source: &NavigationConfigSource,
    catalog: &DocumentParameterCatalog,
) -> Vec<Value> {
    let mut diagnostics = source
        .loaded
        .diagnostics()
        .iter()
        .map(config_source_issue_json)
        .collect::<Vec<_>>();
    if diagnostics.is_empty() {
        if let Err(error) = config::validate_config_source_for_catalog(source, catalog) {
            diagnostics.push(diagnostic_json(error.diagnostic()));
        }
    }
    diagnostics
}

fn load_state(source: &NavigationConfigSource) -> String {
    if source.loaded.value().is_some() {
        return "loaded".to_owned();
    }
    source
        .loaded
        .diagnostics()
        .first()
        .map_or_else(|| "missing".to_owned(), |issue| issue.reason_code.clone())
}

fn source_summary(value: &Value) -> Value {
    let fields = value
        .as_object()
        .map(|object| object.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    json!({
        "top_level_fields": fields,
        "field_count": flatten_fields(value).len(),
    })
}

fn flatten_fields(value: &Value) -> Vec<String> {
    let mut fields = Vec::new();
    flatten_value(value, String::new(), &mut fields);
    fields
}

fn flatten_value(value: &Value, path: String, fields: &mut Vec<String>) {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                let child_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                flatten_value(child, child_path, fields);
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                flatten_value(child, format!("{path}[{index}]"), fields);
            }
        }
        _ if !path.is_empty() => fields.push(path),
        _ => {}
    }
}

fn config_source_projection(field_sets: &[&FieldDefSet]) -> Vec<Value> {
    let processing_id =
        ProcessingId::new(CONFIG_PROCESSING).expect("config processing id is valid");
    field_sets
        .iter()
        .flat_map(|fields| fields.processing_metadata(&processing_id))
        .map(|metadata| {
            json!({
                "identity": metadata.identity().as_str(),
                "path": metadata.path.segments().join("."),
                "value_kind": format!("{:?}", metadata.value_kind()),
                "has_default": !matches!(metadata.default(), docnav_typed_fields::DefaultMetadata::None),
            })
        })
        .collect()
}

fn parameter_facts(field_sets: &[(&FieldDefSet, &ResolutionResult)]) -> Vec<Value> {
    let processing_id =
        ProcessingId::new(CONFIG_PROCESSING).expect("config processing id is valid");
    field_sets
        .iter()
        .flat_map(|(fields, resolution)| {
            resolution
                .fields()
                .iter()
                .filter_map(|(identity, resolved)| {
                    let field = fields.field(identity)?;
                    field.processing_metadata(&processing_id)?;
                    let value = values::projected_field_value(fields, identity, resolved)?;
                    Some(json!({
                        "identity": identity.as_str(),
                        "source": values::field_source_label(resolved).unwrap_or("built_in"),
                        "value": values::typed_value_to_json(value),
                    }))
                })
        })
        .collect()
}

fn parameter_diagnostics(
    resolutions: &[&ResolutionResult],
    config_sources: &NavigationConfigSources,
) -> Vec<Value> {
    let source = DiagnosticSource::with_stage("docnav-navigation", "config-inspect");
    let mut diagnostics = resolutions
        .iter()
        .flat_map(|resolution| resolution.diagnostics())
        .map(|diagnostic| {
            diagnostic_json(&resolution::diagnostic_record(diagnostic, source.clone()))
        })
        .collect::<Vec<_>>();
    diagnostics.extend(
        [&config_sources.project, &config_sources.user]
            .into_iter()
            .flat_map(|source| source.loaded.diagnostics())
            .map(|issue| diagnostic_json(&issue.to_record_draft(source.clone()))),
    );
    diagnostics
}

fn config_source_issue_json(diagnostic: &NavigationConfigSourceIssue) -> Value {
    json!({
        "source_level": diagnostic.source_level,
        "path_origin": diagnostic.path_origin,
        "path": diagnostic.path,
        "field": diagnostic.field,
        "reason_code": diagnostic.reason_code,
    })
}

fn diagnostic_json(diagnostic: &DiagnosticRecordDraft) -> Value {
    diagnostic.details().to_value()
}
