use std::fs;
use std::io;
use std::path::PathBuf;

use docnav_typed_fields::JsonValue;
use serde_json::Value;

use crate::{ParameterConfigSourceIssue, ParameterResolutionHandoff};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigSourceLevel {
    Project,
    User,
}

impl ConfigSourceLevel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::User => "user",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigPathOrigin {
    Default,
    Override,
}

impl ConfigPathOrigin {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Override => "override",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigSourceSkipReason {
    MissingOverride,
    NotFile,
    Unreadable,
    InvalidJson,
    NonObject,
}

impl ConfigSourceSkipReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingOverride => "missing_override",
            Self::NotFile => "not_file",
            Self::Unreadable => "unreadable",
            Self::InvalidJson => "invalid_json",
            Self::NonObject => "non_object",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParameterConfigSourceDescriptor {
    pub level: ConfigSourceLevel,
    pub origin: ConfigPathOrigin,
    pub path: PathBuf,
}

impl ParameterConfigSourceDescriptor {
    pub fn new(level: ConfigSourceLevel, origin: ConfigPathOrigin, path: PathBuf) -> Self {
        Self {
            level,
            origin,
            path,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LoadedParameterConfigSource {
    pub(super) value: Option<JsonValue>,
    pub(super) diagnostics: Vec<ParameterResolutionHandoff>,
}

impl LoadedParameterConfigSource {
    pub fn from_value(value: JsonValue) -> Self {
        Self {
            value: Some(value),
            diagnostics: Vec::new(),
        }
    }

    pub fn value(&self) -> Option<&JsonValue> {
        self.value.as_ref()
    }

    pub fn diagnostics(&self) -> &[ParameterResolutionHandoff] {
        &self.diagnostics
    }

    pub fn with_config_source_issue(mut self, issue: ParameterConfigSourceIssue) -> Self {
        self.diagnostics
            .push(ParameterResolutionHandoff::config_source(issue));
        self
    }

    pub(crate) fn into_parts(self) -> (Option<JsonValue>, Vec<ParameterResolutionHandoff>) {
        (self.value, self.diagnostics)
    }
}

pub fn load_parameter_config_source(
    descriptor: &ParameterConfigSourceDescriptor,
) -> LoadedParameterConfigSource {
    match fs::metadata(&descriptor.path) {
        Ok(metadata) if !metadata.is_file() => skipped(descriptor, ConfigSourceSkipReason::NotFile),
        Ok(_) => read_config_source(descriptor),
        Err(error) if error.kind() == io::ErrorKind::NotFound => missing_source(descriptor),
        Err(_) => skipped(descriptor, ConfigSourceSkipReason::Unreadable),
    }
}

fn read_config_source(descriptor: &ParameterConfigSourceDescriptor) -> LoadedParameterConfigSource {
    let content = match fs::read_to_string(&descriptor.path) {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return missing_source(descriptor),
        Err(_) => return skipped(descriptor, ConfigSourceSkipReason::Unreadable),
    };
    let value = match serde_json::from_str::<Value>(&content) {
        Ok(value) => value,
        Err(_) => return skipped(descriptor, ConfigSourceSkipReason::InvalidJson),
    };
    if !value.is_object() {
        return skipped(descriptor, ConfigSourceSkipReason::NonObject);
    }
    LoadedParameterConfigSource::from_value(value)
}

fn missing_source(descriptor: &ParameterConfigSourceDescriptor) -> LoadedParameterConfigSource {
    match descriptor.origin {
        ConfigPathOrigin::Default => LoadedParameterConfigSource::default(),
        ConfigPathOrigin::Override => skipped(descriptor, ConfigSourceSkipReason::MissingOverride),
    }
}

fn skipped(
    descriptor: &ParameterConfigSourceDescriptor,
    reason: ConfigSourceSkipReason,
) -> LoadedParameterConfigSource {
    let issue = ParameterConfigSourceIssue::new(
        descriptor.level.as_str(),
        descriptor.origin.as_str(),
        descriptor.path.display().to_string(),
        reason.as_str(),
    );
    LoadedParameterConfigSource {
        value: None,
        diagnostics: vec![ParameterResolutionHandoff::config_source(issue)],
    }
}
