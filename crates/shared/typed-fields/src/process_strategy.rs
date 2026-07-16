use crate::metadata::{BuildError, FieldPath, ValueKind};

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
    cli_metadata: Vec<CliProcessingMetadata>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CliProcessingMetadata {
    pub help: Option<String>,
    pub value_name: Option<String>,
    pub boolean_encoding: Option<CliBooleanEncoding>,
}

impl CliProcessingMetadata {
    pub const fn new() -> Self {
        Self {
            help: None,
            value_name: None,
            boolean_encoding: None,
        }
    }

    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn value_name(mut self, value_name: impl Into<String>) -> Self {
        self.value_name = Some(value_name.into());
        self
    }

    pub fn boolean_encoding(mut self, encoding: CliBooleanEncoding) -> Self {
        self.boolean_encoding = Some(encoding);
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CliBooleanEncoding {
    PresenceMeansTrue,
    Explicit {
        true_token: Option<String>,
        false_token: Option<String>,
    },
}

impl CliBooleanEncoding {
    pub fn explicit(true_token: impl Into<String>, false_token: impl Into<String>) -> Self {
        Self::Explicit {
            true_token: Some(true_token.into()),
            false_token: Some(false_token.into()),
        }
    }

    fn validate(&self, value_kind: ValueKind) -> Result<(), BuildError> {
        if value_kind != ValueKind::Boolean {
            return Err(BuildError::IncompatibleCliBooleanEncoding { value_kind });
        }
        match self {
            Self::PresenceMeansTrue => Ok(()),
            Self::Explicit {
                true_token: Some(true_token),
                false_token: Some(false_token),
            } if true_token == false_token => Err(BuildError::AmbiguousCliBooleanMapping {
                token: true_token.clone(),
            }),
            Self::Explicit {
                true_token: Some(_),
                false_token: Some(_),
            } => Ok(()),
            Self::Explicit { .. } => Err(BuildError::IncompleteCliBooleanMapping),
        }
    }
}

impl ProcessStrategy {
    pub fn json_path<I, S>(segments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::new(ProcessStrategyKind::JsonPath(
            segments.into_iter().map(Into::into).collect(),
        ))
    }

    pub fn cli_flag(flag: impl Into<String>) -> Self {
        Self::new(ProcessStrategyKind::CliFlag(flag.into()))
    }

    pub fn env_var(name: impl Into<String>) -> Self {
        Self::new(ProcessStrategyKind::EnvVar(name.into()))
    }

    pub fn config_path<I, S>(segments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::new(ProcessStrategyKind::ConfigPath(
            segments.into_iter().map(Into::into).collect(),
        ))
    }

    pub fn rust_field() -> Self {
        Self::new(ProcessStrategyKind::RustField)
    }

    pub fn cli_metadata(mut self, metadata: CliProcessingMetadata) -> Self {
        self.cli_metadata.push(metadata);
        self
    }

    pub(crate) fn build(self) -> Result<BuiltProcessStrategy, crate::metadata::BuildError> {
        let Self {
            kind,
            mut cli_metadata,
        } = self;
        if cli_metadata.len() > 1 {
            return Err(BuildError::DuplicateCliMetadata);
        }
        let cli_metadata = cli_metadata.pop();
        if cli_metadata.is_some() && !matches!(&kind, ProcessStrategyKind::CliFlag(_)) {
            return Err(BuildError::CliMetadataRequiresCliFlag);
        }
        match kind {
            ProcessStrategyKind::JsonPath(segments) => {
                Ok(BuiltProcessStrategy::JsonPath(FieldPath::new(segments)?))
            }
            ProcessStrategyKind::CliFlag(flag) => Ok(BuiltProcessStrategy::CliFlag {
                flag: validate_cli_flag(flag)?,
                metadata: cli_metadata,
            }),
            ProcessStrategyKind::EnvVar(name) => {
                Ok(BuiltProcessStrategy::EnvVar(validate_env_var(name)?))
            }
            ProcessStrategyKind::ConfigPath(segments) => {
                Ok(BuiltProcessStrategy::ConfigPath(FieldPath::new(segments)?))
            }
            ProcessStrategyKind::RustField => Ok(BuiltProcessStrategy::RustField),
        }
    }

    fn new(kind: ProcessStrategyKind) -> Self {
        Self {
            kind,
            cli_metadata: Vec::new(),
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
    CliFlag {
        flag: String,
        metadata: Option<CliProcessingMetadata>,
    },
    EnvVar(String),
    ConfigPath(FieldPath),
    RustField,
}

impl BuiltProcessStrategy {
    pub(crate) fn input_kind(&self) -> ProcessingInputKind {
        match self {
            Self::JsonPath(_) | Self::ConfigPath(_) => ProcessingInputKind::JsonValue,
            Self::CliFlag { .. } => ProcessingInputKind::CliArguments,
            Self::EnvVar(_) => ProcessingInputKind::EnvironmentVariables,
            Self::RustField => ProcessingInputKind::RustField,
        }
    }

    pub(crate) fn json_path(&self) -> Option<&FieldPath> {
        match self {
            Self::JsonPath(path) | Self::ConfigPath(path) => Some(path),
            Self::CliFlag { .. } | Self::EnvVar(_) | Self::RustField => None,
        }
    }

    pub(crate) fn locator(&self) -> ProcessingLocator {
        match self {
            Self::JsonPath(path) => ProcessingLocator::JsonPath(path.clone()),
            Self::CliFlag { flag, .. } => ProcessingLocator::CliFlag(flag.clone()),
            Self::EnvVar(name) => ProcessingLocator::EnvVar(name.clone()),
            Self::ConfigPath(path) => ProcessingLocator::ConfigPath(path.clone()),
            Self::RustField => ProcessingLocator::RustField,
        }
    }

    pub(crate) fn cli_metadata(&self) -> Option<&CliProcessingMetadata> {
        match self {
            Self::CliFlag { metadata, .. } => metadata.as_ref(),
            _ => None,
        }
    }

    pub(crate) fn validate_cli_metadata(&self, value_kind: ValueKind) -> Result<(), BuildError> {
        let Some(encoding) = self
            .cli_metadata()
            .and_then(|metadata| metadata.boolean_encoding.as_ref())
        else {
            return Ok(());
        };
        encoding.validate(value_kind)
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
