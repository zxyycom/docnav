use std::fmt;

use docnav_protocol::{
    Manifest, Operation, OperationArguments, OperationResult, ProbeResult, RequestEnvelope,
};

use crate::{
    Adapter, AdapterError, AdapterOptionSpec, AdapterResult, NativeOptionHandoff,
    UnstructuredFullRead, UnstructuredFullReadCapabilities, UnstructuredFullReadFacts,
};

mod builder;
mod error;

pub use builder::AdapterDefinitionBuilder;
pub use error::AdapterDefinitionError;

pub(super) const REQUIRED_OPERATIONS: [Operation; 4] = [
    Operation::Outline,
    Operation::Read,
    Operation::Find,
    Operation::Info,
];

#[derive(Clone)]
pub struct AdapterDefinition<'a> {
    adapter: &'a dyn Adapter,
    id: String,
    manifest: Manifest,
    operation_handlers: AdapterOperationHandlers,
    native_options: Vec<AdapterOptionSpec>,
    full_read: Option<FullReadCapabilityGroup>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterOperationHandlers {
    operations: Vec<Operation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FullReadCapabilityGroup {
    capabilities: UnstructuredFullReadCapabilities,
}

impl<'a> AdapterDefinition<'a> {
    pub fn builder(id: impl Into<String>) -> AdapterDefinitionBuilder<'a> {
        AdapterDefinitionBuilder::new(id)
    }

    pub fn transition_from_adapter(
        adapter: &'a dyn Adapter,
    ) -> Result<Self, AdapterDefinitionError> {
        let mut builder = Self::builder(adapter.adapter_id())
            .adapter(adapter)
            .manifest(adapter.manifest())
            .required_operation_handlers()
            .native_options(adapter.adapter_options());
        let capabilities = adapter.unstructured_full_read_capabilities();
        if capabilities != UnstructuredFullReadCapabilities::default() {
            builder = builder.full_read_capability_group(capabilities);
        }
        builder.build()
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    pub fn adapter(&self) -> &'a dyn Adapter {
        self.adapter
    }

    pub fn native_options(&self) -> &[AdapterOptionSpec] {
        &self.native_options
    }

    pub fn operation_handlers(&self) -> &AdapterOperationHandlers {
        &self.operation_handlers
    }

    pub fn full_read_capability_group(&self) -> Option<&FullReadCapabilityGroup> {
        self.full_read.as_ref()
    }

    pub fn unstructured_full_read_capabilities(&self) -> UnstructuredFullReadCapabilities {
        self.full_read
            .as_ref()
            .map(FullReadCapabilityGroup::capabilities)
            .cloned()
            .unwrap_or_default()
    }

    pub fn probe(&self, path: &str) -> ProbeResult {
        self.adapter.probe(path)
    }

    pub fn execute_operation(
        &self,
        request: &RequestEnvelope,
        native_options: &NativeOptionHandoff,
    ) -> AdapterResult<OperationResult> {
        if !self.operation_handlers.supports(request.operation) {
            return Err(AdapterError::internal(format!(
                "adapter-operation-{}-handler-undeclared",
                request.operation
            )));
        }
        match (&request.operation, &request.arguments) {
            (Operation::Outline, OperationArguments::Outline(arguments)) => self
                .adapter
                .outline_with_native_options(request, arguments, native_options)
                .map(OperationResult::Outline),
            (Operation::Read, OperationArguments::Read(arguments)) => self
                .adapter
                .read_with_native_options(request, arguments, native_options)
                .map(OperationResult::Read),
            (Operation::Find, OperationArguments::Find(arguments)) => self
                .adapter
                .find_with_native_options(request, arguments, native_options)
                .map(OperationResult::Find),
            (Operation::Info, OperationArguments::Info(arguments)) => self
                .adapter
                .info_with_native_options(request, arguments, native_options)
                .map(OperationResult::Info),
            _ => Err(AdapterError::invalid_request(
                "arguments",
                format!("arguments do not match operation {}", request.operation),
            )),
        }
    }

    pub fn unstructured_full_read(
        &self,
        request: &RequestEnvelope,
    ) -> AdapterResult<UnstructuredFullRead> {
        self.adapter.unstructured_full_read(request)
    }

    pub fn measure_unstructured_full_read_cost(
        &self,
        request: &RequestEnvelope,
        requested_units: &[String],
    ) -> AdapterResult<docnav_protocol::Cost> {
        self.adapter
            .measure_unstructured_full_read_cost(request, requested_units)
    }

    pub fn unstructured_full_read_facts(
        &self,
        request: &RequestEnvelope,
    ) -> AdapterResult<UnstructuredFullReadFacts> {
        self.adapter.unstructured_full_read_facts(request)
    }
}

impl fmt::Debug for AdapterDefinition<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AdapterDefinition")
            .field("id", &self.id)
            .field("manifest", &self.manifest)
            .field("operation_handlers", &self.operation_handlers)
            .field("native_options", &self.native_options)
            .field("full_read", &self.full_read)
            .finish_non_exhaustive()
    }
}

impl AdapterOperationHandlers {
    pub(super) fn new(operations: Vec<Operation>) -> Self {
        Self { operations }
    }

    pub fn required() -> Self {
        Self {
            operations: REQUIRED_OPERATIONS.to_vec(),
        }
    }

    pub fn supports(&self, operation: Operation) -> bool {
        self.operations.contains(&operation)
    }

    pub fn operations(&self) -> &[Operation] {
        &self.operations
    }
}

impl FullReadCapabilityGroup {
    pub fn new(capabilities: UnstructuredFullReadCapabilities) -> Self {
        Self { capabilities }
    }

    pub fn capabilities(&self) -> &UnstructuredFullReadCapabilities {
        &self.capabilities
    }

    pub fn has_cost_measurement_unit(&self, unit: &str) -> bool {
        self.capabilities.has_cost_measurement_unit(unit)
    }
}
