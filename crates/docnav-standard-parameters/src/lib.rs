mod catalog;
mod construction;
mod path;
mod pipeline;
mod resolution;
mod source;

pub(crate) use catalog::{
    derive_standard_parameter_catalog, OperationArgumentBinding, StandardParameterCatalog,
    StandardParameterCatalogEntry,
};
pub use catalog::{StandardParameterCatalogConflictKind, StandardParameterCatalogError};
#[cfg(test)]
pub(crate) use construction::{construct_config_source, construct_direct_input_source};
pub(crate) use construction::{
    construct_config_source_with_passthrough, construct_default_source,
    construct_direct_input_source_with_passthrough,
};
pub use construction::{
    load_standard_parameter_config_source, ConfigPathOrigin, ConfigSourceLevel,
    ConfigSourceSkipReason, LoadedStandardParameterConfigSource,
    StandardParameterConfigSourceDescriptor,
};
pub use path::{InvalidStandardParameterPath, StandardParameterPath};
pub use pipeline::{
    StandardParameterPipeline, StandardParameterPipelineError, StandardParameterPipelineSourceRole,
};
pub(crate) use resolution::resolve_standard_parameters;
pub use resolution::{
    ResolvedOperationArgumentBinding, ResolvedStandardParameter, StandardParameterDiagnostic,
    StandardParameterResolution, StandardParameterValidationDiagnostic,
};
pub use source::{
    EntryPassthroughPolicy, PassthroughDisposition, PassthroughValue, StandardParameterSourceInfo,
    StandardParameterSourceKind,
};
pub(crate) use source::{StandardParameterSource, StandardParameterSources};

#[cfg(test)]
mod tests;
