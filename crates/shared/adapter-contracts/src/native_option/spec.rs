use docnav_protocol::Operation;
use docnav_typed_fields::{
    BuildError, ExpectedFieldShape, FieldDef, FieldDefBuilder, FieldDefDeclaration,
    FieldValidation, FieldValue, ProcessStrategy, ProcessingId, SchemaMetadataView, ValueKind,
};

use super::descriptions::{expected_value_description, value_kind_name};
use super::spec_error::AdapterOptionSpecError;

const CLI_PROCESSING: &str = "cli";

pub type AdapterOptionProcessStrategy = ProcessStrategy;

#[derive(Clone, Debug)]
pub struct AdapterOptionSpec {
    pub identity: String,
    pub owner: String,
    pub operations: Vec<Operation>,
    field: FieldDefDeclaration,
}

#[derive(Debug)]
pub struct AdapterOptionSpecBuilder<T = ()> {
    identity: String,
    owner: String,
    operations: Vec<Operation>,
    field: FieldDefBuilder<T>,
    declaration_path: Option<Vec<String>>,
    expected: ExpectedFieldShape,
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

    pub fn cli_flag(&self) -> Option<String> {
        self.field
            .processing_metadata(&ProcessingId::from(CLI_PROCESSING))
            .ok()
            .flatten()
            .and_then(|metadata| metadata.locator.cli_flag().map(str::to_owned))
    }

    pub fn cli_arg_id(&self) -> Option<String> {
        self.cli_flag()
            .map(|flag| flag.strip_prefix("--").unwrap_or(&flag).to_owned())
    }

    pub fn cli_input_path(&self) -> Option<Vec<String>> {
        self.cli_flag()?;
        Some(self.declaration_path().to_vec())
    }

    pub fn processing_path(
        &self,
        processing_id: impl Into<ProcessingId>,
    ) -> Result<Option<Vec<String>>, BuildError> {
        self.field
            .processing_metadata(&processing_id.into())
            .map(|metadata| {
                metadata.and_then(|metadata| {
                    metadata
                        .locator
                        .json_path()
                        .map(|path| path.segments().into_iter().map(str::to_owned).collect())
                })
            })
    }

    pub fn field_declaration(&self) -> Result<FieldDefDeclaration, AdapterOptionSpecError> {
        self.validate_declaration()?;
        Ok(self.field.clone())
    }

    pub fn validate_declaration(&self) -> Result<(), AdapterOptionSpecError> {
        let declaration_path = self.declaration_path();
        if !(declaration_path.len() == 2
            && declaration_path
                .first()
                .is_some_and(|segment| segment == "options")
            && declaration_path
                .get(1)
                .is_some_and(|segment| !segment.is_empty()))
        {
            return Err(AdapterOptionSpecError::InvalidDeclarationPath {
                identity: self.identity.clone(),
                path: declaration_path.to_vec(),
            });
        }

        self.schema_metadata().map(|_| ()).map_err(|error| {
            AdapterOptionSpecError::InvalidFieldDeclaration {
                identity: self.identity.clone(),
                reason: error.to_string(),
            }
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

    pub fn process(mut self, processing_id: impl Into<String>, strategy: ProcessStrategy) -> Self {
        self.field = self.field.process(processing_id, strategy);
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
        AdapterOptionSpec {
            identity: self.identity,
            owner: self.owner,
            operations: self.operations,
            field: FieldDefDeclaration::with_declaration_path(
                declaration_path,
                self.field,
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
        }
    }
}
