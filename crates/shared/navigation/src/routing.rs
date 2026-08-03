use std::collections::HashMap;

use docnav_adapter_contracts::AdapterDefinition;
use docnav_diagnostics::{
    typed_codes, AdapterUnavailableDetails, DiagnosticSource, FormatUnknownDetails,
};
use docnav_protocol::protocol_error_record_draft;

use crate::NavigationError;

pub trait NavigationAdapterRegistry {
    fn adapters(&self) -> Vec<AdapterDefinition<'_>>;

    fn routing(&self) -> Result<RegistryRouting, RegistryRoutingError> {
        RegistryRouting::from_adapters(&self.adapters())
    }

    fn find_adapter(&self, adapter_id: &str) -> Option<AdapterDefinition<'_>> {
        self.adapters()
            .into_iter()
            .find(|adapter| adapter.id() == adapter_id)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegistryRoutingError {
    ManifestInvalid,
    FormatIdentityConflict,
    PathHintConflict,
}

impl RegistryRoutingError {
    pub const fn error_id(self) -> &'static str {
        match self {
            Self::ManifestInvalid => "registry-manifest-invalid",
            Self::FormatIdentityConflict => "registry-format-identity-conflict",
            Self::PathHintConflict => "registry-path-hint-conflict",
        }
    }
}

#[derive(Clone, Debug)]
pub struct RegistryRouting {
    formats: HashMap<String, String>,
    filenames: HashMap<String, String>,
    suffixes: HashMap<String, String>,
}

impl RegistryRouting {
    pub fn from_adapters(adapters: &[AdapterDefinition<'_>]) -> Result<Self, RegistryRoutingError> {
        let mut routing = Self {
            formats: HashMap::new(),
            filenames: HashMap::new(),
            suffixes: HashMap::new(),
        };

        for adapter in adapters {
            adapter
                .manifest()
                .validate_semantics()
                .map_err(|_| RegistryRoutingError::ManifestInvalid)?;
            for format in &adapter.manifest().formats {
                if routing
                    .formats
                    .insert(format.id.clone(), adapter.id().to_owned())
                    .is_some()
                {
                    return Err(RegistryRoutingError::FormatIdentityConflict);
                }
                for filename in &format.filenames {
                    if routing
                        .filenames
                        .insert(filename.clone(), format.id.clone())
                        .is_some()
                    {
                        return Err(RegistryRoutingError::PathHintConflict);
                    }
                }
                for suffix in &format.extensions {
                    if routing
                        .suffixes
                        .insert(suffix.to_ascii_lowercase(), format.id.clone())
                        .is_some()
                    {
                        return Err(RegistryRoutingError::PathHintConflict);
                    }
                }
            }
        }

        Ok(routing)
    }

    fn adapter_id_for_path(&self, pathname: &str) -> Option<&str> {
        let basename = complete_basename(pathname);
        let format_id = self.filenames.get(basename).or_else(|| {
            let normalized_basename = basename.to_ascii_lowercase();
            self.suffixes
                .iter()
                .filter(|(suffix, _)| normalized_basename.ends_with(suffix.as_str()))
                .max_by_key(|(suffix, _)| suffix.len())
                .map(|(_, format_id)| format_id)
        })?;
        self.formats.get(format_id).map(String::as_str)
    }
}

#[derive(Clone, Debug)]
pub struct AdapterSelection<'a> {
    pub adapter: AdapterDefinition<'a>,
}

#[derive(Clone, Copy)]
pub struct AdapterSelectionRequest<'registry, 'input, R>
where
    R: NavigationAdapterRegistry + ?Sized,
{
    pub registry: &'registry R,
    pub document_path: &'input str,
    pub preselected_adapter_id: Option<&'input str>,
    pub preselected_adapter_source: &'input str,
}

pub fn select_adapter<'registry, 'input, R>(
    request: AdapterSelectionRequest<'registry, 'input, R>,
) -> Result<AdapterSelection<'registry>, NavigationError>
where
    R: NavigationAdapterRegistry + ?Sized,
{
    let AdapterSelectionRequest {
        registry,
        document_path,
        preselected_adapter_id,
        preselected_adapter_source,
    } = request;
    let routing = registry
        .routing()
        .map_err(|error| NavigationError::internal(error.error_id()))?;
    if let Some(adapter_id) = preselected_adapter_id {
        let adapter = registry
            .find_adapter(adapter_id)
            .ok_or_else(|| explicit_adapter_error(adapter_id, preselected_adapter_source))?;
        return Ok(AdapterSelection { adapter });
    }

    let adapter_id = routing
        .adapter_id_for_path(document_path)
        .ok_or_else(|| format_unknown(document_path))?;
    let adapter = registry
        .find_adapter(adapter_id)
        .ok_or_else(|| NavigationError::internal("registry-format-definition-missing"))?;
    Ok(AdapterSelection { adapter })
}

fn complete_basename(pathname: &str) -> &str {
    pathname.rsplit(['/', '\\']).next().unwrap_or(pathname)
}

fn explicit_adapter_error(adapter_id: &str, selection_source: &str) -> NavigationError {
    NavigationError::new(protocol_error_record_draft::<
        typed_codes::protocol::AdapterUnavailable,
    >(
        AdapterUnavailableDetails::new(adapter_id, selection_source),
        DiagnosticSource::with_stage("docnav-navigation", "routing"),
    ))
}

fn format_unknown(document_path: &str) -> NavigationError {
    NavigationError::new(protocol_error_record_draft::<
        typed_codes::protocol::FormatUnknown,
    >(
        FormatUnknownDetails::new(document_path),
        DiagnosticSource::with_stage("docnav-navigation", "routing"),
    ))
}
