mod construction;
mod path;
mod registration;
mod resolution;
mod source;

pub use construction::{
    construct_config_source, construct_default_source, construct_direct_input_source,
    load_standard_parameter_config_source, resolve_standard_parameter_inputs, ConfigPathOrigin,
    ConfigSourceLevel, ConfigSourceSkipReason, LoadedStandardParameterConfigSource,
    StandardParameterConfigSourceDescriptor, StandardParameterResolutionInputs,
};
pub use path::{InvalidStandardParameterPath, StandardParameterPath};
pub use registration::{
    OperationArgumentBinding, StandardParameterBinding, StandardParameterRegistration,
    StandardParameterRegistrationConflictKind, StandardParameterRegistrationSet,
    StandardParameterRegistrationSetError,
};
pub use resolution::{
    resolve_standard_parameters, ResolvedOperationArgumentBinding, ResolvedStandardParameter,
    StandardParameterDiagnostic, StandardParameterResolution,
    StandardParameterValidationDiagnostic,
};
pub use source::{
    EntryPassthroughPolicy, PassthroughDisposition, PassthroughInput, PassthroughValue,
    StandardParameterSource, StandardParameterSourceInfo, StandardParameterSourceKind,
    StandardParameterSources,
};

#[cfg(test)]
mod tests;
