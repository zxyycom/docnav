use docnav_adapter_contracts::{Adapter, AdapterOptionSpec};
use docnav_parameter_resolution::{
    adapter_id_field, configurable_limit_field, configurable_output_field, document_path_field,
    find_query_field, page_field as standard_page_field, pagination_enabled_field, read_ref_field,
};
use docnav_protocol::Operation;
use docnav_typed_fields::{ExpectedFieldShape, FieldDefBuilder, FieldDefSet, FieldDefSetBuilder};

use crate::{NavigationError, NavigationOutputMode};

use super::{
    values::uses_document_window, CONFIG_PROCESSING, DEFAULT_LIMIT, DEFAULT_PAGE,
    DEFAULT_PAGINATION_ENABLED, DIRECT_PROCESSING,
};

const ADAPTER_OPTION_DECLARATION_ERROR_ID: &str = "adapter-option-field-declaration-invalid";

pub(super) fn adapter_intent_fields() -> Result<FieldDefSet, NavigationError> {
    FieldDefSet::builder()
        .field_with_declaration_path(
            ["adapter"],
            adapter_id_field(DIRECT_PROCESSING, CONFIG_PROCESSING),
            ExpectedFieldShape::optional(),
        )
        .build()
        .map_err(|_| NavigationError::internal("adapter-intent-fields-build-failed"))
}

pub(super) struct OperationFieldSet {
    fields: FieldDefSet,
    adapter_options: Vec<AdapterOptionSpec>,
}

impl OperationFieldSet {
    pub(super) fn adapter_options(&self) -> &[AdapterOptionSpec] {
        &self.adapter_options
    }
}

impl AsRef<FieldDefSet> for OperationFieldSet {
    fn as_ref(&self) -> &FieldDefSet {
        &self.fields
    }
}

struct OperationFieldSetBuilder {
    builder: FieldDefSetBuilder,
    adapter_options: Vec<AdapterOptionSpec>,
}

impl OperationFieldSetBuilder {
    fn new() -> Self {
        Self {
            builder: FieldDefSet::builder(),
            adapter_options: Vec::new(),
        }
    }

    fn field_with_declaration_path<T, I, S>(
        mut self,
        declaration_path: I,
        builder: FieldDefBuilder<T>,
        expected: ExpectedFieldShape,
    ) -> Self
    where
        T: 'static,
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.builder =
            self.builder
                .field_with_declaration_path(declaration_path, builder, expected);
        self
    }

    fn adapter_options(
        mut self,
        options: Vec<AdapterOptionSpec>,
        operation: Operation,
    ) -> Result<Self, NavigationError> {
        for option in options {
            if !option.applies_to(operation) {
                continue;
            }
            validate_adapter_option_config_path(&option)?;
            let declaration = option
                .field_declaration()
                .map_err(|_| invalid_adapter_option_declaration())?;
            self.builder = self.builder.field_declaration(declaration);
            self.adapter_options.push(option);
        }
        Ok(self)
    }

    fn build(self) -> Result<OperationFieldSet, docnav_typed_fields::FieldDefSetBuildError> {
        Ok(OperationFieldSet {
            fields: self.builder.build()?,
            adapter_options: self.adapter_options,
        })
    }
}

pub(super) fn operation_fields(
    operation: Operation,
    selected_adapter: &dyn Adapter,
) -> Result<OperationFieldSet, NavigationError> {
    let mut builder = OperationFieldSetBuilder::new()
        .field_with_declaration_path(
            ["path"],
            document_path_field(DIRECT_PROCESSING),
            ExpectedFieldShape::required(),
        )
        .field_with_declaration_path(
            ["adapter"],
            adapter_id_field(DIRECT_PROCESSING, CONFIG_PROCESSING),
            ExpectedFieldShape::optional(),
        )
        .field_with_declaration_path(
            ["output"],
            configurable_output_field::<NavigationOutputMode>(DIRECT_PROCESSING, CONFIG_PROCESSING)
                .default_static(NavigationOutputMode::ReadableView),
            ExpectedFieldShape::required(),
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

    if uses_document_window(operation) {
        builder = builder
            .field_with_declaration_path(
                ["pagination"],
                pagination_enabled_field(DIRECT_PROCESSING, CONFIG_PROCESSING)
                    .default_static(DEFAULT_PAGINATION_ENABLED),
                ExpectedFieldShape::required(),
            )
            .field_with_declaration_path(
                ["page"],
                standard_page_field(DIRECT_PROCESSING).default_static(DEFAULT_PAGE),
                ExpectedFieldShape::required(),
            )
            .field_with_declaration_path(
                ["limit"],
                configurable_limit_field(DIRECT_PROCESSING, CONFIG_PROCESSING)
                    .default_static(DEFAULT_LIMIT),
                ExpectedFieldShape::required(),
            );
    }

    builder = builder.adapter_options(selected_adapter.adapter_options(), operation)?;

    builder
        .build()
        .map_err(|_| NavigationError::internal("operation-fields-build-failed"))
}

fn validate_adapter_option_config_path(option: &AdapterOptionSpec) -> Result<(), NavigationError> {
    let Some(path) = option
        .processing_path(CONFIG_PROCESSING)
        .map_err(|_| invalid_adapter_option_declaration())?
    else {
        return Ok(());
    };
    if is_adapter_option_config_path(&path) {
        Ok(())
    } else {
        Err(invalid_adapter_option_declaration())
    }
}

fn is_adapter_option_config_path(path: &[String]) -> bool {
    path.len() == 2
        && path.first().is_some_and(|segment| segment == "options")
        && path.get(1).is_some_and(|segment| !segment.is_empty())
}

fn invalid_adapter_option_declaration() -> NavigationError {
    NavigationError::internal(ADAPTER_OPTION_DECLARATION_ERROR_ID)
}
