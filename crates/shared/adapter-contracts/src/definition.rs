use std::{collections::BTreeSet, fmt};

use docnav_protocol::Manifest;

use crate::{Adapter, AdapterDocument, UnstructuredFullReadCapabilities};

mod error;

pub use error::AdapterDefinitionError;

#[derive(Clone)]
pub struct AdapterDefinition<'a> {
    manifest: Manifest,
    factory: &'a dyn Adapter,
    full_read_capabilities: Option<UnstructuredFullReadCapabilities>,
}

impl<'a> AdapterDefinition<'a> {
    pub fn new(
        manifest: Manifest,
        factory: &'a dyn Adapter,
        full_read_capabilities: Option<UnstructuredFullReadCapabilities>,
    ) -> Result<Self, AdapterDefinitionError> {
        validate_full_read_capabilities(&manifest.adapter.id, full_read_capabilities.as_ref())?;
        Ok(Self {
            manifest,
            factory,
            full_read_capabilities,
        })
    }

    pub fn id(&self) -> &str {
        &self.manifest.adapter.id
    }

    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    pub fn unstructured_full_read_capabilities(&self) -> Option<&UnstructuredFullReadCapabilities> {
        self.full_read_capabilities.as_ref()
    }

    /// Creates one invocation-private document after navigation has finalized the request path.
    pub fn create_document(&self, document_path: String) -> Box<dyn AdapterDocument + 'a> {
        self.factory.create_document(document_path)
    }
}

impl fmt::Debug for AdapterDefinition<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AdapterDefinition")
            .field("manifest", &self.manifest)
            .field("full_read_capabilities", &self.full_read_capabilities)
            .finish_non_exhaustive()
    }
}

fn validate_full_read_capabilities(
    id: &str,
    capabilities: Option<&UnstructuredFullReadCapabilities>,
) -> Result<(), AdapterDefinitionError> {
    let Some(capabilities) = capabilities else {
        return Ok(());
    };
    let has_any_hook = capabilities.content_hook
        || capabilities.result_facts_hook
        || !capabilities.cost_measurement_units.is_empty();
    if !has_any_hook {
        return Err(AdapterDefinitionError::UnsupportedCapabilityCombination {
            id: id.to_owned(),
            capability: "full_read",
            reason: "full-read capabilities must declare at least one hook or cost unit",
        });
    }

    let mut units = BTreeSet::new();
    for unit in &capabilities.cost_measurement_units {
        if unit.is_empty() || !units.insert(unit) {
            return Err(AdapterDefinitionError::UnsupportedCapabilityCombination {
                id: id.to_owned(),
                capability: "full_read",
                reason: "full-read cost measurement units must be non-empty and unique",
            });
        }
    }
    Ok(())
}
