#![allow(dead_code)]

use super::*;

mod constraints;
mod field_model;
mod field_presence;
mod field_ranges;
mod set_projection;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputMode {
    ReadableView,
    ReadableJson,
    ProtocolJson,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EmptyMode {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DuplicateMode {
    ReadableView,
    ReadableViewAlias,
    ReadableJson,
    ProtocolJson,
}

impl FieldStringEnum for OutputMode {
    fn variants() -> &'static [Self] {
        &[Self::ReadableView, Self::ReadableJson, Self::ProtocolJson]
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::ReadableView => "readable-view",
            Self::ReadableJson => "readable-json",
            Self::ProtocolJson => "protocol-json",
        }
    }
}

impl FieldStringEnum for EmptyMode {
    fn variants() -> &'static [Self] {
        &[]
    }

    fn as_str(&self) -> &'static str {
        match *self {}
    }
}

impl FieldStringEnum for DuplicateMode {
    fn variants() -> &'static [Self] {
        &[
            Self::ReadableView,
            Self::ReadableViewAlias,
            Self::ReadableJson,
            Self::ProtocolJson,
        ]
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::ReadableView | Self::ReadableViewAlias => "readable-view",
            Self::ReadableJson => "readable-json",
            Self::ProtocolJson => "protocol-json",
        }
    }
}

fn limit_chars_validation() -> FieldValidation<i64> {
    FieldValidation::int().between(FieldBound::closed(1), FieldBound::closed(100_000))
}

fn output_mode_validation() -> FieldValidation<OutputMode> {
    FieldValidation::string_enum::<OutputMode>()
}

#[derive(Debug, FieldDefs)]
pub(crate) struct DocnavParams {
    #[field(group)]
    defaults: DefaultsParams,
}

#[derive(Debug, FieldDefs)]
pub(crate) struct DefaultsParams {
    #[field(
        FieldDef::builder("docnav.defaults.limit_chars")
            .path(["a", "b"])
            .validation(limit_chars_validation())
            .default_static(20_000)
    )]
    limit_chars: Option<i64>,

    #[field(
        FieldDef::builder("docnav.defaults.output")
            .path(["defaults", "output"])
            .validation(output_mode_validation())
            .default_static(OutputMode::ReadableView)
    )]
    output: OutputMode,
}
