use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;

use docnav_adapter_contracts::{
    AdapterDefinition, AdapterError, AdapterResult, UnstructuredFullRead,
};
use docnav_protocol::{Cost, OperationResult, OutlineResult, ProtocolResponse, RequestEnvelope};

use crate::{NavigationConfigSources, NavigationError};

use super::config::{ordered_config_sources, outline_config, thresholds};
use super::{OutlineMode, UnstructuredFullSelection};

const DEFAULT_CONTENT_TYPE: &str = "text/plain; charset=utf-8";
type EffectiveThresholds = BTreeMap<String, u64>;

pub(super) fn resolve_cost_thresholds(
    config_sources: &NavigationConfigSources,
    selected_adapter_id: &str,
    selected_adapter: &AdapterDefinition<'_>,
    request: &RequestEnvelope,
) -> Result<OutlineMode, NavigationError> {
    let effective = effective_thresholds(config_sources, selected_adapter_id)?;
    if effective.is_empty() {
        return Ok(OutlineMode::Structured);
    }

    let requested_units = requested_threshold_units(&effective, selected_adapter);
    if requested_units.is_empty() {
        return Ok(OutlineMode::Structured);
    }

    let cost = match selected_adapter.measure_unstructured_full_read_cost(request, &requested_units)
    {
        Ok(cost) => cost,
        Err(_) => return Ok(OutlineMode::Structured),
    };

    if cost_matches_threshold(&cost, &effective) {
        Ok(OutlineMode::UnstructuredFull(UnstructuredFullSelection {
            reason: docnav_protocol::UnstructuredOutlineReason::CostThreshold,
            cost,
        }))
    } else {
        Ok(OutlineMode::Structured)
    }
}

fn effective_thresholds(
    config_sources: &NavigationConfigSources,
    selected_adapter_id: &str,
) -> Result<EffectiveThresholds, NavigationError> {
    let mut effective = EffectiveThresholds::new();
    for source in ordered_config_sources(config_sources) {
        let Some(outline) = outline_config(source)? else {
            continue;
        };
        for threshold in thresholds(source, outline)? {
            if threshold.adapter != selected_adapter_id {
                continue;
            }
            effective
                .entry(threshold.unit)
                .and_modify(|value| *value = (*value).min(threshold.value))
                .or_insert(threshold.value);
        }
    }
    Ok(effective)
}

fn requested_threshold_units(
    effective: &EffectiveThresholds,
    adapter: &AdapterDefinition<'_>,
) -> Vec<String> {
    let Some(capabilities) = adapter.unstructured_full_read_capabilities() else {
        return Vec::new();
    };
    effective
        .keys()
        .filter(|unit| capabilities.has_cost_measurement_unit(unit))
        .cloned()
        .collect()
}

fn cost_matches_threshold(cost: &Cost, effective: &EffectiveThresholds) -> bool {
    cost.measurements.iter().any(|measurement| {
        effective
            .get(&measurement.unit)
            .is_some_and(|threshold| measurement.value <= *threshold)
    })
}

pub(super) fn execute_unstructured_outline(
    adapter: &AdapterDefinition<'_>,
    request: &RequestEnvelope,
    selection: UnstructuredFullSelection,
) -> ProtocolResponse {
    match unstructured_full_read(adapter, request, selection.cost) {
        Ok(result) => ProtocolResponse::success(
            request.protocol_version.clone(),
            request.request_id.clone(),
            OperationResult::Outline(OutlineResult::unstructured(
                selection.reason,
                result.content,
                result.content_type,
                result.facts.cost.unwrap_or_else(empty_cost),
            )),
        ),
        Err(error) => ProtocolResponse::failure_for_request(request, error.protocol_error()),
    }
}

fn unstructured_full_read(
    adapter: &AdapterDefinition<'_>,
    request: &RequestEnvelope,
    selector_cost: Cost,
) -> AdapterResult<UnstructuredFullRead> {
    let capabilities = adapter.unstructured_full_read_capabilities();
    let mut result = if capabilities.is_some_and(|capabilities| capabilities.content_hook) {
        adapter.unstructured_full_read(request)?
    } else {
        default_utf8_full_read(request)?
    };

    if capabilities.is_some_and(|capabilities| capabilities.result_facts_hook) {
        let facts = adapter.unstructured_full_read_facts(request)?;
        if result.facts.cost.is_none() {
            result.facts.cost = facts.cost;
        }
    }
    if result.facts.cost.is_none() && !selector_cost.measurements.is_empty() {
        result.facts.cost = Some(selector_cost);
    }
    if result.facts.cost.is_none() {
        result.facts.cost = Some(empty_cost());
    }

    Ok(result)
}

fn default_utf8_full_read(request: &RequestEnvelope) -> AdapterResult<UnstructuredFullRead> {
    let path = &request.document.path;
    let bytes = fs::read(path).map_err(|error| match error.kind() {
        ErrorKind::NotFound => AdapterError::document_not_found(path),
        _ => AdapterError::document_path_invalid(path, error.to_string()),
    })?;
    let content = String::from_utf8(bytes)
        .map_err(|_| AdapterError::document_encoding_unsupported(path, "utf-8"))?;
    Ok(UnstructuredFullRead::new(content, DEFAULT_CONTENT_TYPE))
}

pub(super) fn empty_cost() -> Cost {
    Cost {
        measurements: Vec::new(),
    }
}
