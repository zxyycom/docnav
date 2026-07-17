use std::fmt;

use docnav_protocol::Operation;
use docnav_typed_fields::{
    ExpectedFieldShape, FieldDef, FieldDefBuilder, FieldDefSet, FieldDefSetBuildError,
    FieldDefSetBuilder, FieldIdentity,
};

use crate::NavigationError;

use super::{catalog::DocumentParameterCatalog, CONFIG_PROCESSING, DIRECT_PROCESSING};

mod definitions;

#[cfg(test)]
mod tests;

use definitions::{
    adapter_id_field, document_path_field, find_query_field,
    invocation_log_content_capture_enabled_field, invocation_log_content_capture_root_field,
    invocation_log_enabled_field, invocation_log_path_field, read_ref_field,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentCliFieldAttribution {
    identity: FieldIdentity,
    declaration_path: Option<Vec<String>>,
}

impl DocumentCliFieldAttribution {
    pub fn identity(&self) -> &FieldIdentity {
        &self.identity
    }

    pub fn declaration_path(&self) -> Option<&[String]> {
        self.declaration_path.as_deref()
    }
}

#[derive(Debug)]
pub struct DocumentCliFieldSet {
    fields: FieldDefSet,
    attributions: Vec<DocumentCliFieldAttribution>,
}

impl DocumentCliFieldSet {
    pub fn fields(&self) -> &FieldDefSet {
        &self.fields
    }

    pub fn into_fields(self) -> FieldDefSet {
        self.fields
    }

    pub fn attribution(&self, identity: &FieldIdentity) -> Option<&DocumentCliFieldAttribution> {
        self.attributions
            .iter()
            .find(|attribution| attribution.identity() == identity)
    }
}

#[derive(Debug, PartialEq)]
pub enum DocumentCliFieldSetBuildError {
    DuplicateField {
        previous: Box<DocumentCliFieldAttribution>,
        current: Box<DocumentCliFieldAttribution>,
        source: Box<FieldDefSetBuildError>,
    },
    FieldSet {
        source: Box<FieldDefSetBuildError>,
    },
}

impl fmt::Display for DocumentCliFieldSetBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateField {
                previous,
                current,
                source,
            } => write!(
                formatter,
                "field declaration conflict between {} and {}: {source}",
                display_attribution(previous),
                display_attribution(current)
            ),
            Self::FieldSet { source } => write!(formatter, "field set build failed: {source}"),
        }
    }
}

impl std::error::Error for DocumentCliFieldSetBuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::DuplicateField { source, .. } | Self::FieldSet { source } => {
                Some(source.as_ref())
            }
        }
    }
}

pub(crate) fn adapter_routing_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    FieldDefSet::builder()
        .field_with_declaration_path(
            ["adapter"],
            adapter_id_field(DIRECT_PROCESSING, CONFIG_PROCESSING),
            ExpectedFieldShape::optional(),
        )
        .build()
}

pub(crate) fn invocation_log_fields() -> Result<FieldDefSet, FieldDefSetBuildError> {
    FieldDefSet::builder()
        .field(
            invocation_log_enabled_field(CONFIG_PROCESSING),
            ExpectedFieldShape::optional(),
        )
        .field(
            invocation_log_path_field(CONFIG_PROCESSING),
            ExpectedFieldShape::optional(),
        )
        .field(
            invocation_log_content_capture_enabled_field(CONFIG_PROCESSING),
            ExpectedFieldShape::optional(),
        )
        .field(
            invocation_log_content_capture_root_field(CONFIG_PROCESSING),
            ExpectedFieldShape::optional(),
        )
        .build()
}

pub(super) fn adapter_intent_fields() -> Result<FieldDefSet, NavigationError> {
    adapter_routing_fields()
        .map_err(|_| NavigationError::internal("adapter-intent-fields-build-failed"))
}

pub(super) struct OperationFieldSet {
    projection: DocumentCliFieldSet,
}

impl AsRef<FieldDefSet> for OperationFieldSet {
    fn as_ref(&self) -> &FieldDefSet {
        self.projection.fields()
    }
}

struct OperationFieldSetBuilder {
    builder: FieldDefSetBuilder,
    declaration_paths: Vec<Option<Vec<String>>>,
}

impl OperationFieldSetBuilder {
    fn new() -> Self {
        Self {
            builder: FieldDefSet::builder(),
            declaration_paths: Vec::new(),
        }
    }

