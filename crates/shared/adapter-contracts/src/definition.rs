use std::{collections::BTreeSet, fmt};

use docnav_protocol::{Manifest, OperationResult, ProbeResult, RequestEnvelope};

use crate::{
    Adapter, AdapterResult, StandardOperationInput, UnstructuredFullRead,
    UnstructuredFullReadCapabilities, UnstructuredFullReadFacts,
};

mod error;

pub use error::AdapterDefinitionError;

#[derive(Clone)]
pub struct AdapterDefinition<'a> {
    manifest: Manifest,
    strategy: &'a dyn Adapter,
    full_read_capabilities: Option<UnstructuredFullReadCapabilities>,
}

impl<'a> AdapterDefinition<'a> {
    pub fn new(
        manifest: Manifest,
        strategy: &'a dyn Adapter,
        full_read_capabilities: Option<UnstructuredFullReadCapabilities>,
    ) -> Result<Self, AdapterDefinitionError> {
        validate_full_read_capabilities(&manifest.adapter.id, full_read_capabilities.as_ref())?;
        Ok(Self {
            manifest,
            strategy,
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

    pub fn probe(&self, path: &str) -> ProbeResult {
        self.strategy.probe(path)
    }

    pub fn execute_operation(
        &self,
        input: &StandardOperationInput,
    ) -> AdapterResult<OperationResult> {
        match input {
            StandardOperationInput::Outline(input) => {
                self.strategy.outline(input).map(OperationResult::Outline)
            }
            StandardOperationInput::Read(input) => {
                self.strategy.read(input).map(OperationResult::Read)
            }
            StandardOperationInput::Find(input) => {
                self.strategy.find(input).map(OperationResult::Find)
            }
            StandardOperationInput::Info(input) => {
                self.strategy.info(input).map(OperationResult::Info)
            }
        }
    }

    pub fn unstructured_full_read(
        &self,
        request: &RequestEnvelope,
    ) -> AdapterResult<UnstructuredFullRead> {
        self.strategy.unstructured_full_read(request)
    }

    pub fn measure_unstructured_full_read_cost(
        &self,
        request: &RequestEnvelope,
        requested_units: &[String],
    ) -> AdapterResult<docnav_protocol::Cost> {
        self.strategy
            .measure_unstructured_full_read_cost(request, requested_units)
    }

    pub fn unstructured_full_read_facts(
        &self,
        request: &RequestEnvelope,
    ) -> AdapterResult<UnstructuredFullReadFacts> {
        self.strategy.unstructured_full_read_facts(request)
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
