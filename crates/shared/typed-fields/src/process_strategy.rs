use crate::metadata::{BuildError, FieldPath};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessingInputKind {
    JsonValue,
    CliArguments,
    EnvironmentVariables,
    RustField,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcessingLocator {
    JsonPath(FieldPath),
    CliFlag(String),
    EnvVar(String),
    ConfigPath(FieldPath),
    RustField,
}

impl ProcessingLocator {
    pub fn json_path(&self) -> Option<&FieldPath> {
        match self {
            Self::JsonPath(path) | Self::ConfigPath(path) => Some(path),
            Self::CliFlag(_) | Self::EnvVar(_) | Self::RustField => None,
        }
    }

    pub fn cli_flag(&self) -> Option<&str> {
        match self {
            Self::CliFlag(flag) => Some(flag),
            _ => None,
        }
    }

    pub fn env_var(&self) -> Option<&str> {
        match self {
            Self::EnvVar(name) => Some(name),
            _ => None,
        }
    }

    pub fn config_path(&self) -> Option<&FieldPath> {
        match self {
            Self::ConfigPath(path) => Some(path),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProcessStrategy {
    kind: ProcessStrategyKind,
}

impl ProcessStrategy {
    pub fn json_path<I, S>(segments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            kind: ProcessStrategyKind::JsonPath(segments.into_iter().map(Into::into).collect()),
        }
    }

    pub fn cli_flag(flag: impl Into<String>) -> Self {
        Self {
            kind: ProcessStrategyKind::CliFlag(flag.into()),
        }
    }

    pub fn env_var(name: impl Into<String>) -> Self {
        Self {
            kind: ProcessStrategyKind::EnvVar(name.into()),
        }
    }

    pub fn config_path<I, S>(segments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            kind: ProcessStrategyKind::ConfigPath(segments.into_iter().map(Into::into).collect()),
        }
    }

    pub fn rust_field() -> Self {
        Self {
            kind: ProcessStrategyKind::RustField,
        }
    }

    pub(crate) fn build(self) -> Result<BuiltProcessStrategy, crate::metadata::BuildError> {
        match self.kind {
            ProcessStrategyKind::JsonPath(segments) => {
                Ok(BuiltProcessStrategy::JsonPath(FieldPath::new(segments)?))
            }
            ProcessStrategyKind::CliFlag(flag) => {
                Ok(BuiltProcessStrategy::CliFlag(validate_cli_flag(flag)?))
            }
            ProcessStrategyKind::EnvVar(name) => {
                Ok(BuiltProcessStrategy::EnvVar(validate_env_var(name)?))
            }
            ProcessStrategyKind::ConfigPath(segments) => {
                Ok(BuiltProcessStrategy::ConfigPath(FieldPath::new(segments)?))
            }
            ProcessStrategyKind::RustField => Ok(BuiltProcessStrategy::RustField),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum ProcessStrategyKind {
    JsonPath(Vec<String>),
    CliFlag(String),
    EnvVar(String),
    ConfigPath(Vec<String>),
    RustField,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum BuiltProcessStrategy {
    JsonPath(FieldPath),
    CliFlag(String),
    EnvVar(String),
    ConfigPath(FieldPath),
    RustField,
}

impl BuiltProcessStrategy {
    pub(crate) fn input_kind(&self) -> ProcessingInputKind {
        match self {
            Self::JsonPath(_) | Self::ConfigPath(_) => ProcessingInputKind::JsonValue,
            Self::CliFlag(_) => ProcessingInputKind::CliArguments,
            Self::EnvVar(_) => ProcessingInputKind::EnvironmentVariables,
            Self::RustField => ProcessingInputKind::RustField,
        }
    }

    pub(crate) fn json_path(&self) -> Option<&FieldPath> {
        match self {
            Self::JsonPath(path) | Self::ConfigPath(path) => Some(path),
            Self::CliFlag(_) | Self::EnvVar(_) | Self::RustField => None,
        }
    }

    pub(crate) fn locator(&self) -> ProcessingLocator {
        match self {
            Self::JsonPath(path) => ProcessingLocator::JsonPath(path.clone()),
            Self::CliFlag(flag) => ProcessingLocator::CliFlag(flag.clone()),
            Self::EnvVar(name) => ProcessingLocator::EnvVar(name.clone()),
            Self::ConfigPath(path) => ProcessingLocator::ConfigPath(path.clone()),
            Self::RustField => ProcessingLocator::RustField,
        }
    }
}

fn validate_cli_flag(flag: String) -> Result<String, BuildError> {
    let is_long = flag
        .strip_prefix("--")
        .is_some_and(|name| !name.is_empty() && !name.starts_with('-') && valid_cli_name(name));
    let is_short = flag.strip_prefix('-').is_some_and(|name| {
        !name.starts_with('-') && name.chars().count() == 1 && valid_cli_name(name)
    });
    if is_long || is_short {
        Ok(flag)
    } else {
        Err(BuildError::InvalidCliFlag)
    }
}

fn valid_cli_name(name: &str) -> bool {
    name.chars()
        .all(|character| !character.is_whitespace() && character != '=')
}

fn validate_env_var(name: String) -> Result<String, BuildError> {
    let mut characters = name.chars();
    let Some(first) = characters.next() else {
        return Err(BuildError::InvalidEnvVar);
    };
    if !(first.is_ascii_alphabetic() || first == '_')
        || !characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        return Err(BuildError::InvalidEnvVar);
    }
    Ok(name)
}