    fn field_with_declaration_path<T, I, S>(
        mut self,
        declaration_path: I,
        builder: FieldDefBuilder<T>,
        expected: ExpectedFieldShape,
    ) -> Self
    where
        T: 'static,
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let declaration_path = declaration_path
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();
        self.declaration_paths.push(Some(declaration_path.clone()));
        self.builder =
            self.builder
                .field_with_declaration_path(declaration_path, builder, expected);
        self
    }

    fn build<'a>(
        self,
        catalog_fields: impl IntoIterator<Item = &'a FieldDef>,
    ) -> Result<OperationFieldSet, DocumentCliFieldSetBuildError> {
        let Self {
            builder,
            declaration_paths,
        } = self;
        let base_fields =
            builder
                .build()
                .map_err(|source| DocumentCliFieldSetBuildError::FieldSet {
                    source: Box::new(source),
                })?;
        let attributions = base_fields
            .schema_metadata()
            .into_iter()
            .zip(declaration_paths)
            .map(|(metadata, declaration_path)| DocumentCliFieldAttribution {
                identity: metadata.identity().clone(),
                declaration_path,
            })
            .collect::<Vec<_>>();
        let fields = base_fields
            .with_fields(catalog_fields)
            .map_err(|source| attributed_field_set_error(source, &attributions))?;
        Ok(OperationFieldSet {
            projection: DocumentCliFieldSet {
                fields,
                attributions,
            },
        })
    }
}

pub(super) fn operation_fields(
    operation: Operation,
    selected_adapter_id: &str,
    catalog: &DocumentParameterCatalog,
) -> Result<OperationFieldSet, NavigationError> {
    common_operation_fields(operation)
        .build(catalog.selected_operation_fields(selected_adapter_id, operation))
        .map_err(|_| NavigationError::internal("operation-fields-build-failed"))
}

fn common_operation_fields(operation: Operation) -> OperationFieldSetBuilder {
    let mut builder = OperationFieldSetBuilder::new()
        .field_with_declaration_path(
            ["path"],
            document_path_field(DIRECT_PROCESSING),
            ExpectedFieldShape::required(),
        )
        .field_with_declaration_path(
            ["adapter"],
            adapter_id_field(DIRECT_PROCESSING, CONFIG_PROCESSING),
            ExpectedFieldShape::optional(),
        );

    builder = match operation {
        Operation::Read => builder.field_with_declaration_path(
            ["ref"],
            read_ref_field(DIRECT_PROCESSING),
            ExpectedFieldShape::required(),
        ),
        Operation::Find => builder.field_with_declaration_path(
            ["query"],
            find_query_field(DIRECT_PROCESSING),
            ExpectedFieldShape::required(),
        ),
        Operation::Outline | Operation::Info => builder,
    };

    builder
}

fn attributed_field_set_error(
    source: FieldDefSetBuildError,
    attributions: &[DocumentCliFieldAttribution],
) -> DocumentCliFieldSetBuildError {
    let duplicate = duplicate_field_attributions(&source, attributions);
    match duplicate {
        Some((previous, current)) => DocumentCliFieldSetBuildError::DuplicateField {
            previous: Box::new(previous),
            current: Box::new(current),
            source: Box::new(source),
        },
        None => DocumentCliFieldSetBuildError::FieldSet {
            source: Box::new(source),
        },
    }
}

fn duplicate_field_attributions(
    source: &FieldDefSetBuildError,
    attributions: &[DocumentCliFieldAttribution],
) -> Option<(DocumentCliFieldAttribution, DocumentCliFieldAttribution)> {
    let (previous_identity, previous_path, current_identity, current_path) = match source {
        FieldDefSetBuildError::DuplicateIdentity(error) => (
            &error.field,
            &error.previous_declaration_path,
            &error.field,
            &error.declaration_path,
        ),
        FieldDefSetBuildError::DuplicateProcessingPath(error) => (
            &error.previous_identity,
            &error.previous_declaration_path,
            &error.current_identity,
            &error.current_declaration_path,
        ),
        FieldDefSetBuildError::DuplicateProcessingLocator(error) => (
            &error.previous_identity,
            &error.previous_declaration_path,
            &error.current_identity,
            &error.current_declaration_path,
        ),
        _ => return None,
    };
    let previous_index = find_attribution_index(attributions, previous_identity, previous_path, 0)?;
    let current_index = find_attribution_index(
        attributions,
        current_identity,
        current_path,
        previous_index + 1,
    )?;
    Some((
        attributions[previous_index].clone(),
        attributions[current_index].clone(),
    ))
}

fn find_attribution_index(
    attributions: &[DocumentCliFieldAttribution],
    identity: &FieldIdentity,
    declaration_path: &Option<Vec<String>>,
    start: usize,
) -> Option<usize> {
    attributions
        .iter()
        .enumerate()
        .skip(start)
        .find(|(_, attribution)| {
            attribution.identity() == identity && &attribution.declaration_path == declaration_path
        })
        .map(|(index, _)| index)
}

fn display_attribution(attribution: &DocumentCliFieldAttribution) -> String {
    format!(
        "field {} at {}",
        attribution.identity.as_str(),
        display_declaration_path(attribution.declaration_path())
    )
}

fn display_declaration_path(path: Option<&[String]>) -> String {
    path.map(|path| path.join("."))
        .unwrap_or_else(|| "<unknown>".to_owned())
}
