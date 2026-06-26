use std::collections::BTreeMap;

use docnav_typed_fields::{FieldIdentity, JsonValue};

use crate::{
    resolve_standard_parameters, EntryPassthroughPolicy, StandardParameterDiagnostic,
    StandardParameterRegistration, StandardParameterSources,
};

mod config;
mod source;

pub use config::{
    load_standard_parameter_config_source, ConfigPathOrigin, ConfigSourceLevel,
    ConfigSourceSkipReason, LoadedStandardParameterConfigSource,
    StandardParameterConfigSourceDescriptor,
};
pub use source::{
    construct_config_source, construct_default_source, construct_direct_input_source,
};

#[derive(Clone, Debug)]
pub struct StandardParameterResolutionInputs<'a> {
    registrations: &'a [StandardParameterRegistration],
    direct_input: Option<JsonValue>,
    project_config: Option<JsonValue>,
    user_config: Option<JsonValue>,
    dynamic_defaults: BTreeMap<FieldIdentity, JsonValue>,
    passthrough_policy: EntryPassthroughPolicy,
    diagnostics: Vec<StandardParameterDiagnostic>,
}

impl<'a> StandardParameterResolutionInputs<'a> {
    pub fn new(registrations: &'a [StandardParameterRegistration]) -> Self {
        Self {
            registrations,
            direct_input: None,
            project_config: None,
            user_config: None,
            dynamic_defaults: BTreeMap::new(),
            passthrough_policy: EntryPassthroughPolicy::Retain,
            diagnostics: Vec::new(),
        }
    }

    pub fn with_direct_input(mut self, input: JsonValue) -> Self {
        self.direct_input = Some(input);
        self
    }

    pub fn with_project_config(mut self, config: JsonValue) -> Self {
        self.project_config = Some(config);
        self
    }

    pub fn with_user_config(mut self, config: JsonValue) -> Self {
        self.user_config = Some(config);
        self
    }

    pub fn with_loaded_project_config(
        mut self,
        loaded: LoadedStandardParameterConfigSource,
    ) -> Self {
        if let Some(value) = loaded.value {
            self.project_config = Some(value);
        }
        self.diagnostics.extend(loaded.diagnostics);
        self
    }

    pub fn with_loaded_user_config(mut self, loaded: LoadedStandardParameterConfigSource) -> Self {
        if let Some(value) = loaded.value {
            self.user_config = Some(value);
        }
        self.diagnostics.extend(loaded.diagnostics);
        self
    }

    pub fn with_dynamic_defaults(
        mut self,
        dynamic_defaults: BTreeMap<FieldIdentity, JsonValue>,
    ) -> Self {
        self.dynamic_defaults = dynamic_defaults;
        self
    }

    pub fn with_passthrough_policy(mut self, policy: EntryPassthroughPolicy) -> Self {
        self.passthrough_policy = policy;
        self
    }
}

pub fn resolve_standard_parameter_inputs(
    inputs: StandardParameterResolutionInputs<'_>,
) -> crate::StandardParameterResolution {
    let sources = StandardParameterSources {
        direct_input: construct_direct_input_source(
            inputs.registrations,
            inputs.direct_input.as_ref(),
        ),
        project_config: construct_config_source(
            inputs.registrations,
            inputs.project_config.as_ref(),
        ),
        user_config: construct_config_source(inputs.registrations, inputs.user_config.as_ref()),
        default: construct_default_source(inputs.registrations, &inputs.dynamic_defaults),
    };
    let mut resolution =
        resolve_standard_parameters(inputs.registrations, sources, inputs.passthrough_policy);
    resolution.extend_diagnostics(inputs.diagnostics);
    resolution
}
