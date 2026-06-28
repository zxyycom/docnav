use docnav_typed_fields::{
    ExpectedFieldShape, FieldDef, FieldDefSet, FieldDefSetBuildError, FieldValidation,
};

use super::enums::ProtocolErrorCode;
use super::field_builders::{
    add_operation, add_protocol_version, add_request_id, bool_field, json_path,
    non_empty_string_field, non_empty_unique_array, object_field, positive_int_field,
    string_enum_field, string_field,
};

pub(super) fn response_common_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    add_request_id(add_protocol_version(
        FieldDefSet::builder(),
        "protocol_version",
    ))
    .__field_with_declaration_path(
        ["ok"],
        bool_field("ok", ["ok"]),
        ExpectedFieldShape::required(),
    )
    .build()
}

pub(super) fn response_success_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    add_operation(response_common_builder(), ExpectedFieldShape::required())
        .__field_with_declaration_path(
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
    .__field_with_declaration_path(
        ["error"],
        object_field("error", ["error"]),
        ExpectedFieldShape::required(),
    )
    .__field_with_declaration_path(
        ["error", "code"],
        string_enum_field::<ProtocolErrorCode>("error.code", ["error", "code"]),
        ExpectedFieldShape::required(),
    )
    .__field_with_declaration_path(
        ["error", "message"],
        string_field("error.message", ["error", "message"]),
        ExpectedFieldShape::required(),
    )
    .__field_with_declaration_path(
        ["error", "details"],
        object_field("error.details", ["error", "details"]),
        ExpectedFieldShape::required(),
    )
    .__field_with_declaration_path(
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

fn response_common_builder() -> docnav_typed_fields::__private::FieldDefSetBuilder {
    add_request_id(add_protocol_version(
        FieldDefSet::builder(),
        "protocol_version",
    ))
    .__field_with_declaration_path(
        ["ok"],
        bool_field("ok", ["ok"]),
        ExpectedFieldShape::required(),
    )
}

pub(super) fn response_outline_result_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    FieldDefSet::builder()
        .__field_with_declaration_path(
            ["result", "entries"],
            FieldDef::builder("result.entries")
                .process(
                    super::JSON_CONTRACT_PROCESSING,
                    json_path(["result", "entries"]),
                )
                .validation(FieldValidation::array()),
            ExpectedFieldShape::required(),
        )
        .__field_with_declaration_path(
            ["result", "page"],
            positive_int_field("result.page", ["result", "page"]),
            ExpectedFieldShape::required_nullable(),
        )
        .build()
}

pub(super) fn response_read_result_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    FieldDefSet::builder()
        .__field_with_declaration_path(
            ["result", "ref"],
            non_empty_string_field("result.ref", ["result", "ref"]),
            ExpectedFieldShape::required(),
        )
        .__field_with_declaration_path(
            ["result", "content"],
            string_field("result.content", ["result", "content"]),
            ExpectedFieldShape::required(),
        )
        .__field_with_declaration_path(
            ["result", "content_type"],
            non_empty_string_field("result.content_type", ["result", "content_type"]),
            ExpectedFieldShape::required(),
        )
        .__field_with_declaration_path(
            ["result", "cost"],
            non_empty_string_field("result.cost", ["result", "cost"]),
            ExpectedFieldShape::required(),
        )
        .__field_with_declaration_path(
            ["result", "page"],
            positive_int_field("result.page", ["result", "page"]),
            ExpectedFieldShape::required_nullable(),
        )
        .build()
}

pub(super) fn response_find_result_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    FieldDefSet::builder()
        .__field_with_declaration_path(
            ["result", "matches"],
            FieldDef::builder("result.matches")
                .process(
                    super::JSON_CONTRACT_PROCESSING,
                    json_path(["result", "matches"]),
                )
                .validation(FieldValidation::array()),
            ExpectedFieldShape::required(),
        )
        .__field_with_declaration_path(
            ["result", "page"],
            positive_int_field("result.page", ["result", "page"]),
            ExpectedFieldShape::required_nullable(),
        )
        .build()
}

pub(super) fn response_info_result_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    FieldDefSet::builder()
        .__field_with_declaration_path(
            ["result", "display"],
            non_empty_string_field("result.display", ["result", "display"]),
            ExpectedFieldShape::required(),
        )
        .__field_with_declaration_path(
            ["result", "capabilities"],
            FieldDef::builder("result.capabilities")
                .process(
                    super::JSON_CONTRACT_PROCESSING,
                    json_path(["result", "capabilities"]),
                )
                .validation(non_empty_unique_array()),
            ExpectedFieldShape::required(),
        )
        .build()
}

pub(super) fn response_entry_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    FieldDefSet::builder()
        .__field_with_declaration_path(
            ["ref"],
            non_empty_string_field("entry.ref", ["ref"]),
            ExpectedFieldShape::required(),
        )
        .__field_with_declaration_path(
            ["display"],
            non_empty_string_field("entry.display", ["display"]),
            ExpectedFieldShape::required(),
        )
        .build()
}
