mod catalog;
mod construction;
mod field_helpers;
mod path;
mod pipeline;
mod resolution;
mod source;

pub(crate) use catalog::{
    derive_parameter_catalog, OperationArgumentBinding, ParameterCatalog, ParameterCatalogEntry,
};
pub use catalog::{ParameterCatalogConflictKind, ParameterCatalogError};
#[cfg(test)]
pub(crate) use construction::{construct_config_source, construct_direct_input_source};
pub(crate) use construction::{
    construct_config_source_with_passthrough, construct_default_source,
    construct_direct_input_source_with_passthrough,
};
pub use construction::{
    load_parameter_config_source, ConfigPathOrigin, ConfigSourceLevel, ConfigSourceSkipReason,
    LoadedParameterConfigSource, ParameterConfigSourceDescriptor,
};
pub use field_helpers::{
    adapter_id_field, config_pagination_enabled_field, configurable_limit_field,
    configurable_output_field, document_path_field, find_query_field, ids,
    invocation_log_content_capture_enabled_field, invocation_log_content_capture_root_field,
    invocation_log_enabled_field, invocation_log_path_field, limit_field, page_field,
    pagination_enabled_field, read_ref_field, MAX_PAGINATION_LIMIT,
};
pub use path::{InvalidParameterPath, ParameterPath};
pub use pipeline::{
    ParameterResolutionPipeline, ParameterResolutionPipelineError, ParameterResolutionSourceRole,
};
pub(crate) use resolution::resolve_parameters;
pub use resolution::{
    ParameterConfigSourceIssue, ParameterResolution, ParameterResolutionHandoff,
    ParameterValidationIssue, ResolvedOperationArgumentBinding, ResolvedParameter,
};
pub use source::{
    EntryPassthroughPolicy, ParameterSourceInfo, ParameterSourceKind, PassthroughDisposition,
    PassthroughValue,
};
pub(crate) use source::{ParameterSource, ParameterSources};

#[cfg(test)]
mod tests;
