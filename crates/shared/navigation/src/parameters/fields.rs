use docnav_protocol::Operation;
use docnav_typed_fields::{
    ExpectedFieldShape, FieldDefSet, FieldDefSetBuildError, FieldDefSetBuilder,
};

use crate::NavigationError;

use super::{catalog::DocumentParameterCatalog, CONFIG_PROCESSING, DIRECT_PROCESSING};

mod definitions;

#[cfg(test)]
mod tests;

use definitions::{
    adapter_id_field, document_path_field, find_query_field,
    invocation_log_content_capture_enabled_field, invocation_log_content_capture_root_field,
    invocation_log_enabled_field, invocation_log_path_field, read_ref_field,
};

pub(crate) fn adapter_routing_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    FieldDefSet::builder()
        .field_with_declaration_path(
            ["adapter"],
            adapter_id_field(DIRECT_PROCESSING, CONFIG_PROCESSING),
            ExpectedFieldShape::optional(),
        )
        .build()
}

pub(crate) fn invocation_log_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    FieldDefSet::builder()
        .field(
            invocation_log_enabled_field(CONFIG_PROCESSING),
            ExpectedFieldShape::optional(),
        )
        .field(
            invocation_log_path_field(CONFIG_PROCESSING),
            ExpectedFieldShape::optional(),
        )
        .field(
            invocation_log_content_capture_enabled_field(CONFIG_PROCESSING),
            ExpectedFieldShape::optional(),
        )
        .field(
            invocation_log_content_capture_root_field(CONFIG_PROCESSING),
            ExpectedFieldShape::optional(),
        )
        .build()
}

pub(super) fn adapter_intent_fields() -> Result<FieldDefSet, NavigationError> {
    adapter_routing_fields()
        .map_err(|_| NavigationError::internal("adapter-intent-fields-build-failed"))
}

pub(super) struct OperationFieldSet {
    fields: FieldDefSet,
}

impl AsRef<FieldDefSet> for OperationFieldSet {
    fn as_ref(&self) -> &FieldDefSet {
        &self.fields
    }
}

pub(super) fn operation_fields(
    operation: Operation,
    selected_adapter_id: &str,
    catalog: &DocumentParameterCatalog,
) -> Result<OperationFieldSet, NavigationError> {
    let fields = common_operation_fields(operation)
        .build()
        .and_then(|fields| {
            fields.with_fields(catalog.selected_operation_fields(selected_adapter_id, operation))
        })
        .map_err(|_| NavigationError::internal("operation-fields-build-failed"))?;
    Ok(OperationFieldSet { fields })
}

fn common_operation_fields(operation: Operation) -> FieldDefSetBuilder {
    let mut builder = FieldDefSet::builder()
        .field_with_declaration_path(
            ["path"],
            document_path_field(DIRECT_PROCESSING),
            ExpectedFieldShape::required(),
        )
        .field_with_declaration_path(
            ["adapter"],
            adapter_id_field(DIRECT_PROCESSING, CONFIG_PROCESSING),
            ExpectedFieldShape::optional(),
        );

    builder = match operation {
        Operation::Read => builder.field_with_declaration_path(
            ["ref"],
            read_ref_field(DIRECT_PROCESSING),
            ExpectedFieldShape::required(),
        ),
        Operation::Find => builder.field_with_declaration_path(
            ["query"],
            find_query_field(DIRECT_PROCESSING),
            ExpectedFieldShape::required(),
        ),
        Operation::Outline | Operation::Info => builder,
    };

    builder
}
