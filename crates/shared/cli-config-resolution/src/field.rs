use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::resolution::MergeStrategy;
use crate::source::{ConfigPath, SourceKind, SourceLocator};
use crate::value::{ReceivedValueKind, Value, ValueKind};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct FieldIdentity(String);

impl FieldIdentity {
    pub fn new(value: impl Into<String>) -> Result<Self, FieldSetBuildError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(FieldSetBuildError::InvalidFieldDeclaration(
                FieldBuildError::EmptyIdentity,
            ));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum DefaultMetadata {
    #[default]
    None,
    Static(Value),
    Dynamic(DynamicDefaultMetadata),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DynamicDefaultMetadata {
    key: String,
}

impl DynamicDefaultMetadata {
    pub fn new(key: impl Into<String>) -> Result<Self, FieldBuildError> {
        let key = key.into();
        if key.trim().is_empty() {
            return Err(FieldBuildError::InvalidDefaultMarker);
        }
        Ok(Self { key })
    }

    pub fn key(&self) -> &str {
        &self.key
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FieldConstraints {
    pub required: bool,
    pub nullable: bool,
    pub allowed_values: Vec<Value>,
    pub min_number: Option<f64>,
    pub max_number: Option<f64>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FieldContract {
    identity: FieldIdentity,
    value_kind: ValueKind,
    constraints: FieldConstraints,
    default: DefaultMetadata,
    projections: Vec<FieldProjection>,
    merge_strategy: MergeStrategy,
}

impl FieldContract {
    pub fn builder(identity: impl Into<String>, value_kind: ValueKind) -> FieldContractBuilder {
        FieldContractBuilder::new(identity, value_kind)
    }

    pub fn identity(&self) -> &FieldIdentity {
        &self.identity
    }

    pub fn value_kind(&self) -> ValueKind {
        self.value_kind
    }

    pub fn constraints(&self) -> &FieldConstraints {
        &self.constraints
    }

    pub fn default(&self) -> &DefaultMetadata {
        &self.default
    }

    pub fn projections(&self) -> &[FieldProjection] {
        &self.projections
    }

    pub fn merge_strategy(&self) -> MergeStrategy {
        self.merge_strategy
    }

    pub fn validate_value(&self, value: &Value) -> Result<(), ValidationFailure> {
        let received_kind = value.received_kind();
        if received_kind == ReceivedValueKind::Null {
            if self.constraints.nullable {
                return Ok(());
            }
            return Err(self.failure(ValidationReason::NullNotAllowed));
        }
        if !self.value_kind.accepts(received_kind) {
            return Err(self.failure(ValidationReason::WrongType {
                expected: self.value_kind,
                actual: received_kind,
            }));
        }
        if !self.constraints.allowed_values.is_empty()
            && !self
                .constraints
                .allowed_values
                .iter()
                .any(|allowed| allowed == value)
        {
            return Err(self.failure(ValidationReason::DisallowedValue {
                allowed: self.constraints.allowed_values.clone(),
            }));
        }
        if let Some(number) = number_value(value) {
            if let Some(minimum) = self.constraints.min_number {
                if number < minimum {
                    return Err(self.failure(ValidationReason::BelowMinimum { minimum }));
                }
            }
            if let Some(maximum) = self.constraints.max_number {
                if number > maximum {
                    return Err(self.failure(ValidationReason::AboveMaximum { maximum }));
                }
            }
        }
        if let Some(length) = value.len() {
            if let Some(minimum) = self.constraints.min_length {
                if length < minimum {
                    return Err(self.failure(ValidationReason::BelowMinimumLength { minimum }));
                }
            }
            if let Some(maximum) = self.constraints.max_length {
                if length > maximum {
                    return Err(self.failure(ValidationReason::AboveMaximumLength { maximum }));
                }
            }
        }
        Ok(())
    }

    fn failure(&self, reason: ValidationReason) -> ValidationFailure {
        ValidationFailure {
            field: self.identity.clone(),
            reason,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FieldContractBuilder {
    identity: String,
    value_kind: ValueKind,
    constraints: FieldConstraints,
    default: DefaultMetadata,
    projections: Vec<FieldProjectionDeclaration>,
    merge_strategy: MergeStrategy,
}

impl FieldContractBuilder {
    pub fn new(identity: impl Into<String>, value_kind: ValueKind) -> Self {
        Self {
            identity: identity.into(),
            value_kind,
            constraints: FieldConstraints::default(),
            default: DefaultMetadata::None,
            projections: Vec::new(),
            merge_strategy: MergeStrategy::ScalarReplace,
        }
    }

    pub fn constraints(mut self, constraints: FieldConstraints) -> Self {
        self.constraints = constraints;
        self
    }

    pub fn required(mut self) -> Self {
        self.constraints.required = true;
        self
    }

    pub fn nullable(mut self) -> Self {
        self.constraints.nullable = true;
        self
    }

    pub fn default(mut self, default: DefaultMetadata) -> Self {
        self.default = default;
        self
    }

    pub fn projection(mut self, projection: FieldProjectionDeclaration) -> Self {
        self.projections.push(projection);
        self
    }

    pub fn merge_strategy(mut self, merge_strategy: MergeStrategy) -> Self {
        self.merge_strategy = merge_strategy;
        self
    }

    pub fn build(self) -> Result<FieldContract, FieldBuildError> {
        let identity = FieldIdentity::new(self.identity).map_err(|error| match error {
            FieldSetBuildError::InvalidFieldDeclaration(error) => error,
            _ => FieldBuildError::EmptyIdentity,
        })?;
        if !merge_strategy_matches_kind(self.value_kind, self.merge_strategy) {
            return Err(FieldBuildError::IncompatibleMergeStrategy {
                value_kind: self.value_kind,
                merge_strategy: self.merge_strategy,
            });
        }
        let projections = self
            .projections
            .into_iter()
            .map(|projection| projection.build(identity.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        let field = FieldContract {
            identity,
            value_kind: self.value_kind,
            constraints: self.constraints,
            default: self.default,
            projections,
            merge_strategy: self.merge_strategy,
        };
        Ok(field)
    }
}

fn merge_strategy_matches_kind(value_kind: ValueKind, merge_strategy: MergeStrategy) -> bool {
    match merge_strategy {
        MergeStrategy::ScalarReplace | MergeStrategy::DenyConflict => true,
        MergeStrategy::ListAppend | MergeStrategy::ListReplace => value_kind == ValueKind::List,
        MergeStrategy::MapMerge | MergeStrategy::MapReplace => value_kind == ValueKind::Map,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldProjectionDeclaration {
    source_kind: SourceKind,
    locator: SourceLocator,
}

impl FieldProjectionDeclaration {
    pub fn new(source_kind: SourceKind, locator: SourceLocator) -> Self {
        Self {
            source_kind,
            locator,
        }
    }

    pub fn cli_flag(flag: impl Into<String>) -> Self {
        Self::new(SourceKind::Cli, SourceLocator::CliFlag(flag.into()))
    }

    pub fn env_var(name: impl Into<String>) -> Self {
        Self::new(SourceKind::Env, SourceLocator::EnvVar(name.into()))
    }

    pub fn config_path<I, S>(segments: I) -> Result<Self, FieldBuildError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let path = ConfigPath::new(segments).map_err(|_| FieldBuildError::InvalidProjectionPath)?;
        Ok(Self::new(
            SourceKind::Config,
            SourceLocator::ConfigPath(path),
        ))
    }

    pub fn default(locator: impl Into<String>) -> Self {
        Self::new(SourceKind::Default, SourceLocator::Default(locator.into()))
    }

    pub fn custom(source_name: impl Into<String>, locator: impl Into<String>) -> Self {
        Self::new(
            SourceKind::Custom(source_name.into()),
            SourceLocator::Custom(locator.into()),
        )
    }

    fn build(self, field: FieldIdentity) -> Result<FieldProjection, FieldBuildError> {
        if !locator_matches_kind(&self.source_kind, &self.locator) {
            return Err(FieldBuildError::IncompatibleProjectionLocator {
                source_kind: self.source_kind,
                locator: self.locator,
            });
        }
        if self.locator.as_key().trim().is_empty() {
            return Err(FieldBuildError::InvalidProjectionPath);
        }
        Ok(FieldProjection {
            field,
            source_kind: self.source_kind,
            locator: self.locator,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldProjection {
    field: FieldIdentity,
    source_kind: SourceKind,
    locator: SourceLocator,
}

impl FieldProjection {
    pub fn field(&self) -> &FieldIdentity {
        &self.field
    }

    pub fn source_kind(&self) -> &SourceKind {
        &self.source_kind
    }

    pub fn locator(&self) -> &SourceLocator {
        &self.locator
    }

    pub fn key(&self) -> ProjectionKey {
        ProjectionKey {
            source_kind: self.source_kind.clone(),
            locator: self.locator.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectionKey {
    source_kind: SourceKind,
    locator: SourceLocator,
}

impl ProjectionKey {
    pub fn source_kind(&self) -> &SourceKind {
        &self.source_kind
    }

    pub fn locator(&self) -> &SourceLocator {
        &self.locator
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FieldSet {
    fields: Vec<FieldContract>,
    by_identity: BTreeMap<FieldIdentity, usize>,
}

impl FieldSet {
    pub fn builder() -> FieldSetBuilder {
        FieldSetBuilder::default()
    }

    pub fn fields(&self) -> &[FieldContract] {
        &self.fields
    }

    pub fn get(&self, identity: &FieldIdentity) -> Option<&FieldContract> {
        self.by_identity
            .get(identity)
            .and_then(|index| self.fields.get(*index))
    }
}

#[derive(Clone, Debug)]
enum FieldSetDeclaration {
    Built(FieldContract),
    Builder(FieldContractBuilder),
}

#[derive(Clone, Debug, Default)]
pub struct FieldSetBuilder {
    declarations: Vec<FieldSetDeclaration>,
}

impl FieldSetBuilder {
    pub fn add_field(mut self, field: FieldContract) -> Self {
        self.declarations.push(FieldSetDeclaration::Built(field));
        self
    }

    pub fn add_declaration(mut self, field: FieldContractBuilder) -> Self {
        self.declarations.push(FieldSetDeclaration::Builder(field));
        self
    }

    pub fn build(self) -> Result<FieldSet, FieldSetBuildError> {
        let mut identities = BTreeMap::<FieldIdentity, usize>::new();
        let mut projection_keys = BTreeMap::<ProjectionKey, FieldIdentity>::new();
        let mut local_projection_keys = BTreeSet::<ProjectionKey>::new();
        let mut fields: Vec<FieldContract> = Vec::new();

        for declaration in self.declarations {
            let field = match declaration {
                FieldSetDeclaration::Built(field) => field,
                FieldSetDeclaration::Builder(builder) => builder
                    .build()
                    .map_err(FieldSetBuildError::InvalidFieldDeclaration)?,
            };
            if let Some(previous_index) = identities.get(field.identity()) {
                return Err(FieldSetBuildError::DuplicateIdentity {
                    field: field.identity().clone(),
                    previous: fields[*previous_index].identity().clone(),
                });
            }
            local_projection_keys.clear();
            for projection in field.projections() {
                let key = projection.key();
                if !local_projection_keys.insert(key.clone()) {
                    return Err(FieldSetBuildError::DuplicateProjectionLocator {
                        field: field.identity().clone(),
                        previous_field: field.identity().clone(),
                        key,
                    });
                }
                if let Some(previous_field) =
                    projection_keys.insert(key.clone(), field.identity().clone())
                {
                    return Err(FieldSetBuildError::DuplicateProjectionLocator {
                        field: field.identity().clone(),
                        previous_field,
                        key,
                    });
                }
            }
            identities.insert(field.identity().clone(), fields.len());
            fields.push(field);
        }
        Ok(FieldSet {
            fields,
            by_identity: identities,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FieldSetBuildError {
    DuplicateIdentity {
        field: FieldIdentity,
        previous: FieldIdentity,
    },
    DuplicateProjectionLocator {
        field: FieldIdentity,
        previous_field: FieldIdentity,
        key: ProjectionKey,
    },
    InvalidFieldDeclaration(FieldBuildError),
}

impl fmt::Display for FieldSetBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateIdentity { field, .. } => {
                write!(
                    formatter,
                    "field {} is declared more than once",
                    field.as_str()
                )
            }
            Self::DuplicateProjectionLocator { field, key, .. } => write!(
                formatter,
                "field {} uses duplicate projection locator {:?}",
                field.as_str(),
                key.locator()
            ),
            Self::InvalidFieldDeclaration(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for FieldSetBuildError {}

#[derive(Clone, Debug, PartialEq)]
pub enum FieldBuildError {
    EmptyIdentity,
    InvalidDefaultMarker,
    InvalidProjectionPath,
    IncompatibleMergeStrategy {
        value_kind: ValueKind,
        merge_strategy: MergeStrategy,
    },
    IncompatibleProjectionLocator {
        source_kind: SourceKind,
        locator: SourceLocator,
    },
}

impl fmt::Display for FieldBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyIdentity => write!(formatter, "field identity is empty"),
            Self::InvalidDefaultMarker => write!(formatter, "dynamic default marker is empty"),
            Self::InvalidProjectionPath => write!(formatter, "projection path is invalid"),
            Self::IncompatibleMergeStrategy {
                value_kind,
                merge_strategy,
            } => write!(
                formatter,
                "merge strategy {merge_strategy:?} is incompatible with value kind {value_kind:?}"
            ),
            Self::IncompatibleProjectionLocator {
                source_kind,
                locator,
            } => write!(
                formatter,
                "projection locator {:?} is incompatible with source kind {:?}",
                locator, source_kind
            ),
        }
    }
}

impl std::error::Error for FieldBuildError {}

#[derive(Clone, Debug, PartialEq)]
pub struct ValidationFailure {
    pub field: FieldIdentity,
    pub reason: ValidationReason,
}

impl fmt::Display for ValidationFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} failed validation: {:?}",
            self.field.as_str(),
            self.reason
        )
    }
}

impl std::error::Error for ValidationFailure {}

#[derive(Clone, Debug, PartialEq)]
pub enum ValidationReason {
    MissingRequired,
    NullNotAllowed,
    WrongType {
        expected: ValueKind,
        actual: ReceivedValueKind,
    },
    DisallowedValue {
        allowed: Vec<Value>,
    },
    BelowMinimum {
        minimum: f64,
    },
    AboveMaximum {
        maximum: f64,
    },
    BelowMinimumLength {
        minimum: usize,
    },
    AboveMaximumLength {
        maximum: usize,
    },
}

fn number_value(value: &Value) -> Option<f64> {
    match value {
        Value::Integer(value) => Some(*value as f64),
        Value::Number(value) => Some(*value),
        _ => None,
    }
}

fn locator_matches_kind(source_kind: &SourceKind, locator: &SourceLocator) -> bool {
    matches!(
        (source_kind, locator),
        (SourceKind::Cli, SourceLocator::CliFlag(_))
            | (SourceKind::Env, SourceLocator::EnvVar(_))
            | (SourceKind::Config, SourceLocator::ConfigPath(_))
            | (SourceKind::Default, SourceLocator::Default(_))
            | (SourceKind::Custom(_), SourceLocator::Custom(_))
    )
}
