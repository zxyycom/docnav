use serde_json::Value;

use docnav_typed_fields::{
    ExpectedFieldShape, FieldBound, FieldDef, FieldDefSet, FieldDefSetBuildError, FieldValidation,
};

use crate::constants::schema_names;
use crate::{ProbeReasonCode, SchemaValidationError};

use super::field_builders::{
    bool_field, json_path, non_empty_array_field, string_enum_field, string_field,
};
use super::helpers::{
    reject_unknown_fields, schema_result, validate_field_set, validate_object_array_items,
    ObjectArraySpec,
};

pub(crate) fn validate_probe_result_contract_value(
    value: &Value,
) -> Result<(), SchemaValidationError> {
    let mut errors = Vec::new();
    validate_field_set(
        schema_names::PROBE_RESULT,
        probe_fields,
        value,
        &[],
        &mut errors,
    );
    reject_unknown_fields(
        Some(value),
        &[],
        &[
            "probe_version",
            "adapter_id",
            "path",
            "supported",
            "format",
            "confidence",
            "reasons",
        ],
        &mut errors,
    );
    validate_object_array_items(
        value,
        &["reasons"],
        ObjectArraySpec {
            schema: schema_names::PROBE_RESULT,
            build: probe_reason_fields,
            allowed_fields: &["code", "detail"],
        },
        |_, _, _| {},
        &mut errors,
    );
    schema_result(schema_names::PROBE_RESULT, errors)
}

fn probe_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    FieldDefSet::builder()
        .__field_with_declaration_path(
            ["probe_version"],
            string_enum_field::<super::enums::ContractVersion>("probe_version", ["probe_version"]),
            ExpectedFieldShape::required(),
        )
        .__field_with_declaration_path(
            ["adapter_id"],
            string_field("adapter_id", ["adapter_id"]),
            ExpectedFieldShape::required(),
        )
        .__field_with_declaration_path(
            ["path"],
            string_field("path", ["path"]),
            ExpectedFieldShape::required(),
        )
        .__field_with_declaration_path(
            ["supported"],
            bool_field("supported", ["supported"]),
            ExpectedFieldShape::required(),
        )
        .__field_with_declaration_path(
            ["format"],
            string_field("format", ["format"]),
            ExpectedFieldShape::required_nullable(),
        )
        .__field_with_declaration_path(
            ["confidence"],
            FieldDef::builder("confidence")
                .process(super::JSON_CONTRACT_PROCESSING, json_path(["confidence"]))
                .validation(
                    FieldValidation::num()
                        .between(FieldBound::closed(0.0), FieldBound::closed(1.0)),
                ),
            ExpectedFieldShape::required(),
        )
        .__field_with_declaration_path(
            ["reasons"],
            non_empty_array_field("reasons", ["reasons"]),
            ExpectedFieldShape::required(),
        )
        .build()
}

fn probe_reason_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    FieldDefSet::builder()
        .__field_with_declaration_path(
            ["code"],
            string_enum_field::<ProbeReasonCode>("reasons[].code", ["code"]),
            ExpectedFieldShape::required(),
        )
        .__field_with_declaration_path(
            ["detail"],
            string_field("reasons[].detail", ["detail"]),
            ExpectedFieldShape::required(),
        )
        .build()
}
