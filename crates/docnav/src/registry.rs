use docnav_adapter_contracts::AdapterDefinition;
use docnav_markdown::markdown_adapter_definition;
use docnav_navigation::{NavigationAdapterRef, NavigationAdapterRegistry};
use docnav_protocol::Manifest;
use serde_json::{json, Value};

use crate::cli::AdapterCommand;
use crate::config::DoctorCheck;
use crate::error::{AppResult, DocnavExitCode};
use crate::output::CommandOutcome;
use crate::project_context::ProjectContext;

static ADAPTERS: &[AdapterRecord] = &[AdapterRecord {
    definition: markdown_adapter_definition,
    implementation_source: "core_static",
}];

#[derive(Clone, Copy)]
pub struct AdapterRegistry {
    pub adapters: &'static [AdapterRecord],
}

#[derive(Clone, Copy)]
pub struct AdapterRecord {
    definition: fn() -> AdapterDefinition<'static>,
    implementation_source: &'static str,
}

impl AdapterRegistry {
    pub fn builtin() -> Self {
        Self { adapters: ADAPTERS }
    }

    pub fn load(_project: &ProjectContext) -> AppResult<Self> {
        Ok(Self::builtin())
    }

    pub fn len(&self) -> usize {
        self.adapters.len()
    }

    pub fn is_empty(&self) -> bool {
        self.adapters.is_empty()
    }
}

impl NavigationAdapterRegistry for AdapterRegistry {
    fn adapters(&self) -> Vec<NavigationAdapterRef<'_>> {
        self.adapters
            .iter()
            .map(|record| NavigationAdapterRef::new(record.definition()))
            .collect()
    }
}

impl AdapterRecord {
    pub fn id(&self) -> String {
        self.definition().id().to_owned()
    }

    pub fn definition(&self) -> AdapterDefinition<'static> {
        (self.definition)()
    }

    pub fn manifest(&self) -> Manifest {
        self.definition().manifest().clone()
    }

    pub fn implementation_source(&self) -> &'static str {
        self.implementation_source
    }
}

impl std::fmt::Debug for AdapterRecord {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AdapterRecord")
            .field("id", &self.id())
            .field("implementation_source", &self.implementation_source)
            .finish_non_exhaustive()
    }
}

pub fn execute(command: AdapterCommand) -> AppResult<CommandOutcome> {
    match command {
        AdapterCommand::List => adapter_list(),
    }
}

pub fn adapter_list() -> AppResult<CommandOutcome> {
    let registry = AdapterRegistry { adapters: ADAPTERS };
    let adapters = registry
        .adapters
        .iter()
        .map(adapter_metadata)
        .collect::<Vec<_>>();
    Ok(CommandOutcome::json(json!({
        "registry": "core_static",
        "adapters": adapters,
    })))
}

pub fn registry_check(registry: &AdapterRegistry) -> DoctorCheck {
    DoctorCheck::pass(json!({
        "name": "core_static_adapter_registry",
        "status": "pass",
        "message": "core release static adapter registry is available",
        "adapter_count": registry.len(),
    }))
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
        .map(|adapter| {
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
                "implementation_source": adapter.implementation_source(),
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

fn adapter_metadata(adapter: &AdapterRecord) -> Value {
    let manifest = adapter.manifest();
    json!({
        "id": manifest.adapter.id,
        "name": manifest.adapter.name,
        "version": manifest.adapter.version,
        "implementation_source": adapter.implementation_source(),
        "formats": manifest.formats,
    })
}

#[cfg(test)]
mod tests;
