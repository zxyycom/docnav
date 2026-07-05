use docnav_protocol::Operation;
use docnav_typed_fields::{
    BuildError, ExpectedFieldShape, FieldDef, FieldDefBuilder, FieldDefDeclaration,
    FieldValidation, FieldValue, ProcessStrategy as FieldProcessStrategy, ProcessingId,
    SchemaMetadataView, ValueKind,
};

use super::descriptions::{expected_value_description, value_kind_name};
use super::spec_error::AdapterOptionSpecError;

#[derive(Clone, Debug)]
pub struct AdapterOptionSpec {
    pub identity: String,
    pub owner: String,
    pub operations: Vec<Operation>,
    sources: Vec<AdapterOptionSource>,
    field: FieldDefDeclaration,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AdapterOptionSource {
    processing_id: String,
    strategy: AdapterOptionProcessStrategy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdapterOptionProcessStrategy {
    JsonPath(Vec<String>),
    CliFlag { flag: &'static str },
}

#[derive(Debug)]
pub struct AdapterOptionSpecBuilder<T = ()> {
    identity: String,
    owner: String,
    operations: Vec<Operation>,
    field: FieldDefBuilder<T>,
    declaration_path: Option<Vec<String>>,
    expected: ExpectedFieldShape,
    sources: Vec<AdapterOptionSource>,
}

impl AdapterOptionSpec {
    pub fn builder(identity: impl Into<String>) -> AdapterOptionSpecBuilder {
        let identity = identity.into();
        AdapterOptionSpecBuilder {
            field: FieldDef::builder(identity.clone()),
            identity,
            owner: String::new(),
            operations: Vec::new(),
            declaration_path: None,
            expected: ExpectedFieldShape::optional(),
            sources: Vec::new(),
        }
    }

    pub fn applies_to(&self, operation: Operation) -> bool {
        self.operations.contains(&operation)
    }

    pub fn namespace(&self) -> &str {
        self.declaration_path()
            .first()
            .map(String::as_str)
            .unwrap_or("")
    }

    pub fn key(&self) -> &str {
        self.declaration_path()
            .get(1)
            .map(String::as_str)
            .unwrap_or("")
    }

    pub fn cli_flag(&self) -> Option<&'static str> {
        self.sources
            .iter()
            .find_map(|source| source.strategy.as_cli_flag())
    }

    pub fn cli_arg_id(&self) -> Option<&'static str> {
        self.cli_flag()
            .map(|flag| flag.strip_prefix("--").unwrap_or(flag))
    }

    pub fn cli_input_path(&self) -> Option<Vec<String>> {
        self.cli_flag()?;
        self.processing_path(ProcessingId::from("cli"))
            .ok()
            .flatten()
    }

    pub fn processing_path(
        &self,
        processing_id: impl Into<ProcessingId>,
    ) -> Result<Option<Vec<String>>, BuildError> {
        self.field
            .processing_metadata(&processing_id.into())
            .map(|metadata| {
                metadata.map(|metadata| {
                    metadata
                        .path
                        .segments()
                        .into_iter()
                        .map(str::to_owned)
                        .collect()
                })
            })
    }

    pub fn field_declaration(&self) -> Result<FieldDefDeclaration, AdapterOptionSpecError> {
        self.validate_declaration()?;
        Ok(self.field.clone())
    }

    pub fn validate_declaration(&self) -> Result<(), AdapterOptionSpecError> {
        let declaration_path = self.declaration_path();
        if declaration_path.len() == 2
            && declaration_path
                .first()
                .is_some_and(|segment| segment == "options")
            && declaration_path
                .get(1)
                .is_some_and(|segment| !segment.is_empty())
        {
            return Ok(());
        }
        Err(AdapterOptionSpecError::InvalidDeclarationPath {
            identity: self.identity.clone(),
            path: declaration_path.to_vec(),
        })
    }

    pub fn value_kind(&self) -> ValueKind {
        self.schema_metadata()
            .map(|metadata| metadata.value_kind)
            .unwrap_or(ValueKind::Json)
    }

    pub fn value_kind_name(&self) -> &'static str {
        value_kind_name(self.value_kind())
    }

    pub fn expected_value_description(&self) -> String {
        self.schema_metadata()
            .map(|metadata| expected_value_description(&metadata))
            .unwrap_or_else(|_| "a JSON value".to_owned())
    }

    fn schema_metadata(&self) -> Result<SchemaMetadataView, docnav_typed_fields::BuildError> {
        self.field.schema_metadata()
    }

    fn declaration_path(&self) -> &[String] {
        self.field.declaration_path().unwrap_or(&[])
    }
}

impl AdapterOptionProcessStrategy {
    pub fn json_path<I, S>(segments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::JsonPath(segments.into_iter().map(Into::into).collect())
    }

    pub fn cli_flag(flag: &'static str) -> Self {
        Self::CliFlag { flag }
    }

    fn as_cli_flag(&self) -> Option<&'static str> {
        match self {
            Self::CliFlag { flag } => Some(*flag),
            Self::JsonPath(_) => None,
        }
    }

    fn field_process(&self, declaration_path: &[String]) -> FieldProcessStrategy {
        match self {
            Self::JsonPath(path) => FieldProcessStrategy::json_path(path.clone()),
            Self::CliFlag { .. } => FieldProcessStrategy::json_path(declaration_path.to_vec()),
        }
    }
}

impl<T> AdapterOptionSpecBuilder<T> {
    pub fn owner(mut self, owner: impl Into<String>) -> Self {
        self.owner = owner.into();
        self
    }

    pub fn operations(mut self, operations: &[Operation]) -> Self {
        self.operations = operations.to_vec();
        self
    }

    pub fn path<I, S>(mut self, path: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.declaration_path = Some(path.into_iter().map(Into::into).collect());
        self
    }

    pub fn process(
        mut self,
        processing_id: impl Into<String>,
        strategy: AdapterOptionProcessStrategy,
    ) -> Self {
        let processing_id = processing_id.into();
        if let Some(existing) = self
            .sources
            .iter_mut()
            .find(|source| source.processing_id == processing_id)
        {
            existing.strategy = strategy;
        } else {
            self.sources.push(AdapterOptionSource {
                processing_id,
                strategy,
            });
        }
        self
    }

    pub fn default_static(mut self, value: impl Into<T>) -> Self
    where
        T: FieldValue,
    {
        self.field = self.field.default_static(value);
        self
    }

    pub fn expected_shape(mut self, expected: ExpectedFieldShape) -> Self {
        self.expected = expected;
        self
    }

    pub fn build(self) -> AdapterOptionSpec
    where
        T: 'static,
    {
        let declaration_path = self.declaration_path.unwrap_or_default();
        let mut field = self.field;
        for source in &self.sources {
            field = field.process(
                source.processing_id.clone(),
                source.strategy.field_process(&declaration_path),
            );
        }
        AdapterOptionSpec {
            identity: self.identity,
            owner: self.owner,
            operations: self.operations,
            sources: self.sources,
            field: FieldDefDeclaration::with_declaration_path(
                declaration_path,
                field,
                self.expected,
            ),
        }
    }
}

impl AdapterOptionSpecBuilder<()> {
    pub fn validation<U>(self, validation: FieldValidation<U>) -> AdapterOptionSpecBuilder<U> {
        AdapterOptionSpecBuilder {
            identity: self.identity,
            owner: self.owner,
            operations: self.operations,
            field: self.field.validation(validation),
            declaration_path: self.declaration_path,
            expected: self.expected,
            sources: self.sources,
        }
    }
}
