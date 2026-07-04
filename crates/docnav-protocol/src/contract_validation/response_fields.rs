use docnav_typed_fields::{
    ExpectedFieldShape, FieldDef, FieldDefSet, FieldDefSetBuildError, FieldDefSetBuilder,
    FieldValidation,
};

use super::enums::{OutlineResultKind, ProtocolErrorCode};
use super::field_builders::{
    add_operation, add_protocol_version, add_request_id, bool_field, json_path,
    non_empty_string_field, non_empty_unique_array, non_negative_int_field, number_field,
    object_field, positive_int_field, string_enum_field, string_field,
};
use crate::UnstructuredOutlineReason;

pub(super) fn response_common_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    add_request_id(add_protocol_version(
        FieldDefSet::builder(),
        "protocol_version",
    ))
    .field_with_declaration_path(
        ["ok"],
        bool_field("ok", ["ok"]),
        ExpectedFieldShape::required(),
    )
    .build()
}

pub(super) fn response_success_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    add_operation(response_common_builder(), ExpectedFieldShape::required())
        .field_with_declaration_path(
            ["result"],
            object_field("result", ["result"]),
            ExpectedFieldShape::required(),
        )
        .build()
}

pub(super) fn response_failure_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    add_operation(
        response_common_builder(),
        ExpectedFieldShape::required_nullable(),
    )
    .field_with_declaration_path(
        ["error"],
        object_field("error", ["error"]),
        ExpectedFieldShape::required(),
    )
    .field_with_declaration_path(
        ["error", "code"],
        string_enum_field::<ProtocolErrorCode>("error.code", ["error", "code"]),
        ExpectedFieldShape::required(),
    )
    .field_with_declaration_path(
        ["error", "message"],
        string_field("error.message", ["error", "message"]),
        ExpectedFieldShape::required(),
    )
    .field_with_declaration_path(
        ["error", "owner"],
        non_empty_string_field("error.owner", ["error", "owner"]),
        ExpectedFieldShape::required(),
    )
    .field_with_declaration_path(
        ["error", "location"],
        object_field("error.location", ["error", "location"]),
        ExpectedFieldShape::optional(),
    )
    .field_with_declaration_path(
        ["error", "details"],
        object_field("error.details", ["error", "details"]),
        ExpectedFieldShape::required(),
    )
    .field_with_declaration_path(
        ["error", "guidance"],
        FieldDef::builder("error.guidance")
            .process(
                super::JSON_CONTRACT_PROCESSING,
                json_path(["error", "guidance"]),
            )
            .validation(FieldValidation::array()),
        ExpectedFieldShape::optional(),
    )
    .build()
}

pub(super) fn response_unknown_shape_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    add_operation(response_common_builder(), ExpectedFieldShape::optional())
        .field_with_declaration_path(
            ["result"],
            object_field("result", ["result"]),
            ExpectedFieldShape::optional(),
        )
        .field_with_declaration_path(
            ["error"],
            object_field("error", ["error"]),
            ExpectedFieldShape::optional(),
        )
        .build()
}

fn response_common_builder() -> FieldDefSetBuilder {
    add_request_id(add_protocol_version(
        FieldDefSet::builder(),
        "protocol_version",
    ))
    .field_with_declaration_path(
        ["ok"],
        bool_field("ok", ["ok"]),
        ExpectedFieldShape::required(),
    )
}

pub(super) fn response_structured_outline_result_fields(
) -> Result<FieldDefSet, FieldDefSetBuildError> {
    FieldDefSet::builder()
        .field_with_declaration_path(
            ["result", "kind"],
            string_enum_field::<OutlineResultKind>("result.kind", ["result", "kind"]),
            ExpectedFieldShape::required(),
        )
        .field_with_declaration_path(
            ["result", "entries"],
            FieldDef::builder("result.entries")
                .process(
                    super::JSON_CONTRACT_PROCESSING,
                    json_path(["result", "entries"]),
                )
                .validation(FieldValidation::array()),
            ExpectedFieldShape::required(),
        )
        .field_with_declaration_path(
            ["result", "page"],
            positive_int_field("result.page", ["result", "page"]),
            ExpectedFieldShape::required_nullable(),
        )
        .build()
}

pub(super) fn response_unstructured_outline_result_fields(
) -> Result<FieldDefSet, FieldDefSetBuildError> {
    add_result_content_fields(
        FieldDefSet::builder()
            .field_with_declaration_path(
                ["result", "kind"],
                string_enum_field::<OutlineResultKind>("result.kind", ["result", "kind"]),
                ExpectedFieldShape::required(),
            )
            .field_with_declaration_path(
                ["result", "reason"],
                string_enum_field::<UnstructuredOutlineReason>(
                    "result.reason",
                    ["result", "reason"],
                ),
                ExpectedFieldShape::required(),
            ),
    )
    .build()
}

pub(super) fn response_read_result_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    add_result_content_fields(FieldDefSet::builder().field_with_declaration_path(
        ["result", "ref"],
        non_empty_string_field("result.ref", ["result", "ref"]),
        ExpectedFieldShape::required(),
    ))
    .field_with_declaration_path(
        ["result", "page"],
        positive_int_field("result.page", ["result", "page"]),
        ExpectedFieldShape::required_nullable(),
    )
    .build()
}

fn add_result_content_fields(builder: FieldDefSetBuilder) -> FieldDefSetBuilder {
    builder
        .field_with_declaration_path(
            ["result", "content"],
            string_field("result.content", ["result", "content"]),
            ExpectedFieldShape::required(),
        )
        .field_with_declaration_path(
            ["result", "content_type"],
            non_empty_string_field("result.content_type", ["result", "content_type"]),
            ExpectedFieldShape::required(),
        )
        .field_with_declaration_path(
            ["result", "cost"],
            object_field("result.cost", ["result", "cost"]),
            ExpectedFieldShape::required(),
        )
}

