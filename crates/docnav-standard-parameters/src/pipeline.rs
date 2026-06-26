use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;

use docnav_typed_fields::{ExtractionStrategyId, FieldDefSet, FieldIdentity, JsonValue};

use crate::{
    construct_config_source, construct_default_source, construct_direct_input_source,
    derive_standard_parameter_catalog, load_standard_parameter_config_source,
    resolve_standard_parameters, ConfigPathOrigin, ConfigSourceLevel, EntryPassthroughPolicy,
    LoadedStandardParameterConfigSource, StandardParameterCatalogError,
    StandardParameterConfigSourceDescriptor, StandardParameterDiagnostic,
    StandardParameterResolution, StandardParameterSources,
};

#[derive(Clone)]
pub struct StandardParameterPipeline<'a> {
    fields: &'a FieldDefSet,
    direct_input_strategy: Option<ExtractionStrategyId>,
    config_strategy: Option<ExtractionStrategyId>,
    project_config: Option<PipelineConfigSource>,
    user_config: Option<PipelineConfigSource>,
    dynamic_defaults: BTreeMap<FieldIdentity, JsonValue>,
    passthrough_policy: EntryPassthroughPolicy,
}

impl<'a> StandardParameterPipeline<'a> {
    pub fn new<D>(fields: &'a D) -> Self
    where
        D: AsRef<FieldDefSet> + ?Sized,
    {
        Self {
            fields: fields.as_ref(),
            direct_input_strategy: None,
            config_strategy: None,
            project_config: None,
            user_config: None,
            dynamic_defaults: BTreeMap::new(),
            passthrough_policy: EntryPassthroughPolicy::Retain,
        }
    }

    pub fn with_direct_input_strategy(
        mut self,
        strategy_id: impl Into<ExtractionStrategyId>,
    ) -> Self {
        self.direct_input_strategy = Some(strategy_id.into());
        self
    }

    pub fn with_config_strategy(mut self, strategy_id: impl Into<ExtractionStrategyId>) -> Self {
        self.config_strategy = Some(strategy_id.into());
        self
    }

    pub fn with_project_config_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.project_config = Some(PipelineConfigSource::Descriptor(
            StandardParameterConfigSourceDescriptor::new(
                ConfigSourceLevel::Project,
                ConfigPathOrigin::Override,
                path.into(),
            ),
        ));
        self
    }

    pub fn with_user_config_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.user_config = Some(PipelineConfigSource::Descriptor(
            StandardParameterConfigSourceDescriptor::new(
                ConfigSourceLevel::User,
                ConfigPathOrigin::Override,
                path.into(),
            ),
        ));
        self
    }

    pub fn with_config_source_descriptor(
        mut self,
        descriptor: StandardParameterConfigSourceDescriptor,
    ) -> Self {
        match descriptor.level {
            ConfigSourceLevel::Project => {
                self.project_config = Some(PipelineConfigSource::Descriptor(descriptor));
            }
            ConfigSourceLevel::User => {
                self.user_config = Some(PipelineConfigSource::Descriptor(descriptor));
            }
        }
        self
    }

    pub fn with_loaded_project_config(
        mut self,
        loaded: LoadedStandardParameterConfigSource,
    ) -> Self {
        self.project_config = Some(PipelineConfigSource::Loaded(loaded));
        self
    }

    pub fn with_loaded_user_config(mut self, loaded: LoadedStandardParameterConfigSource) -> Self {
        self.user_config = Some(PipelineConfigSource::Loaded(loaded));
        self
    }

    pub fn with_dynamic_default(mut self, identity: FieldIdentity, value: JsonValue) -> Self {
        self.dynamic_defaults.insert(identity, value);
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

    pub fn resolve(
        self,
        direct_input: impl Into<Option<JsonValue>>,
    ) -> Result<StandardParameterResolution, StandardParameterPipelineError> {
        let catalog = self.catalog()?;
        let entries = catalog.entries();
        let direct_input = direct_input.into();
        let (project_config, mut diagnostics) = config_source_parts(self.project_config);
        let (user_config, user_diagnostics) = config_source_parts(self.user_config);
        diagnostics.extend(user_diagnostics);

        let sources = StandardParameterSources {
            direct_input: construct_direct_input_source(entries, direct_input.as_ref()),
            project_config: construct_config_source(entries, project_config.as_ref()),
            user_config: construct_config_source(entries, user_config.as_ref()),
            default: construct_default_source(entries, &self.dynamic_defaults),
        };
        let mut resolution = resolve_standard_parameters(entries, sources, self.passthrough_policy);
        resolution.extend_diagnostics(diagnostics);
        Ok(resolution)
    }

    fn catalog(&self) -> Result<crate::StandardParameterCatalog, StandardParameterPipelineError> {
        let direct_strategy = self.direct_input_strategy.as_ref().ok_or(
            StandardParameterPipelineError::MissingStrategyRole(
                StandardParameterPipelineSourceRole::DirectInput,
            ),
        )?;
        let config_strategy = self.config_strategy.as_ref().ok_or(
            StandardParameterPipelineError::MissingStrategyRole(
                StandardParameterPipelineSourceRole::Config,
            ),
        )?;

        Ok(derive_standard_parameter_catalog(
            self.fields,
            direct_strategy,
            config_strategy,
        )?)
    }
}

#[derive(Clone, Debug)]
enum PipelineConfigSource {
    Descriptor(StandardParameterConfigSourceDescriptor),
    Loaded(LoadedStandardParameterConfigSource),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardParameterPipelineSourceRole {
    DirectInput,
    Config,
}

impl StandardParameterPipelineSourceRole {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DirectInput => "direct",
            Self::Config => "config",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum StandardParameterPipelineError {
    MissingStrategyRole(StandardParameterPipelineSourceRole),
    Catalog(StandardParameterCatalogError),
}

impl fmt::Display for StandardParameterPipelineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingStrategyRole(role) => {
                write!(
                    formatter,
                    "{} strategy role is not configured",
                    role.as_str()
                )
            }
            Self::Catalog(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for StandardParameterPipelineError {}

impl From<StandardParameterCatalogError> for StandardParameterPipelineError {
    fn from(error: StandardParameterCatalogError) -> Self {
        Self::Catalog(error)
    }
}

fn config_source_parts(
    config: Option<PipelineConfigSource>,
) -> (Option<JsonValue>, Vec<StandardParameterDiagnostic>) {
    match config {
        Some(PipelineConfigSource::Descriptor(descriptor)) => {
            load_standard_parameter_config_source(&descriptor).into_parts()
        }
        Some(PipelineConfigSource::Loaded(loaded)) => loaded.into_parts(),
        None => (None, Vec::new()),
    }
}
