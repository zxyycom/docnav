use std::fs;
use std::io;
use std::path::PathBuf;

use docnav_typed_fields::JsonValue;
use serde_json::Value;

use crate::{StandardParameterConfigSourceIssue, StandardParameterHandoff};

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
pub struct StandardParameterConfigSourceDescriptor {
    pub level: ConfigSourceLevel,
    pub origin: ConfigPathOrigin,
    pub path: PathBuf,
}

impl StandardParameterConfigSourceDescriptor {
    pub fn new(level: ConfigSourceLevel, origin: ConfigPathOrigin, path: PathBuf) -> Self {
        Self {
            level,
            origin,
            path,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LoadedStandardParameterConfigSource {
    pub(super) value: Option<JsonValue>,
    pub(super) diagnostics: Vec<StandardParameterHandoff>,
}

impl LoadedStandardParameterConfigSource {
    pub fn from_value(value: JsonValue) -> Self {
        Self {
            value: Some(value),
            diagnostics: Vec::new(),
        }
    }

    pub fn value(&self) -> Option<&JsonValue> {
        self.value.as_ref()
    }

    pub fn diagnostics(&self) -> &[StandardParameterHandoff] {
        &self.diagnostics
    }

    pub(crate) fn into_parts(self) -> (Option<JsonValue>, Vec<StandardParameterHandoff>) {
        (self.value, self.diagnostics)
    }
}

pub fn load_standard_parameter_config_source(
    descriptor: &StandardParameterConfigSourceDescriptor,
) -> LoadedStandardParameterConfigSource {
    match fs::metadata(&descriptor.path) {
        Ok(metadata) if !metadata.is_file() => skipped(descriptor, ConfigSourceSkipReason::NotFile),
        Ok(_) => read_config_source(descriptor),
        Err(error) if error.kind() == io::ErrorKind::NotFound => missing_source(descriptor),
        Err(_) => skipped(descriptor, ConfigSourceSkipReason::Unreadable),
    }
}

fn read_config_source(
    descriptor: &StandardParameterConfigSourceDescriptor,
) -> LoadedStandardParameterConfigSource {
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
    LoadedStandardParameterConfigSource::from_value(value)
}

fn missing_source(
    descriptor: &StandardParameterConfigSourceDescriptor,
) -> LoadedStandardParameterConfigSource {
    match descriptor.origin {
        ConfigPathOrigin::Default => LoadedStandardParameterConfigSource::default(),
        ConfigPathOrigin::Override => skipped(descriptor, ConfigSourceSkipReason::MissingOverride),
    }
}

fn skipped(
    descriptor: &StandardParameterConfigSourceDescriptor,
    reason: ConfigSourceSkipReason,
) -> LoadedStandardParameterConfigSource {
    let issue = StandardParameterConfigSourceIssue::new(
        descriptor.level.as_str(),
        descriptor.origin.as_str(),
        descriptor.path.display().to_string(),
        reason.as_str(),
    );
    LoadedStandardParameterConfigSource {
        value: None,
        diagnostics: vec![StandardParameterHandoff::config_source(issue)],
    }
}
