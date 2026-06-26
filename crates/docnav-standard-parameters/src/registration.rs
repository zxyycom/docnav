use docnav_typed_fields::{FieldIdentity, SchemaMetadataView};

use crate::StandardParameterPath;

#[derive(Clone, Debug, PartialEq)]
pub struct OperationArgumentBinding {
    pub arguments_path: StandardParameterPath,
}

impl OperationArgumentBinding {
    pub fn new(arguments_path: StandardParameterPath) -> Self {
        Self { arguments_path }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StandardParameterRegistration {
    pub metadata: SchemaMetadataView,
    pub operation_argument: Option<OperationArgumentBinding>,
}

impl StandardParameterRegistration {
    pub fn new(metadata: SchemaMetadataView) -> Self {
        Self {
            metadata,
            operation_argument: None,
        }
    }

    pub fn with_operation_argument(mut self, binding: OperationArgumentBinding) -> Self {
        self.operation_argument = Some(binding);
        self
    }

    pub fn identity(&self) -> &FieldIdentity {
        &self.metadata.identity
    }
}
