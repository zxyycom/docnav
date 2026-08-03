use docnav_adapter_contracts::AdapterDefinition;
use docnav_json::json_adapter_definition;
use docnav_markdown::markdown_adapter_definition;
use docnav_navigation::{NavigationAdapterRegistry, RegistryRouting, RegistryRoutingError};
use serde_json::{json, Value};

use crate::cli::AdapterCommand;
use crate::config::DoctorCheck;
use crate::error::{AppResult, DocnavExitCode};
use crate::output::CommandOutcome;

static ADAPTERS: &[fn() -> AdapterDefinition<'static>] =
    &[markdown_adapter_definition, json_adapter_definition];

#[derive(Clone)]
pub struct AdapterRegistry {
    pub adapters: &'static [fn() -> AdapterDefinition<'static>],
    routing: Result<RegistryRouting, RegistryRoutingError>,
}

impl AdapterRegistry {
    pub fn builtin() -> Self {
        Self::new(ADAPTERS)
    }

    pub fn new(adapters: &'static [fn() -> AdapterDefinition<'static>]) -> Self {
        let definitions = adapters
            .iter()
            .map(|definition| definition())
            .collect::<Vec<_>>();
        Self {
            adapters,
            routing: RegistryRouting::from_adapters(&definitions),
        }
    }

    pub fn len(&self) -> usize {
        self.adapters.len()
    }

    pub fn is_empty(&self) -> bool {
        self.adapters.is_empty()
    }
}

impl NavigationAdapterRegistry for AdapterRegistry {
    fn adapters(&self) -> Vec<AdapterDefinition<'_>> {
        self.adapters
            .iter()
            .map(|definition| definition())
            .collect()
    }

    fn routing(&self) -> Result<RegistryRouting, RegistryRoutingError> {
        self.routing.clone()
    }
}

pub fn execute(command: AdapterCommand) -> AppResult<CommandOutcome> {
    match command {
        AdapterCommand::List => adapter_list(),
    }
}

pub fn adapter_list() -> AppResult<CommandOutcome> {
    let registry = AdapterRegistry::builtin();
    let adapters = registry
        .adapters
        .iter()
        .map(|definition| adapter_metadata(definition()))
        .collect::<Vec<_>>();
    Ok(CommandOutcome::json(json!({
        "registry": "core_static",
        "adapters": adapters,
    })))
}

pub fn registry_check(registry: &AdapterRegistry) -> DoctorCheck {
    match registry.routing() {
        Ok(_) => DoctorCheck::pass(json!({
            "name": "core_static_adapter_registry",
            "status": "pass",
            "message": "core release static adapter registry is available",
            "adapter_count": registry.len(),
        })),
        Err(error) => DoctorCheck::failure(
            json!({
                "name": "core_static_adapter_registry",
                "status": "fail",
                "message": "core release static adapter registry contains conflicting routing metadata",
                "adapter_count": registry.len(),
                "error_id": error.error_id(),
            }),
            DocnavExitCode::AdapterOrProtocolError,
        ),
    }
}

pub fn adapter_layer_checks(registry: &AdapterRegistry) -> Vec<DoctorCheck> {
    if registry.is_empty() {
        return vec![DoctorCheck::failure(
            json!({
                "name": "adapter_layer",
                "status": "fail",
                "message": "core release static adapter registry has no adapters",
            }),
            DocnavExitCode::AdapterOrProtocolError,
        )];
    }

    registry
        .adapters
        .iter()
        .map(|definition| {
            let adapter = definition();
            let manifest = adapter.manifest();
            let id = adapter.id();
            let status = if manifest.adapter.id == id && manifest.validate_semantics().is_ok() {
                "pass"
            } else {
                "fail"
            };
            let value = json!({
                "name": "adapter_layer",
                "status": status,
                "adapter_id": id,
                "implementation_source": "core_static",
                "version": manifest.adapter.version,
                "formats": manifest.formats,
                "message": if status == "pass" {
                    "built-in adapter layer metadata is available"
                } else {
                    "built-in adapter layer metadata is invalid"
                },
            });
            if status == "pass" {
                DoctorCheck::pass(value)
            } else {
                DoctorCheck::failure(value, DocnavExitCode::AdapterOrProtocolError)
            }
        })
        .collect()
}

fn adapter_metadata(adapter: AdapterDefinition<'_>) -> Value {
    let manifest = adapter.manifest();
    json!({
        "id": manifest.adapter.id,
        "name": manifest.adapter.name,
        "version": manifest.adapter.version,
        "implementation_source": "core_static",
        "formats": manifest.formats,
    })
}

#[cfg(test)]
mod tests;
