use docnav_adapter_contracts::{Adapter, AdapterOptionSpec};
use docnav_markdown::MarkdownAdapter;
use docnav_navigation::{NavigationAdapterRef, NavigationAdapterRegistry};
use docnav_protocol::{Manifest, Operation};
use serde_json::{json, Value};

use crate::cli::AdapterCommand;
use crate::error::AppResult;
use crate::output::CommandOutcome;
use crate::project_context::ProjectContext;

static MARKDOWN_ADAPTER: MarkdownAdapter = MarkdownAdapter;
static ADAPTERS: &[AdapterRecord] = &[AdapterRecord {
    adapter: &MARKDOWN_ADAPTER,
}];

#[derive(Clone, Copy)]
pub struct AdapterRegistry {
    pub adapters: &'static [AdapterRecord],
}

#[derive(Clone, Copy)]
pub struct AdapterRecord {
    adapter: &'static dyn Adapter,
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

    pub fn native_options_for(&self, operation: Operation) -> Vec<AdapterOptionSpec> {
        self.adapters
            .iter()
            .flat_map(|record| record.native_options_for(operation))
            .collect()
    }

    pub fn has_native_option_config_key(&self, key: &str) -> bool {
        self.adapters
            .iter()
            .flat_map(|record| record.adapter_options())
            .any(|option| native_option_config_key(&option).as_deref() == Some(key))
    }

    pub fn native_option_config_keys(&self) -> Vec<String> {
        let mut keys = Vec::new();
        for option in self
            .adapters
            .iter()
            .flat_map(|record| record.adapter_options())
        {
            if let Some(key) = native_option_config_key(&option) {
                if !keys.contains(&key) {
                    keys.push(key);
                }
            }
        }
        keys
    }
}

fn native_option_config_key(option: &AdapterOptionSpec) -> Option<String> {
    let path = option.processing_path("config").ok().flatten()?;
    if path.len() == 2 && path.first().is_some_and(|segment| segment == "options") {
        Some(path.join("."))
    } else {
        None
    }
}

impl NavigationAdapterRegistry for AdapterRegistry {
    fn adapters(&self) -> Vec<NavigationAdapterRef<'_>> {
        self.adapters
            .iter()
            .map(|record| NavigationAdapterRef {
                id: record.id(),
                adapter: record.adapter(),
            })
            .collect()
    }
}

pub fn native_options_for(operation: Operation) -> Vec<AdapterOptionSpec> {
    AdapterRegistry::builtin().native_options_for(operation)
}

impl AdapterRecord {
    pub fn id(&self) -> &str {
        self.adapter.adapter_id()
    }

    pub fn adapter(&self) -> &'static dyn Adapter {
        self.adapter
    }

    pub fn manifest(&self) -> Manifest {
        self.adapter.manifest()
    }

    pub fn adapter_options(&self) -> Vec<AdapterOptionSpec> {
        self.adapter.adapter_options()
    }

    pub fn native_options_for(&self, operation: Operation) -> Vec<AdapterOptionSpec> {
        self.adapter_options()
            .into_iter()
            .filter(|option| option.applies_to(operation))
            .collect()
    }

    #[cfg(test)]
    pub(crate) const fn from_adapter(adapter: &'static dyn Adapter) -> Self {
        Self { adapter }
    }
}

impl std::fmt::Debug for AdapterRecord {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AdapterRecord")
            .field("id", &self.id())
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

pub fn registry_check(registry: &AdapterRegistry) -> Value {
    json!({
        "name": "core_static_adapter_registry",
        "status": "pass",
        "message": "core release static adapter registry is available",
        "adapter_count": registry.len(),
    })
}

pub fn adapter_layer_checks(registry: &AdapterRegistry) -> Vec<Value> {
    if registry.is_empty() {
        return vec![json!({
            "name": "adapter_layer",
            "status": "fail",
            "message": "core release static adapter registry has no adapters",
        })];
    }

    registry
        .adapters
        .iter()
        .map(|adapter| {
            let manifest = adapter.manifest();
            let status =
                if manifest.adapter.id == adapter.id() && manifest.validate_semantics().is_ok() {
                    "pass"
                } else {
                    "fail"
                };
            json!({
                "name": "adapter_layer",
                "status": status,
                "adapter_id": adapter.id(),
                "version": manifest.adapter.version,
                "formats": manifest.formats,
                "message": if status == "pass" {
                    "built-in adapter layer metadata is available"
                } else {
                    "built-in adapter layer metadata is invalid"
                },
            })
        })
        .collect()
}

fn adapter_metadata(adapter: &AdapterRecord) -> Value {
    let manifest = adapter.manifest();
    json!({
        "id": manifest.adapter.id,
        "name": manifest.adapter.name,
        "version": manifest.adapter.version,
        "formats": manifest.formats,
    })
}

#[cfg(test)]
mod tests;
