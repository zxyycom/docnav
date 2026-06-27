use crate::details::{DetailFieldRule, DetailFieldType as DetailKind};

const fn required(name: &'static str, kind: DetailKind) -> DetailFieldRule {
    DetailFieldRule::required(name, kind)
}

const fn optional(name: &'static str, kind: DetailKind) -> DetailFieldRule {
    DetailFieldRule::optional(name, kind)
}

pub(super) const FIELD_REASON_FIELDS: &[DetailFieldRule] = &[
    required("field", DetailKind::String),
    required("reason", DetailKind::String),
    optional("path", DetailKind::String),
    optional("received", DetailKind::String),
    optional("accepted", DetailKind::StringArray),
];
pub(super) const PATH_FIELDS: &[DetailFieldRule] = &[required("path", DetailKind::String)];
pub(super) const PATH_REASON_FIELDS: &[DetailFieldRule] = &[
    required("path", DetailKind::String),
    required("reason", DetailKind::String),
];
pub(super) const PATH_ENCODING_FIELDS: &[DetailFieldRule] = &[
    required("path", DetailKind::String),
    required("encoding", DetailKind::String),
];
pub(super) const FORMAT_UNKNOWN_FIELDS: &[DetailFieldRule] = &[
    required("path", DetailKind::String),
    required("reason", DetailKind::String),
    required("candidates", DetailKind::Any),
];
pub(super) const FORMAT_AMBIGUOUS_FIELDS: &[DetailFieldRule] = &[
    required("path", DetailKind::String),
    required("candidates", DetailKind::Any),
];
pub(super) const CAPABILITY_ADAPTER_FIELDS: &[DetailFieldRule] = &[
    required("capability", DetailKind::String),
    required("adapter_id", DetailKind::String),
];
pub(super) const REF_FIELDS: &[DetailFieldRule] = &[required("ref", DetailKind::String)];
pub(super) const REF_CANDIDATE_FIELDS: &[DetailFieldRule] = &[
    required("ref", DetailKind::String),
    required("candidate_count", DetailKind::U32),
];
pub(super) const REF_REASON_FIELDS: &[DetailFieldRule] = &[
    required("ref", DetailKind::String),
    required("reason", DetailKind::String),
];
pub(super) const ADAPTER_REASON_FIELDS: &[DetailFieldRule] = &[
    required("adapter_id", DetailKind::String),
    required("reason", DetailKind::String),
    optional("exit_code", DetailKind::I32),
    optional("stderr", DetailKind::String),
];
pub(super) const INTERNAL_FIELDS: &[DetailFieldRule] = &[required("error_id", DetailKind::String)];
pub(super) const CLI_ARGV_FIELDS: &[DetailFieldRule] =
    &[required("tokens", DetailKind::StringArray)];
pub(super) const ADAPTER_CANDIDATE_FIELDS: &[DetailFieldRule] = &[
    required("adapter_id", DetailKind::String),
    required("stage", DetailKind::String),
    required("code", DetailKind::String),
    optional("preselected", DetailKind::Boolean),
];
pub(super) const ADAPTER_CONFIG_FIELDS: &[DetailFieldRule] = &[
    required("source_level", DetailKind::String),
    required("path_origin", DetailKind::String),
    required("path", DetailKind::String),
    required("reason_code", DetailKind::String),
];
pub(super) const BOUNDARY_FIELDS: &[DetailFieldRule] = &[
    required("reason", DetailKind::String),
    optional("label", DetailKind::String),
];
