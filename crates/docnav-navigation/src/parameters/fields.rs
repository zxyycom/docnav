use docnav_adapter_contracts::{NativeOptionDefaultValue, NativeOptionSpec, NativeOptionValueSpec};
use docnav_parameter_resolution::{
    adapter_id_field, configurable_limit_field, configurable_output_field, document_path_field,
    find_query_field, page_field as standard_page_field, pagination_enabled_field, read_ref_field,
};
use docnav_protocol::Operation;
use docnav_typed_fields::{
    ExpectedFieldShape, FieldBound, FieldDef, FieldDefSet, FieldValidation, ProcessStrategy,
};

use crate::{NavigationError, NavigationOutputMode};

use super::{
    values::uses_document_window, CONFIG_PROCESSING, DEFAULT_LIMIT, DEFAULT_PAGE,
    DEFAULT_PAGINATION_ENABLED, DIRECT_PROCESSING,
};

pub(super) fn adapter_intent_fields() -> Result<FieldDefSet, NavigationError> {
    FieldDefSet::builder()
        .field_with_declaration_path(
            ["adapter"],
            adapter_id_field(DIRECT_PROCESSING, CONFIG_PROCESSING),
            ExpectedFieldShape::optional(),
        )
        .build()
        .map_err(|error| NavigationError::internal(format!("adapter-intent-fields:{error}")))
}

pub(super) fn operation_fields(
    operation: Operation,
    selected_native_options: &[NativeOptionSpec],
) -> Result<FieldDefSet, NavigationError> {
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

    for spec in selected_native_options
        .iter()
        .copied()
        .filter(|spec| spec.applies_to(operation))
    {
        builder = add_native_option_field(builder, spec);
    }

    builder
        .build()
        .map_err(|error| NavigationError::internal(format!("operation-fields:{error}")))
}

fn add_native_option_field(
    builder: docnav_typed_fields::FieldDefSetBuilder,
    spec: NativeOptionSpec,
) -> docnav_typed_fields::FieldDefSetBuilder {
    let base = FieldDef::builder(spec.identity)
        .process(
            DIRECT_PROCESSING,
            ProcessStrategy::json_path(["options", spec.key]),
        )
        .process(
            CONFIG_PROCESSING,
            ProcessStrategy::json_path(["options", spec.key]),
        );
    match spec.value {
        NativeOptionValueSpec::Integer { min, max } => {
            let field = base.validation(
                FieldValidation::int().between(FieldBound::closed(min), FieldBound::closed(max)),
            );
            let field = match spec.default {
                Some(NativeOptionDefaultValue::Integer(default)) => field.default_static(default),
                _ => field,
            };
            builder.field_with_declaration_path(
                ["options", spec.key],
                field,
                ExpectedFieldShape::optional(),
            )
        }
        NativeOptionValueSpec::String => builder.field_with_declaration_path(
            ["options", spec.key],
            native_string_field(base, spec),
            ExpectedFieldShape::optional(),
        ),
        NativeOptionValueSpec::Boolean => builder.field_with_declaration_path(
            ["options", spec.key],
            native_boolean_field(base, spec),
            ExpectedFieldShape::optional(),
        ),
        NativeOptionValueSpec::Json => builder.field_with_declaration_path(
            ["options", spec.key],
            base.validation(FieldValidation::json()),
            ExpectedFieldShape::optional(),
        ),
    }
}

fn native_string_field(
    base: docnav_typed_fields::FieldDefBuilder,
    spec: NativeOptionSpec,
) -> docnav_typed_fields::FieldDefBuilder<String> {
    let field = base.validation(FieldValidation::string());
    match spec.default {
        Some(NativeOptionDefaultValue::String(default)) => field.default_static(default.to_owned()),
        _ => field,
    }
}

fn native_boolean_field(
    base: docnav_typed_fields::FieldDefBuilder,
    spec: NativeOptionSpec,
) -> docnav_typed_fields::FieldDefBuilder<bool> {
    let field = base.validation(FieldValidation::boolean());
    match spec.default {
        Some(NativeOptionDefaultValue::Boolean(default)) => field.default_static(default),
        _ => field,
    }
}
