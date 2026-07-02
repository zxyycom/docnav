use serde_json::{Map, Value};

use docnav_typed_fields::{
    ExpectedFieldShape, FieldBound, FieldDef, FieldDefSet, FieldDefSetBuildError,
    FieldDefSetBuilder, FieldLength, FieldStringEnum, FieldValidation, ProcessStrategy,
};

use crate::Operation;

use super::{JSON_CONTRACT_PROCESSING, VALUE_FIELD};

pub(super) fn add_protocol_version(
    builder: FieldDefSetBuilder,
    identity: &'static str,
) -> FieldDefSetBuilder {
    builder.field_with_declaration_path(
        [identity],
        string_enum_field::<super::enums::ContractVersion>(identity, [identity]),
        ExpectedFieldShape::required(),
    )
}

pub(super) fn add_request_id(builder: FieldDefSetBuilder) -> FieldDefSetBuilder {
    builder.field_with_declaration_path(
        ["request_id"],
        non_empty_string_field("request_id", ["request_id"]),
        ExpectedFieldShape::required(),
    )
}

pub(super) fn add_operation(
    builder: FieldDefSetBuilder,
    shape: ExpectedFieldShape,
) -> FieldDefSetBuilder {
    builder.field_with_declaration_path(
        ["operation"],
        string_enum_field::<Operation>("operation", ["operation"]),
        shape,
    )
}

pub(super) fn string_value_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    value_field_set("string", FieldValidation::string())
}

pub(super) fn value_field_set<T: 'static>(
    identity: &'static str,
    validation: FieldValidation<T>,
) -> Result<FieldDefSet, FieldDefSetBuildError> {
    FieldDefSet::builder()
        .field_with_declaration_path(
            [VALUE_FIELD],
            FieldDef::builder(identity)
                .process(JSON_CONTRACT_PROCESSING, json_path([VALUE_FIELD]))
                .validation(validation),
            ExpectedFieldShape::required(),
        )
        .build()
}

pub(super) fn string_field(
    identity: &str,
    path: impl IntoIterator<Item = &'static str>,
) -> docnav_typed_fields::FieldDefBuilder<String> {
    FieldDef::builder(identity)
        .process(JSON_CONTRACT_PROCESSING, json_path(path))
        .validation(FieldValidation::string())
}

pub(super) fn non_empty_string_field(
    identity: &str,
    path: impl IntoIterator<Item = &'static str>,
) -> docnav_typed_fields::FieldDefBuilder<String> {
    FieldDef::builder(identity)
        .process(JSON_CONTRACT_PROCESSING, json_path(path))
        .validation(FieldValidation::string().length(FieldLength::min(FieldBound::closed(1))))
}

pub(super) fn string_enum_field<T>(
    identity: &str,
    path: impl IntoIterator<Item = &'static str>,
) -> docnav_typed_fields::FieldDefBuilder<T>
where
    T: FieldStringEnum,
{
    FieldDef::builder(identity)
        .process(JSON_CONTRACT_PROCESSING, json_path(path))
        .validation(FieldValidation::string_enum::<T>())
}

pub(super) fn bool_field(
    identity: &str,
    path: impl IntoIterator<Item = &'static str>,
) -> docnav_typed_fields::FieldDefBuilder<bool> {
    FieldDef::builder(identity)
        .process(JSON_CONTRACT_PROCESSING, json_path(path))
        .validation(FieldValidation::boolean())
}

pub(super) fn object_field(
    identity: &str,
    path: impl IntoIterator<Item = &'static str>,
) -> docnav_typed_fields::FieldDefBuilder<Map<String, Value>> {
    FieldDef::builder(identity)
        .process(JSON_CONTRACT_PROCESSING, json_path(path))
        .validation(FieldValidation::object())
}

pub(super) fn positive_int_field(
    identity: &str,
    path: impl IntoIterator<Item = &'static str>,
) -> docnav_typed_fields::FieldDefBuilder<i64> {
    FieldDef::builder(identity)
        .process(JSON_CONTRACT_PROCESSING, json_path(path))
        .validation(FieldValidation::int().min(FieldBound::closed(1)))
}

pub(super) fn non_negative_int_field(
    identity: &str,
    path: impl IntoIterator<Item = &'static str>,
) -> docnav_typed_fields::FieldDefBuilder<i64> {
    FieldDef::builder(identity)
        .process(JSON_CONTRACT_PROCESSING, json_path(path))
        .validation(FieldValidation::int().min(FieldBound::closed(0)))
}

pub(super) fn number_field(
    identity: &str,
    path: impl IntoIterator<Item = &'static str>,
) -> docnav_typed_fields::FieldDefBuilder<f64> {
    FieldDef::builder(identity)
        .process(JSON_CONTRACT_PROCESSING, json_path(path))
        .validation(FieldValidation::num())
}

pub(super) fn non_empty_array_field(
    identity: &str,
    path: impl IntoIterator<Item = &'static str>,
) -> docnav_typed_fields::FieldDefBuilder<Vec<Value>> {
    FieldDef::builder(identity)
        .process(JSON_CONTRACT_PROCESSING, json_path(path))
        .validation(non_empty_array())
}

pub(super) fn non_empty_array() -> FieldValidation<Vec<Value>> {
    FieldValidation::array().length(FieldLength::min(FieldBound::closed(1)))
}

pub(super) fn non_empty_unique_array() -> FieldValidation<Vec<Value>> {
    non_empty_array().unique_items()
}

pub(super) fn json_path<I, S>(segments: I) -> ProcessStrategy
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    ProcessStrategy::json_path(segments)
}