pub(super) fn response_find_result_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    FieldDefSet::builder()
        .field_with_declaration_path(
            ["result", "matches"],
            FieldDef::builder("result.matches")
                .process(
                    super::JSON_CONTRACT_PROCESSING,
                    json_path(["result", "matches"]),
                )
                .validation(FieldValidation::array()),
            ExpectedFieldShape::required(),
        )
        .field_with_declaration_path(
            ["result", "page"],
            positive_int_field("result.page", ["result", "page"]),
            ExpectedFieldShape::required_nullable(),
        )
        .build()
}

pub(super) fn response_info_result_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    FieldDefSet::builder()
        .field_with_declaration_path(
            ["result", "document"],
            object_field("result.document", ["result", "document"]),
            ExpectedFieldShape::optional(),
        )
        .field_with_declaration_path(
            ["result", "adapter"],
            object_field("result.adapter", ["result", "adapter"]),
            ExpectedFieldShape::optional(),
        )
        .field_with_declaration_path(
            ["result", "metadata"],
            object_field("result.metadata", ["result", "metadata"]),
            ExpectedFieldShape::optional(),
        )
        .build()
}

pub(super) fn response_entry_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    FieldDefSet::builder()
        .field_with_declaration_path(
            ["ref"],
            non_empty_string_field("entry.ref", ["ref"]),
            ExpectedFieldShape::required(),
        )
        .field_with_declaration_path(
            ["label"],
            non_empty_string_field("entry.label", ["label"]),
            ExpectedFieldShape::required(),
        )
        .field_with_declaration_path(
            ["kind"],
            non_empty_string_field("entry.kind", ["kind"]),
            ExpectedFieldShape::optional(),
        )
        .field_with_declaration_path(
            ["location"],
            object_field("entry.location", ["location"]),
            ExpectedFieldShape::optional(),
        )
        .field_with_declaration_path(
            ["summary"],
            string_field("entry.summary", ["summary"]),
            ExpectedFieldShape::optional(),
        )
        .field_with_declaration_path(
            ["excerpt"],
            string_field("entry.excerpt", ["excerpt"]),
            ExpectedFieldShape::optional(),
        )
        .field_with_declaration_path(
            ["rank"],
            number_field("entry.rank", ["rank"]),
            ExpectedFieldShape::optional(),
        )
        .field_with_declaration_path(
            ["cost"],
            object_field("entry.cost", ["cost"]),
            ExpectedFieldShape::optional(),
        )
        .field_with_declaration_path(
            ["metadata"],
            object_field("entry.metadata", ["metadata"]),
            ExpectedFieldShape::optional(),
        )
        .build()
}

pub(super) fn response_location_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    FieldDefSet::builder()
        .field_with_declaration_path(
            ["line_start"],
            positive_int_field("location.line_start", ["line_start"]),
            ExpectedFieldShape::required(),
        )
        .field_with_declaration_path(
            ["line_end"],
            positive_int_field("location.line_end", ["line_end"]),
            ExpectedFieldShape::optional(),
        )
        .build()
}

pub(super) fn response_cost_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    FieldDefSet::builder()
        .field_with_declaration_path(
            ["measurements"],
            FieldDef::builder("cost.measurements")
                .process(super::JSON_CONTRACT_PROCESSING, json_path(["measurements"]))
                .validation(non_empty_unique_array()),
            ExpectedFieldShape::required(),
        )
        .build()
}

pub(super) fn response_cost_fields_allow_empty() -> Result<FieldDefSet, FieldDefSetBuildError> {
    FieldDefSet::builder()
        .field_with_declaration_path(
            ["measurements"],
            FieldDef::builder("cost.measurements")
                .process(super::JSON_CONTRACT_PROCESSING, json_path(["measurements"]))
                .validation(FieldValidation::array().unique_items()),
            ExpectedFieldShape::required(),
        )
        .build()
}

pub(super) fn response_measurement_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    FieldDefSet::builder()
        .field_with_declaration_path(
            ["unit"],
            non_empty_string_field("measurement.unit", ["unit"]),
            ExpectedFieldShape::required(),
        )
        .field_with_declaration_path(
            ["value"],
            non_negative_int_field("measurement.value", ["value"]),
            ExpectedFieldShape::required(),
        )
        .field_with_declaration_path(
            ["scope"],
            non_empty_string_field("measurement.scope", ["scope"]),
            ExpectedFieldShape::optional(),
        )
        .build()
}

pub(super) fn response_info_document_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    FieldDefSet::builder()
        .field_with_declaration_path(
            ["content_type"],
            non_empty_string_field("info.document.content_type", ["content_type"]),
            ExpectedFieldShape::optional(),
        )
        .field_with_declaration_path(
            ["encoding"],
            non_empty_string_field("info.document.encoding", ["encoding"]),
            ExpectedFieldShape::optional(),
        )
        .field_with_declaration_path(
            ["size"],
            object_field("info.document.size", ["size"]),
            ExpectedFieldShape::optional(),
        )
        .build()
}

pub(super) fn response_info_adapter_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    FieldDefSet::builder()
        .field_with_declaration_path(
            ["id"],
            non_empty_string_field("info.adapter.id", ["id"]),
            ExpectedFieldShape::optional(),
        )
        .field_with_declaration_path(
            ["format"],
            non_empty_string_field("info.adapter.format", ["format"]),
            ExpectedFieldShape::optional(),
        )
        .build()
}
