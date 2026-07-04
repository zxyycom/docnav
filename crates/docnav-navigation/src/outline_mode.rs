mod config;
mod path_rules;
mod unstructured;

use docnav_adapter_contracts::Adapter;
use docnav_protocol::{Cost, RequestEnvelope, UnstructuredOutlineReason};

use crate::{NavigationCommand, NavigationConfigSources, NavigationError};

use config::RuleMode;
use path_rules::{normalized_document_path, resolve_path_rules};
use unstructured::{empty_cost, resolve_cost_thresholds};

#[derive(Clone, Debug, PartialEq)]
pub(super) enum OutlineMode {
    Structured,
    UnstructuredFull(UnstructuredFullSelection),
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct UnstructuredFullSelection {
    pub reason: UnstructuredOutlineReason,
    pub cost: Cost,
}

pub(super) fn resolve_outline_mode(
    config_sources: &NavigationConfigSources,
    selected_adapter_id: &str,
    selected_adapter: &dyn Adapter,
    request: &RequestEnvelope,
) -> Result<OutlineMode, NavigationError> {
    let normalized_path = normalized_document_path(&request.document.path, config_sources);
    if let Some(mode) = resolve_path_rules(config_sources, &normalized_path)? {
        return Ok(match mode {
            RuleMode::Structured => OutlineMode::Structured,
            RuleMode::UnstructuredFull => {
                OutlineMode::UnstructuredFull(UnstructuredFullSelection {
                    reason: UnstructuredOutlineReason::PathRule,
                    cost: empty_cost(),
                })
            }
        });
    }

    resolve_cost_thresholds(
        config_sources,
        selected_adapter_id,
        selected_adapter,
        request,
    )
}

pub(super) fn validate_outline_config_sources(
    command: &NavigationCommand,
    config_sources: &NavigationConfigSources,
) -> Result<(), NavigationError> {
    config::validate_outline_config_sources(command, config_sources)
}

pub(super) fn execute_unstructured_outline(
    adapter: &dyn Adapter,
    request: &RequestEnvelope,
    selection: UnstructuredFullSelection,
) -> docnav_protocol::ProtocolResponse {
    unstructured::execute_unstructured_outline(adapter, request, selection)
}
