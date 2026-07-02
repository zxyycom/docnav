use docnav_adapter_contracts::{Adapter, NativeOptionSpec};
use docnav_markdown::MarkdownAdapter;
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

    pub fn find(&self, adapter_id: &str) -> Option<&AdapterRecord> {
        self.adapters
            .iter()
            .find(|adapter| adapter.id() == adapter_id)
    }

    pub fn len(&self) -> usize {
        self.adapters.len()
    }

    pub fn is_empty(&self) -> bool {
        self.adapters.is_empty()
    }

    pub fn native_options_for(&self, operation: Operation) -> Vec<NativeOptionSpec> {
        self.adapters
            .iter()
            .flat_map(|record| record.native_options_for(operation))
            .collect()
    }

    pub fn has_native_option_config_key(&self, key: &str) -> bool {
        self.adapters
            .iter()
            .flat_map(|record| record.native_options().iter().copied())
            .any(|option| option.config_key() == key)
    }

    pub fn native_option_config_keys(&self) -> Vec<String> {
        let mut keys = Vec::new();
        for option in self
            .adapters
            .iter()
            .flat_map(|record| record.native_options().iter().copied())
        {
            let key = option.config_key();
            if !keys.contains(&key) {
                keys.push(key);
            }
        }
        keys
    }
}

pub fn native_options_for(operation: Operation) -> Vec<NativeOptionSpec> {
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

    pub fn probe(&self, path: &str) -> docnav_protocol::ProbeResult {
        self.adapter.probe(path)
    }

    pub fn native_options(&self) -> &'static [NativeOptionSpec] {
        self.adapter.native_options()
    }

    pub fn native_options_for(&self, operation: Operation) -> Vec<NativeOptionSpec> {
        self.native_options()
            .iter()
            .copied()
            .filter(|option| option.applies_to(operation))
            .collect()
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
            let status = if manifest.adapter.id == adapter.id()
                && manifest.validate_semantics().is_ok()
                && !manifest.capabilities.is_empty()
            {
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
                "capabilities": capabilities(&manifest.capabilities),
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
        "capabilities": capabilities(&manifest.capabilities),
    })
}

fn capabilities(capabilities: &[Operation]) -> Vec<&'static str> {
    capabilities
        .iter()
        .map(|operation| operation.as_str())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // @case WB-CORE-ADAPTER-001
    #[test]
    fn static_registry_contains_built_in_markdown_adapter() {
        let registry = AdapterRegistry { adapters: ADAPTERS };
        let record = registry
            .find("docnav-markdown")
            .expect("built-in markdown adapter");

        let manifest = record.manifest();

        assert_eq!(record.id(), "docnav-markdown");
        assert_eq!(manifest.adapter.id, "docnav-markdown");
        assert!(manifest.capabilities.contains(&Operation::Outline));
        assert!(manifest.capabilities.contains(&Operation::Read));
        assert!(manifest.capabilities.contains(&Operation::Find));
        assert!(manifest.capabilities.contains(&Operation::Info));
    }

    #[test]
    fn static_registry_exposes_full_native_option_specs() {
        let registry = AdapterRegistry { adapters: ADAPTERS };
        let native_options = registry.native_options_for(Operation::Outline);

        assert!(registry.has_native_option_config_key("options.max_heading_level"));
        assert!(registry
            .native_option_config_keys()
            .contains(&"options.max_heading_level".to_owned()));
        assert!(native_options.iter().any(|option| {
            option.owner == "docnav-markdown"
                && option.namespace == "options"
                && option.key == "max_heading_level"
        }));
    }
}
