use std::collections::BTreeSet;
use std::fmt;

use docnav_typed_fields::{FieldIdentity, FieldPath, JsonValue, ProcessingLocator};

mod env;

pub use env::extract_env;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SourceId(String);

impl SourceId {
    pub fn new(value: impl Into<String>) -> Result<Self, SourceError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(SourceError::EmptySourceId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn static_default() -> Self {
        Self("static-default".to_owned())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceKind {
    Cli,
    Env,
    Config,
    Default,
    Custom(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceLocator {
    CliFlag(String),
    EnvVar(String),
    ConfigPath(FieldPath),
    Default(String),
    Custom(String),
}

impl SourceLocator {
    pub fn as_key(&self) -> String {
        match self {
            Self::CliFlag(flag) => flag.clone(),
            Self::EnvVar(name) => name.clone(),
            Self::ConfigPath(path) => path.segments().join("."),
            Self::Default(label) | Self::Custom(label) => label.clone(),
        }
    }
}

impl TryFrom<ProcessingLocator> for SourceLocator {
    type Error = SourceError;

    fn try_from(locator: ProcessingLocator) -> Result<Self, Self::Error> {
        match locator {
            ProcessingLocator::CliFlag(flag) => Ok(Self::CliFlag(flag)),
            ProcessingLocator::EnvVar(name) => Ok(Self::EnvVar(name)),
            ProcessingLocator::ConfigPath(path) => Ok(Self::ConfigPath(path)),
            ProcessingLocator::JsonPath(_) | ProcessingLocator::RustField => {
                Err(SourceError::UnsupportedProcessingLocator(locator))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CandidateInput {
    Value(JsonValue),
    Invalid { raw: JsonValue, reason: String },
}

impl CandidateInput {
    pub fn raw(&self) -> &JsonValue {
        match self {
            Self::Value(value) | Self::Invalid { raw: value, .. } => value,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SourceCandidate {
    field: FieldIdentity,
    locator: SourceLocator,
    input: CandidateInput,
}

impl SourceCandidate {
    pub fn value(field: FieldIdentity, locator: SourceLocator, value: JsonValue) -> Self {
        Self {
            field,
            locator,
            input: CandidateInput::Value(value),
        }
    }

    pub fn invalid(
        field: FieldIdentity,
        locator: SourceLocator,
        raw: JsonValue,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            field,
            locator,
            input: CandidateInput::Invalid {
                raw,
                reason: reason.into(),
            },
        }
    }

    pub fn field(&self) -> &FieldIdentity {
        &self.field
    }

    pub fn locator(&self) -> &SourceLocator {
        &self.locator
    }

    pub fn input(&self) -> &CandidateInput {
        &self.input
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Source {
    id: SourceId,
    kind: SourceKind,
    priority: i32,
    candidates: Vec<SourceCandidate>,
}

impl Source {
    pub fn new(
        id: SourceId,
        kind: SourceKind,
        priority: i32,
        candidates: Vec<SourceCandidate>,
    ) -> Result<Self, SourceError> {
        validate_source_kind(&kind)?;
        let mut fields = BTreeSet::new();
        for candidate in &candidates {
            validate_locator(&kind, candidate.locator())?;
            if !fields.insert(candidate.field().clone()) {
                return Err(SourceError::DuplicateCandidate(candidate.field().clone()));
            }
        }
        Ok(Self {
            id,
            kind,
            priority,
            candidates,
        })
    }

    pub fn id(&self) -> &SourceId {
        &self.id
    }

    pub fn kind(&self) -> &SourceKind {
        &self.kind
    }

    pub fn priority(&self) -> i32 {
        self.priority
    }

    pub fn candidates(&self) -> &[SourceCandidate] {
        &self.candidates
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SourceError {
    EmptySourceId,
    EmptyCustomKind,
    EmptyLocator,
    DuplicateCandidate(FieldIdentity),
    IncompatibleLocator {
        source_kind: SourceKind,
        locator: SourceLocator,
    },
    UnsupportedProcessingLocator(ProcessingLocator),
}

impl fmt::Display for SourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySourceId => formatter.write_str("source id is empty"),
            Self::EmptyCustomKind => formatter.write_str("custom source kind is empty"),
            Self::EmptyLocator => formatter.write_str("source locator is empty"),
            Self::DuplicateCandidate(field) => write!(
                formatter,
                "source contains more than one candidate for field {}",
                field.as_str()
            ),
            Self::IncompatibleLocator {
                source_kind,
                locator,
            } => write!(
                formatter,
                "source locator {locator:?} is incompatible with source kind {source_kind:?}"
            ),
            Self::UnsupportedProcessingLocator(locator) => {
                write!(
                    formatter,
                    "processing locator {locator:?} is not a source locator"
                )
            }
        }
    }
}

impl std::error::Error for SourceError {}

fn validate_source_kind(kind: &SourceKind) -> Result<(), SourceError> {
    if matches!(kind, SourceKind::Custom(name) if name.trim().is_empty()) {
        Err(SourceError::EmptyCustomKind)
    } else {
        Ok(())
    }
}

fn validate_locator(kind: &SourceKind, locator: &SourceLocator) -> Result<(), SourceError> {
    let label = match locator {
        SourceLocator::CliFlag(label)
        | SourceLocator::EnvVar(label)
        | SourceLocator::Default(label)
        | SourceLocator::Custom(label) => Some(label),
        SourceLocator::ConfigPath(_) => None,
    };
    if label.is_some_and(|value| value.trim().is_empty()) {
        return Err(SourceError::EmptyLocator);
    }
    let matches = matches!(
        (kind, locator),
        (SourceKind::Cli, SourceLocator::CliFlag(_))
            | (SourceKind::Env, SourceLocator::EnvVar(_))
            | (SourceKind::Config, SourceLocator::ConfigPath(_))
            | (SourceKind::Default, SourceLocator::Default(_))
            | (SourceKind::Custom(_), SourceLocator::Custom(_))
    );
    if matches {
        Ok(())
    } else {
        Err(SourceError::IncompatibleLocator {
            source_kind: kind.clone(),
            locator: locator.clone(),
        })
    }
}
