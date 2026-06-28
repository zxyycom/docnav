use serde_json::Value;

use docnav_typed_fields::{ExpectedFieldShape, FieldDefSet, FieldDefSetBuildError};

use crate::constants::schema_names;
use crate::SchemaValidationError;

use super::field_builders::{
    add_operation, add_protocol_version, add_request_id, non_empty_string_field, object_field,
    positive_int_field,
};
use super::helpers::{
    operation_name, reject_unknown_fields, schema_result, validate_field_set, value_at,
};

pub(crate) fn validate_protocol_request_contract_value(
    value: &Value,
) -> Result<(), SchemaValidationError> {
    let mut errors = Vec::new();
    validate_field_set(
        schema_names::PROTOCOL_REQUEST,
        request_base_fields,
        value,
        &[],
        &mut errors,
    );
    reject_unknown_fields(
        Some(value),
        &[],
        &[
            "protocol_version",
            "request_id",
            "operation",
            "document",
            "arguments",
        ],
        &mut errors,
    );
    reject_unknown_fields(
        value_at(value, &["document"]),
        &["document"],
        &["path"],
        &mut errors,
    );

    match operation_name(value) {
        Some("outline") => validate_field_set(
            schema_names::PROTOCOL_REQUEST,
            request_outline_argument_fields,
            value,
            &[],
            &mut errors,
        ),
        Some("read") => validate_field_set(
            schema_names::PROTOCOL_REQUEST,
            request_read_argument_fields,
            value,
            &[],
            &mut errors,
        ),
        Some("find") => validate_field_set(
            schema_names::PROTOCOL_REQUEST,
            request_find_argument_fields,
            value,
            &[],
            &mut errors,
        ),
        Some("info") => validate_field_set(
            schema_names::PROTOCOL_REQUEST,
            request_info_argument_fields,
            value,
            &[],
            &mut errors,
        ),
        _ => {}
    }
    schema_result(schema_names::PROTOCOL_REQUEST, errors)
}

fn request_base_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    add_operation(
        add_request_id(add_protocol_version(
            FieldDefSet::builder(),
            "protocol_version",
        )),
        ExpectedFieldShape::required(),
    )
    .__field_with_declaration_path(
        ["document"],
        object_field("document", ["document"]),
        ExpectedFieldShape::required(),
    )
    .__field_with_declaration_path(
        ["document", "path"],
        non_empty_string_field("document.path", ["document", "path"]),
        ExpectedFieldShape::required(),
    )
    .__field_with_declaration_path(
        ["arguments"],
        object_field("arguments", ["arguments"]),
        ExpectedFieldShape::required(),
    )
    .build()
}

fn request_outline_argument_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    add_common_argument_fields(FieldDefSet::builder()).build()
}

fn request_read_argument_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    add_common_argument_fields(FieldDefSet::builder())
        .__field_with_declaration_path(
            ["arguments", "ref"],
            non_empty_string_field("arguments.ref", ["arguments", "ref"]),
            ExpectedFieldShape::required(),
        )
        .build()
}

fn request_find_argument_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    add_common_argument_fields(FieldDefSet::builder())
        .__field_with_declaration_path(
            ["arguments", "query"],
            non_empty_string_field("arguments.query", ["arguments", "query"]),
            ExpectedFieldShape::required(),
        )
        .build()
}

fn request_info_argument_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    FieldDefSet::builder()
        .__field_with_declaration_path(
            ["arguments", "options"],
            object_field("arguments.options", ["arguments", "options"]),
            ExpectedFieldShape::optional(),
        )
        .build()
}

fn add_common_argument_fields(
    builder: docnav_typed_fields::__private::FieldDefSetBuilder,
) -> docnav_typed_fields::__private::FieldDefSetBuilder {
    builder
        .__field_with_declaration_path(
            ["arguments", "limit_chars"],
            positive_int_field("arguments.limit_chars", ["arguments", "limit_chars"]),
            ExpectedFieldShape::optional(),
        )
        .__field_with_declaration_path(
            ["arguments", "page"],
            positive_int_field("arguments.page", ["arguments", "page"]),
            ExpectedFieldShape::optional(),
        )
        .__field_with_declaration_path(
            ["arguments", "options"],
            object_field("arguments.options", ["arguments", "options"]),
            ExpectedFieldShape::optional(),
        )
}
