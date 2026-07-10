use std::fs;
use std::io;
use std::path::Path;

use docnav_diagnostics::{
    typed_codes, AdapterConfigSourceDetails, DiagnosticRecordDraft, DiagnosticSource,
    FieldReasonDetails,
};
use serde_json::Value;

use crate::NavigationConfigSourceOrigin;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct LoadedNavigationConfigSource {
    value: Option<Value>,
    diagnostics: Vec<NavigationConfigSourceIssue>,
}

impl LoadedNavigationConfigSource {
    pub(crate) fn from_value(value: Value) -> Self {
        Self {
            value: Some(value),
            diagnostics: Vec::new(),
        }
    }

    pub(crate) fn value(&self) -> Option<&Value> {
        self.value.as_ref()
    }

    pub(crate) fn diagnostics(&self) -> &[NavigationConfigSourceIssue] {
        &self.diagnostics
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NavigationConfigSourceIssue {
    pub(crate) source_level: String,
    pub(crate) path_origin: String,
    pub(crate) path: String,
    pub(crate) field: Option<String>,
    pub(crate) reason_code: String,
}

impl NavigationConfigSourceIssue {
    pub(crate) fn new(
        source_level: impl Into<String>,
        path_origin: impl Into<String>,
        path: impl Into<String>,
        reason_code: impl Into<String>,
    ) -> Self {
        Self {
            source_level: source_level.into(),
            path_origin: path_origin.into(),
            path: path.into(),
            field: None,
            reason_code: reason_code.into(),
        }
    }

    pub(crate) fn message(&self) -> String {
        let field = self
            .field
            .as_ref()
            .map_or(String::new(), |field| format!(" field {field}"));
        format!(
            "adapter config source failed: {} {} {}{} ({})",
            self.source_level, self.path_origin, self.path, field, self.reason_code
        )
    }

    pub(crate) fn to_record_draft(&self, source: DiagnosticSource) -> DiagnosticRecordDraft {
        let field = self.field.as_deref().unwrap_or("config");
        let mut details = FieldReasonDetails::new(field, self.reason_code.clone());
        details.path = Some(self.path.clone());
        details.received = Some(self.field.clone().unwrap_or_else(|| self.path.clone()));
        let mut issue = AdapterConfigSourceDetails::new(
            &self.source_level,
            &self.path_origin,
            &self.path,
            &self.reason_code,
        );
        if let Some(field) = &self.field {
            issue = issue.with_field(field);
        }
        details.config_issues = Some(vec![issue]);
        DiagnosticRecordDraft::new::<typed_codes::protocol::InvalidRequest>(
            self.message(),
            details,
            source,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ConfigSourceSkipReason {
    MissingExplicitCli,
    MissingOverride,
    NotFile,
    Unreadable,
    InvalidJson,
    NonObject,
}

impl ConfigSourceSkipReason {
    const fn as_str(self) -> &'static str {
        match self {
            Self::MissingExplicitCli => "missing_explicit_cli",
            Self::MissingOverride => "missing_override",
            Self::NotFile => "not_file",
            Self::Unreadable => "unreadable",
            Self::InvalidJson => "invalid_json",
            Self::NonObject => "non_object",
        }
    }
}

pub(crate) fn load_config_source(
    level: &'static str,
    origin: NavigationConfigSourceOrigin,
    path: &Path,
) -> LoadedNavigationConfigSource {
    match fs::metadata(path) {
        Ok(metadata) if !metadata.is_file() => {
            skipped(level, origin, path, ConfigSourceSkipReason::NotFile)
        }
        Ok(_) => read_config_source(level, origin, path),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            missing_source(level, origin, path)
        }
        Err(_) => skipped(level, origin, path, ConfigSourceSkipReason::Unreadable),
    }
}

fn read_config_source(
    level: &'static str,
    origin: NavigationConfigSourceOrigin,
    path: &Path,
) -> LoadedNavigationConfigSource {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return missing_source(level, origin, path);
        }
        Err(_) => return skipped(level, origin, path, ConfigSourceSkipReason::Unreadable),
    };
    let value = match serde_json::from_str::<Value>(&content) {
        Ok(value) => value,
        Err(_) => return skipped(level, origin, path, ConfigSourceSkipReason::InvalidJson),
    };
    if !value.is_object() {
        return skipped(level, origin, path, ConfigSourceSkipReason::NonObject);
    }
    LoadedNavigationConfigSource::from_value(value)
}

fn missing_source(
    level: &'static str,
    origin: NavigationConfigSourceOrigin,
    path: &Path,
) -> LoadedNavigationConfigSource {
    match origin {
        NavigationConfigSourceOrigin::Default => LoadedNavigationConfigSource::default(),
        NavigationConfigSourceOrigin::ExplicitCli => skipped(
            level,
            origin,
            path,
            ConfigSourceSkipReason::MissingExplicitCli,
        ),
        NavigationConfigSourceOrigin::Override => {
            skipped(level, origin, path, ConfigSourceSkipReason::MissingOverride)
        }
    }
}

fn skipped(
    level: &'static str,
    origin: NavigationConfigSourceOrigin,
    path: &Path,
    reason: ConfigSourceSkipReason,
) -> LoadedNavigationConfigSource {
    let issue = NavigationConfigSourceIssue::new(
        level,
        origin.as_str(),
        path.display().to_string(),
        reason.as_str(),
    );
    LoadedNavigationConfigSource {
        value: None,
        diagnostics: vec![issue],
    }
}
