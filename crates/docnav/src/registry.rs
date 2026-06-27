use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::{AppError, AppResult};
use crate::project_context::ProjectContext;
use crate::project_paths::{path_to_slash, resolve_project_relative_command};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterRegistry {
    pub path: PathBuf,
    pub adapters: Vec<AdapterRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterRecord {
    pub id: String,
    pub command: String,
    pub command_path: PathBuf,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RegistryFile {
    version: u32,
    adapters: Vec<RegistryAdapter>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RegistryAdapter {
    id: String,
    command: String,
}

impl AdapterRegistry {
    pub fn load(project: &ProjectContext) -> AppResult<Self> {
        let path = registry_path(project);
        if !path.exists() {
            return Ok(Self {
                path,
                adapters: Vec::new(),
            });
        }

        let content = fs::read_to_string(&path).map_err(|error| {
            AppError::invalid_request(
                "adapter_registry",
                format!("failed to read {}: {error}", path_to_slash(&path)),
            )
        })?;
        let registry: RegistryFile = serde_json::from_str(&content).map_err(|error| {
            AppError::invalid_request(
                "adapter_registry",
                format!("failed to parse {}: {error}", path_to_slash(&path)),
            )
        })?;
        if registry.version != 1 {
            return Err(AppError::invalid_request(
                "adapter_registry.version",
                "temporary adapter registry version must be 1",
            ));
        }

        let mut seen = HashSet::new();
        let adapters = registry
            .adapters
            .into_iter()
            .map(|adapter| parse_record(project, adapter, &mut seen))
            .collect::<AppResult<Vec<_>>>()?;

        Ok(Self { path, adapters })
    }

    pub fn find(&self, adapter_id: &str) -> Option<&AdapterRecord> {
        self.adapters
            .iter()
            .find(|adapter| adapter.id == adapter_id)
    }
}

pub fn registry_path(project: &ProjectContext) -> PathBuf {
    project.project_root.join(".docnav").join("adapters.json")
}

fn parse_record(
    project: &ProjectContext,
    adapter: RegistryAdapter,
    seen: &mut HashSet<String>,
) -> AppResult<AdapterRecord> {
    if adapter.id.is_empty() {
        return Err(AppError::invalid_request(
            "adapter_registry.adapters[].id",
            "adapter id must not be empty",
        ));
    }
    if !seen.insert(adapter.id.clone()) {
        return Err(AppError::invalid_request(
            "adapter_registry.adapters[].id",
            format!("duplicate adapter id {:?}", adapter.id),
        ));
    }
    let command_path = resolve_project_relative_command(&project.project_root, &adapter.command)
        .map_err(|reason| {
            AppError::invalid_request(
                "adapter_registry.adapters[].command",
                format!("adapter {} has invalid command: {reason}", adapter.id),
            )
        })?;

    Ok(AdapterRecord {
        id: adapter.id,
        command: adapter.command,
        command_path,
    })
}

pub fn registry_check(path: &Path, result: &AppResult<AdapterRegistry>) -> serde_json::Value {
    match result {
        Ok(registry) if path.exists() => serde_json::json!({
            "name": "temporary_adapter_registry",
            "status": "pass",
            "path": path_to_slash(path),
            "message": "registry is readable",
            "adapter_count": registry.adapters.len(),
        }),
        Ok(_) => serde_json::json!({
            "name": "temporary_adapter_registry",
            "status": "pass",
            "path": path_to_slash(path),
            "message": "registry file is absent; no adapters are registered",
            "adapter_count": 0,
        }),
        Err(error) => {
            let diagnostic = error.diagnostic();
            serde_json::json!({
                "name": "temporary_adapter_registry",
                "status": "fail",
                "path": path_to_slash(path),
                "message": diagnostic.summary(),
                "details": diagnostic.details().to_value(),
            })
        }
    }
}
