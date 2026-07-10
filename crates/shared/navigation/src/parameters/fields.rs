use docnav_adapter_contracts::{AdapterDefinition, AdapterOptionSpec};
use docnav_protocol::Operation;
use docnav_typed_fields::{
    ExpectedFieldShape, FieldBound, FieldDef, FieldDefBuilder, FieldDefSet, FieldDefSetBuilder,
    FieldLength, FieldStringEnum, FieldValidation, ProcessStrategy,
};

use crate::{NavigationAdapterRegistry, NavigationError, NavigationOutputMode};

use super::{
    ids, values::uses_document_window, CONFIG_PROCESSING, DEFAULT_LIMIT, DEFAULT_PAGE,
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

    fn all_adapter_options(
        mut self,
        registry: &(impl NavigationAdapterRegistry + ?Sized),
    ) -> Result<Self, NavigationError> {
        for adapter in registry.adapters() {
            for option in adapter.definition.native_options().iter().cloned() {
                validate_adapter_option_config_path(&option)?;
                let declaration = option
                    .field_declaration()
                    .map_err(|_| invalid_adapter_option_declaration())?;
                self.builder = self.builder.field_declaration(declaration);
                self.adapter_options.push(option);
            }
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
    selected_adapter: &AdapterDefinition<'_>,
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

    builder = builder.adapter_options(selected_adapter.native_options().to_vec(), operation)?;

    builder
        .build()
        .map_err(|_| NavigationError::internal("operation-fields-build-failed"))
}

pub(super) fn config_inspection_fields(
    registry: &(impl NavigationAdapterRegistry + ?Sized),
) -> Result<OperationFieldSet, NavigationError> {
    OperationFieldSetBuilder::new()
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
        )
        .field_with_declaration_path(
            ["pagination"],
            pagination_enabled_field(DIRECT_PROCESSING, CONFIG_PROCESSING)
                .default_static(DEFAULT_PAGINATION_ENABLED),
            ExpectedFieldShape::required(),
        )
        .field_with_declaration_path(
            ["limit"],
            configurable_limit_field(DIRECT_PROCESSING, CONFIG_PROCESSING)
                .default_static(DEFAULT_LIMIT),
            ExpectedFieldShape::required(),
        )
        .field_with_declaration_path(
            ["invocation_log", "enabled"],
            invocation_log_enabled_field(CONFIG_PROCESSING),
            ExpectedFieldShape::optional(),
        )
        .field_with_declaration_path(
            ["invocation_log", "path"],
            invocation_log_path_field(CONFIG_PROCESSING),
            ExpectedFieldShape::optional(),
        )
        .field_with_declaration_path(
            ["invocation_log", "content_capture", "enabled"],
            invocation_log_content_capture_enabled_field(CONFIG_PROCESSING),
            ExpectedFieldShape::optional(),
        )
        .field_with_declaration_path(
            ["invocation_log", "content_capture", "root"],
            invocation_log_content_capture_root_field(CONFIG_PROCESSING),
            ExpectedFieldShape::optional(),
        )
        .all_adapter_options(registry)?
        .build()
        .map_err(|_| NavigationError::internal("config-inspection-fields-build-failed"))
}

fn validate_adapter_option_config_path(option: &AdapterOptionSpec) -> Result<(), NavigationError> {
    let Some(path) = option
        .processing_path(CONFIG_PROCESSING)
        .map_err(|_| invalid_adapter_option_declaration())?
    else {
        return Ok(());
    };
    if is_adapter_option_config_path(&path, &option.owner, option.key()) {
        Ok(())
    } else {
        Err(invalid_adapter_option_declaration())
    }
}

fn is_adapter_option_config_path(path: &[String], adapter_id: &str, option_key: &str) -> bool {
    path.len() == 3
        && path.first().is_some_and(|segment| segment == "options")
        && path.get(1).is_some_and(|segment| segment == adapter_id)
        && path.get(2).is_some_and(|segment| segment == option_key)
}

fn invalid_adapter_option_declaration() -> NavigationError {
    NavigationError::internal(ADAPTER_OPTION_DECLARATION_ERROR_ID)
}

fn document_path_field(processing_id: &'static str) -> FieldDefBuilder<String> {
    direct_string_field(ids::PATH, processing_id, ["path"])
}

fn read_ref_field(processing_id: &'static str) -> FieldDefBuilder<String> {
    direct_string_field(ids::REF, processing_id, ["ref"])
}

fn find_query_field(processing_id: &'static str) -> FieldDefBuilder<String> {
    direct_string_field(ids::QUERY, processing_id, ["query"])
}

fn adapter_id_field(
    direct_processing_id: &'static str,
    config_processing_id: &'static str,
) -> FieldDefBuilder<String> {
    FieldDef::builder(ids::ADAPTER)
        .process(
            direct_processing_id,
            ProcessStrategy::json_path(["adapter"]),
        )
        .process(
            config_processing_id,
            ProcessStrategy::json_path(["defaults", "adapter"]),
        )
        .validation(non_empty_string_validation())
}

fn standard_page_field(processing_id: &'static str) -> FieldDefBuilder<i64> {
    direct_positive_u32_field(ids::PAGE, processing_id, ["page"])
}

fn configurable_limit_field(
    direct_processing_id: &'static str,
    config_processing_id: &'static str,
) -> FieldDefBuilder<i64> {
    FieldDef::builder(ids::LIMIT)
        .process(direct_processing_id, ProcessStrategy::json_path(["limit"]))
        .process(
            config_processing_id,
            ProcessStrategy::json_path(["defaults", "pagination", "limit"]),
        )
        .validation(positive_u32_int_validation())
}

fn pagination_enabled_field(
    direct_processing_id: &'static str,
    config_processing_id: &'static str,
) -> FieldDefBuilder<bool> {
    FieldDef::builder(ids::PAGINATION_ENABLED)
        .process(
            direct_processing_id,
            ProcessStrategy::json_path(["pagination"]),
        )
        .process(
            config_processing_id,
            ProcessStrategy::json_path(["defaults", "pagination", "enabled"]),
        )
        .validation(FieldValidation::boolean())
}

fn invocation_log_enabled_field(config_processing_id: &'static str) -> FieldDefBuilder<bool> {
    FieldDef::builder(ids::INVOCATION_LOG_ENABLED)
        .process(
            config_processing_id,
            ProcessStrategy::json_path(["invocation_log", "enabled"]),
        )
        .validation(FieldValidation::boolean())
}

fn invocation_log_path_field(config_processing_id: &'static str) -> FieldDefBuilder<String> {
    FieldDef::builder(ids::INVOCATION_LOG_PATH)
        .process(
            config_processing_id,
            ProcessStrategy::json_path(["invocation_log", "path"]),
        )
        .validation(non_empty_string_validation())
}

fn invocation_log_content_capture_enabled_field(
    config_processing_id: &'static str,
) -> FieldDefBuilder<bool> {
    FieldDef::builder(ids::INVOCATION_LOG_CONTENT_CAPTURE_ENABLED)
        .process(
            config_processing_id,
            ProcessStrategy::json_path(["invocation_log", "content_capture", "enabled"]),
        )
        .validation(FieldValidation::boolean())
}

fn invocation_log_content_capture_root_field(
    config_processing_id: &'static str,
) -> FieldDefBuilder<String> {
    FieldDef::builder(ids::INVOCATION_LOG_CONTENT_CAPTURE_ROOT)
        .process(
            config_processing_id,
            ProcessStrategy::json_path(["invocation_log", "content_capture", "root"]),
        )
        .validation(non_empty_string_validation())
}

fn configurable_output_field<T>(
    direct_processing_id: &'static str,
    config_processing_id: &'static str,
) -> FieldDefBuilder<T>
where
    T: FieldStringEnum,
{
    FieldDef::builder(ids::OUTPUT)
        .process(direct_processing_id, ProcessStrategy::json_path(["output"]))
        .process(
            config_processing_id,
            ProcessStrategy::json_path(["defaults", "output"]),
        )
        .validation(FieldValidation::string_enum::<T>())
}

fn direct_string_field<const N: usize>(
    identity: &str,
    processing_id: &'static str,
    direct_path: [&str; N],
) -> FieldDefBuilder<String> {
    FieldDef::builder(identity)
        .process(processing_id, ProcessStrategy::json_path(direct_path))
        .validation(non_empty_string_validation())
}

fn direct_positive_u32_field<const N: usize>(
    identity: &str,
    processing_id: &'static str,
    direct_path: [&str; N],
) -> FieldDefBuilder<i64> {
    FieldDef::builder(identity)
        .process(processing_id, ProcessStrategy::json_path(direct_path))
        .validation(positive_u32_int_validation())
}

fn non_empty_string_validation() -> FieldValidation<String> {
    FieldValidation::string().length(FieldLength::min(FieldBound::closed(1)))
}

fn positive_u32_int_validation() -> FieldValidation<i64> {
    FieldValidation::int().between(
        FieldBound::closed(1),
        FieldBound::closed(i64::from(super::MAX_PAGINATION_LIMIT)),
    )
}
