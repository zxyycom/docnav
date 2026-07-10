use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::field::{DefaultMetadata, FieldIdentity, FieldSet};
use crate::value::Value;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SourceId(String);

impl SourceId {
    pub fn new(value: impl Into<String>) -> Result<Self, SourceCollectionError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(SourceCollectionError::EmptySourceId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SourceKind {
    Cli,
    Env,
    Config,
    Default,
    Custom(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceLoadState {
    NotLoaded,
    Missing,
    Loaded,
    Invalid { reason: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceExplicitness {
    Explicit,
    Implicit,
    Fallback,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceSpec {
    id: SourceId,
    kind: SourceKind,
    priority: i32,
    applicable: bool,
    load_state: SourceLoadState,
    explicitness: SourceExplicitness,
}

impl SourceSpec {
    pub fn new(id: SourceId, kind: SourceKind, priority: i32) -> Self {
        let explicitness = if kind == SourceKind::Default {
            SourceExplicitness::Fallback
        } else {
            SourceExplicitness::Explicit
        };
        Self {
            id,
            kind,
            priority,
            applicable: true,
            load_state: SourceLoadState::Loaded,
            explicitness,
        }
    }

    pub fn with_applicability(mut self, applicable: bool) -> Self {
        self.applicable = applicable;
        self
    }

    pub fn with_load_state(mut self, load_state: SourceLoadState) -> Self {
        self.load_state = load_state;
        self
    }

    pub fn with_explicitness(mut self, explicitness: SourceExplicitness) -> Self {
        self.explicitness = explicitness;
        self
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

    pub fn is_applicable(&self) -> bool {
        self.applicable
    }

    pub fn load_state(&self) -> &SourceLoadState {
        &self.load_state
    }

    pub fn explicitness(&self) -> &SourceExplicitness {
        &self.explicitness
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SourceCollection {
    specs: Vec<SourceSpec>,
}

impl SourceCollection {
    pub fn new(specs: Vec<SourceSpec>) -> Result<Self, SourceCollectionError> {
        let mut ids = BTreeSet::new();
        for spec in &specs {
            if !ids.insert(spec.id.clone()) {
                return Err(SourceCollectionError::DuplicateSourceId(spec.id.clone()));
            }
        }
        Ok(Self { specs })
    }

    pub fn specs(&self) -> &[SourceSpec] {
        &self.specs
    }

    pub fn get(&self, id: &SourceId) -> Option<&SourceSpec> {
        self.specs.iter().find(|spec| spec.id() == id)
    }

    pub fn ordered_applicable(&self) -> Vec<(usize, &SourceSpec)> {
        let mut indexed = self
            .specs
            .iter()
            .enumerate()
            .filter(|(_, spec)| spec.is_applicable())
            .collect::<Vec<_>>();
        indexed.sort_by(|(left_index, left), (right_index, right)| {
            right
                .priority()
                .cmp(&left.priority())
                .then_with(|| left_index.cmp(right_index))
        });
        indexed
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceCollectionError {
    EmptySourceId,
    DuplicateSourceId(SourceId),
}

impl fmt::Display for SourceCollectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySourceId => write!(formatter, "source id is empty"),
            Self::DuplicateSourceId(id) => {
                write!(
                    formatter,
                    "source id {} is declared more than once",
                    id.as_str()
                )
            }
        }
    }
}

impl std::error::Error for SourceCollectionError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigPathError {
    EmptyPath,
    EmptySegment { index: usize },
}

impl fmt::Display for ConfigPathError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPath => formatter.write_str("config path is empty"),
            Self::EmptySegment { index } => {
                write!(
                    formatter,
                    "config path contains an empty segment at index {index}"
                )
            }
        }
    }
}

impl std::error::Error for ConfigPathError {}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ConfigPath(Vec<String>);

impl ConfigPath {
    pub fn new<I, S>(segments: I) -> Result<Self, ConfigPathError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let segments = segments.into_iter().map(Into::into).collect::<Vec<_>>();
        if segments.is_empty() {
            return Err(ConfigPathError::EmptyPath);
        }
        if let Some(index) = segments
            .iter()
            .position(|segment| segment.trim().is_empty())
        {
            return Err(ConfigPathError::EmptySegment { index });
        }
        Ok(Self(segments))
    }

    pub fn segments(&self) -> &[String] {
        &self.0
    }

    pub fn dotted(&self) -> String {
        self.0.join(".")
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SourceLocator {
    CliFlag(String),
    EnvVar(String),
    ConfigPath(ConfigPath),
    Default(String),
    Custom(String),
}

impl SourceLocator {
    pub fn as_key(&self) -> String {
        match self {
            Self::CliFlag(flag) => flag.clone(),
            Self::EnvVar(name) => name.clone(),
            Self::ConfigPath(path) => path.dotted(),
            Self::Default(label) | Self::Custom(label) => label.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CandidateState {
    Missing,
    Present(Value),
    Invalid {
        received: Option<Value>,
        reason: String,
    },
    ExplicitAbsent,
    DefaultFallback {
        value: Value,
        dynamic: bool,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct SourceCandidate {
    field: FieldIdentity,
    source_id: SourceId,
    source_kind: SourceKind,
    locator: SourceLocator,
    state: CandidateState,
}

impl SourceCandidate {
    pub fn new(
        field: FieldIdentity,
        source: &SourceSpec,
        locator: SourceLocator,
        state: CandidateState,
    ) -> Self {
        Self {
            field,
            source_id: source.id().clone(),
            source_kind: source.kind().clone(),
            locator,
            state,
        }
    }

    pub fn field(&self) -> &FieldIdentity {
        &self.field
    }

    pub fn source_id(&self) -> &SourceId {
        &self.source_id
    }

    pub fn source_kind(&self) -> &SourceKind {
        &self.source_kind
    }

    pub fn locator(&self) -> &SourceLocator {
        &self.locator
    }

    pub fn state(&self) -> &CandidateState {
        &self.state
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RawSourceValue {
    Present(Value),
    Invalid {
        received: Option<Value>,
        reason: String,
    },
    ExplicitAbsent,
}

impl RawSourceValue {
    fn into_candidate_state(self) -> CandidateState {
        match self {
            Self::Present(value) => CandidateState::Present(value),
            Self::Invalid { received, reason } => CandidateState::Invalid { received, reason },
            Self::ExplicitAbsent => CandidateState::ExplicitAbsent,
        }
    }
}

pub trait SourceExtractor {
    fn extract(&self, source: &SourceSpec, fields: &FieldSet) -> Vec<SourceCandidate>;
}

#[derive(Clone, Debug, Default)]
pub struct CliFlagSource {
    values: BTreeMap<String, RawSourceValue>,
}

impl CliFlagSource {
    pub fn new(values: BTreeMap<String, RawSourceValue>) -> Self {
        Self { values }
    }
}

impl SourceExtractor for CliFlagSource {
    fn extract(&self, source: &SourceSpec, fields: &FieldSet) -> Vec<SourceCandidate> {
        extract_keyed_source(source, fields, &SourceKind::Cli, &self.values)
    }
}

#[derive(Clone, Debug, Default)]
pub struct EnvVarSource {
    values: BTreeMap<String, RawSourceValue>,
}

impl EnvVarSource {
    pub fn new(values: BTreeMap<String, RawSourceValue>) -> Self {
        Self { values }
    }
}

impl SourceExtractor for EnvVarSource {
    fn extract(&self, source: &SourceSpec, fields: &FieldSet) -> Vec<SourceCandidate> {
        extract_keyed_source(source, fields, &SourceKind::Env, &self.values)
    }
}

#[derive(Clone, Debug, Default)]
pub struct CustomSource {
    values: BTreeMap<String, RawSourceValue>,
}

impl CustomSource {
    pub fn new(values: BTreeMap<String, RawSourceValue>) -> Self {
        Self { values }
    }
}

impl SourceExtractor for CustomSource {
    fn extract(&self, source: &SourceSpec, fields: &FieldSet) -> Vec<SourceCandidate> {
        let expected = source.kind().clone();
        extract_keyed_source(source, fields, &expected, &self.values)
    }
}

#[derive(Clone, Debug)]
pub struct ConfigDocumentSource {
    root: Value,
    path_values: BTreeMap<String, RawSourceValue>,
}

impl ConfigDocumentSource {
    pub fn new(root: Value) -> Self {
        Self {
            root,
            path_values: BTreeMap::new(),
        }
    }

    pub fn with_path_values(mut self, path_values: BTreeMap<String, RawSourceValue>) -> Self {
        self.path_values = path_values;
        self
    }
}

impl SourceExtractor for ConfigDocumentSource {
    fn extract(&self, source: &SourceSpec, fields: &FieldSet) -> Vec<SourceCandidate> {
        if source.kind() != &SourceKind::Config {
            return Vec::new();
        }
        fields
            .fields()
            .iter()
            .flat_map(|field| field.projections())
            .filter(|projection| projection.source_kind() == &SourceKind::Config)
            .map(|projection| {
                let state = match projection.locator() {
                    SourceLocator::ConfigPath(path) => self
                        .path_values
                        .get(&path.dotted())
                        .cloned()
                        .map(RawSourceValue::into_candidate_state)
                        .unwrap_or_else(|| {
                            lookup_config_path(&self.root, path)
                                .map(CandidateState::Present)
                                .unwrap_or(CandidateState::Missing)
                        }),
                    _ => CandidateState::Invalid {
                        received: None,
                        reason: "config projection does not use a config path locator".to_owned(),
                    },
                };
                SourceCandidate::new(
                    projection.field().clone(),
                    source,
                    projection.locator().clone(),
                    state,
                )
            })
            .collect()
    }
}

#[derive(Clone, Debug, Default)]
pub struct DefaultSource {
    dynamic_values: BTreeMap<String, RawSourceValue>,
}

impl DefaultSource {
    pub fn new(dynamic_values: BTreeMap<String, RawSourceValue>) -> Self {
        Self { dynamic_values }
    }

    pub fn from_values(dynamic_values: BTreeMap<String, Value>) -> Self {
        Self {
            dynamic_values: dynamic_values
                .into_iter()
                .map(|(key, value)| (key, RawSourceValue::Present(value)))
                .collect(),
        }
    }
}

impl SourceExtractor for DefaultSource {
    fn extract(&self, source: &SourceSpec, fields: &FieldSet) -> Vec<SourceCandidate> {
        if source.kind() != &SourceKind::Default {
            return Vec::new();
        }
        fields
            .fields()
            .iter()
            .map(|field| {
                let locator = field
                    .projections()
                    .iter()
                    .find(|projection| projection.source_kind() == &SourceKind::Default)
                    .map(|projection| projection.locator().clone())
                    .unwrap_or_else(|| {
                        SourceLocator::Default(field.identity().as_str().to_owned())
                    });
                let state = match field.default() {
                    DefaultMetadata::None => CandidateState::Missing,
                    DefaultMetadata::Static(value) => CandidateState::DefaultFallback {
                        value: value.clone(),
                        dynamic: false,
                    },
                    DefaultMetadata::Dynamic(metadata) => self
                        .dynamic_values
                        .get(metadata.key())
                        .cloned()
                        .map(|raw| match raw {
                            RawSourceValue::Present(value) => CandidateState::DefaultFallback {
                                value,
                                dynamic: true,
                            },
                            RawSourceValue::Invalid { received, reason } => {
                                CandidateState::Invalid { received, reason }
                            }
                            RawSourceValue::ExplicitAbsent => CandidateState::ExplicitAbsent,
                        })
                        .unwrap_or(CandidateState::Missing),
                };
                SourceCandidate::new(field.identity().clone(), source, locator, state)
            })
            .collect()
    }
}

fn extract_keyed_source(
    source: &SourceSpec,
    fields: &FieldSet,
    expected_kind: &SourceKind,
    values: &BTreeMap<String, RawSourceValue>,
) -> Vec<SourceCandidate> {
    if source.kind() != expected_kind {
        return Vec::new();
    }
    fields
        .fields()
        .iter()
        .flat_map(|field| field.projections())
        .filter(|projection| projection.source_kind() == source.kind())
        .map(|projection| {
            let key = projection.locator().as_key();
            let state = values
                .get(&key)
                .cloned()
                .map(RawSourceValue::into_candidate_state)
                .unwrap_or(CandidateState::Missing);
            SourceCandidate::new(
                projection.field().clone(),
                source,
                projection.locator().clone(),
                state,
            )
        })
        .collect()
}

fn lookup_config_path(root: &Value, path: &ConfigPath) -> Option<Value> {
    let mut current = root;
    for segment in path.segments() {
        let Value::Map(map) = current else {
            return None;
        };
        current = map.get(segment)?;
    }
    Some(current.clone())
}
