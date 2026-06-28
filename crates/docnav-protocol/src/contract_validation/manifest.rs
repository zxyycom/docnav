use serde_json::Value;

use docnav_typed_fields::{
    ExpectedFieldShape, FieldBound, FieldDef, FieldDefSet, FieldDefSetBuildError, FieldLength,
    FieldValidation,
};

use crate::constants::schema_names;
use crate::SchemaValidationError;

use super::field_builders::{
    json_path, non_empty_array_field, non_empty_string_field, non_empty_unique_array, object_field,
    operation_value_fields, string_enum_field, value_field_set,
};
use super::helpers::{
    reject_unknown_fields, schema_result, validate_field_set, validate_object_array_items,
    validate_value_array_items, validate_value_array_items_with_owned_prefix, ObjectArraySpec,
    ValueArraySpec,
};

pub(crate) fn validate_manifest_contract_value(value: &Value) -> Result<(), SchemaValidationError> {
    let mut errors = Vec::new();
    validate_field_set(
        schema_names::MANIFEST,
        manifest_fields,
        value,
        &[],
        &mut errors,
    );
    reject_unknown_fields(
        schema_names::MANIFEST,
        manifest_fields,
        value,
        &[],
        &mut errors,
    );
    reject_unknown_fields(
        schema_names::MANIFEST,
        manifest_fields,
        value,
        &["adapter"],
        &mut errors,
    );

    validate_object_array_items(
        value,
        &["formats"],
        ObjectArraySpec {
            schema: schema_names::MANIFEST,
            build: manifest_format_fields,
        },
        |format, path, errors| {
            validate_value_array_items_with_owned_prefix(
                format,
                &["extensions"],
                path,
                ValueArraySpec {
                    schema: schema_names::MANIFEST,
                    build: manifest_extension_fields,
                },
                errors,
            );
            validate_value_array_items_with_owned_prefix(
                format,
                &["content_types"],
                path,
                ValueArraySpec {
                    schema: schema_names::MANIFEST,
                    build: manifest_content_type_fields,
                },
                errors,
            );
        },
        &mut errors,
    );
    validate_value_array_items(
        value,
        &["capabilities"],
        &[],
        ValueArraySpec {
            schema: schema_names::MANIFEST,
            build: operation_value_fields,
        },
        &mut errors,
    );
    schema_result(schema_names::MANIFEST, errors)
}

fn manifest_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    FieldDefSet::builder()
        .__field_with_declaration_path(
            ["manifest_version"],
            string_enum_field::<super::enums::ContractVersion>(
                "manifest_version",
                ["manifest_version"],
            ),
            ExpectedFieldShape::required(),
        )
        .__field_with_declaration_path(
            ["adapter"],
            object_field("adapter", ["adapter"]),
            ExpectedFieldShape::required(),
        )
        .__field_with_declaration_path(
            ["adapter", "id"],
            non_empty_string_field("adapter.id", ["adapter", "id"]),
            ExpectedFieldShape::required(),
        )
        .__field_with_declaration_path(
            ["adapter", "name"],
            non_empty_string_field("adapter.name", ["adapter", "name"]),
            ExpectedFieldShape::required(),
        )
        .__field_with_declaration_path(
            ["adapter", "version"],
            non_empty_string_field("adapter.version", ["adapter", "version"]),
            ExpectedFieldShape::required(),
        )
        .__field_with_declaration_path(
            ["formats"],
            non_empty_array_field("formats", ["formats"]),
            ExpectedFieldShape::required(),
        )
        .__field_with_declaration_path(
            ["capabilities"],
            FieldDef::builder("capabilities")
                .process(super::JSON_CONTRACT_PROCESSING, json_path(["capabilities"]))
                .validation(non_empty_unique_array()),
            ExpectedFieldShape::required(),
        )
        .build()
}

fn manifest_format_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    FieldDefSet::builder()
        .__field_with_declaration_path(
            ["id"],
            non_empty_string_field("formats[].id", ["id"]),
            ExpectedFieldShape::required(),
        )
        .__field_with_declaration_path(
            ["extensions"],
            non_empty_array_field("formats[].extensions", ["extensions"]),
            ExpectedFieldShape::required(),
        )
        .__field_with_declaration_path(
            ["content_types"],
            non_empty_array_field("formats[].content_types", ["content_types"]),
            ExpectedFieldShape::required(),
        )
        .build()
}

fn manifest_extension_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    value_field_set(
        "formats[].extensions[]",
        FieldValidation::string().regex(r"^\.[A-Za-z0-9][A-Za-z0-9._-]*$"),
    )
}

fn manifest_content_type_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    value_field_set(
        "formats[].content_types[]",
        FieldValidation::string().length(FieldLength::min(FieldBound::closed(1))),
    )
}
