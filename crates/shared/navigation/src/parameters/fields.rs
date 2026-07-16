use std::fmt;

use docnav_adapter_contracts::{AdapterDefinition, AdapterOptionSpec};
use docnav_protocol::Operation;
use docnav_typed_fields::{
    ExpectedFieldShape, FieldDefBuilder, FieldDefDeclaration, FieldDefSet, FieldDefSetBuildError,
    FieldDefSetBuilder, FieldIdentity,
};

use crate::{NavigationAdapterRegistry, NavigationError, NavigationOutputMode};

use super::{
    values::uses_document_window, CONFIG_PROCESSING, DEFAULT_LIMIT, DEFAULT_PAGE,
    DEFAULT_PAGINATION_ENABLED, DIRECT_PROCESSING,
};

mod definitions;

#[cfg(test)]
mod tests;

use definitions::{
    adapter_id_field, configurable_limit_field, configurable_output_field, document_path_field,
    find_query_field, invocation_log_content_capture_enabled_field,
    invocation_log_content_capture_root_field, invocation_log_enabled_field,
    invocation_log_path_field, pagination_enabled_field, read_ref_field, standard_page_field,
};

const ADAPTER_OPTION_DECLARATION_ERROR_ID: &str = "adapter-option-field-declaration-invalid";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentCliFieldOwner {
    Navigation,
    Adapter,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentCliFieldAttribution {
    owner: DocumentCliFieldOwner,
    identity: FieldIdentity,
    declaration_path: Option<Vec<String>>,
    adapter_id: Option<String>,
}

impl DocumentCliFieldAttribution {
    pub fn owner(&self) -> DocumentCliFieldOwner {
        self.owner
    }

    pub fn identity(&self) -> &FieldIdentity {
        &self.identity
    }

    pub fn declaration_path(&self) -> Option<&[String]> {
        self.declaration_path.as_deref()
    }

    pub fn adapter_id(&self) -> Option<&str> {
        self.adapter_id.as_deref()
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
    InvalidAdapterOption {
        attribution: DocumentCliFieldAttribution,
        reason: String,
    },
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
            Self::InvalidAdapterOption {
                attribution,
                reason,
            } => write!(
                formatter,
                "adapter field {} at {} is invalid: {reason}",
                attribution.identity.as_str(),
                display_declaration_path(attribution.declaration_path())
            ),
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
            Self::InvalidAdapterOption { .. } => None,
        }
    }
}

pub(super) fn adapter_intent_fields() -> Result<FieldDefSet, NavigationError> {
    FieldDefSet::builder()
        .field_with_declaration_path(
            ["adapter"],
            adapter_id_field(DIRECT_PROCESSING, CONFIG_PROCESSING),
            ExpectedFieldShape::optional(),
        )
        .build()
        .map_err(|_| NavigationError::internal("adapter-intent-fields-build-failed"))
}

pub(super) struct OperationFieldSet {
    projection: DocumentCliFieldSet,
    adapter_options: Vec<AdapterOptionSpec>,
}

impl OperationFieldSet {
    pub(super) fn adapter_options(&self) -> &[AdapterOptionSpec] {
        &self.adapter_options
    }

    fn into_projection(self) -> DocumentCliFieldSet {
        self.projection
    }
}

impl AsRef<FieldDefSet> for OperationFieldSet {
    fn as_ref(&self) -> &FieldDefSet {
        self.projection.fields()
    }
}

struct OperationFieldSetBuilder {
    builder: FieldDefSetBuilder,
    adapter_options: Vec<AdapterOptionSpec>,
    attributions: Vec<DocumentCliFieldAttribution>,
}

impl OperationFieldSetBuilder {
    fn new() -> Self {
        Self {
            builder: FieldDefSet::builder(),
            adapter_options: Vec::new(),
            attributions: Vec::new(),
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
        let declaration =
            FieldDefDeclaration::with_declaration_path(declaration_path, builder, expected);
        let identity = declaration
            .schema_metadata()
            .expect("navigation-owned field declarations are valid")
            .identity;
        self.attributions.push(DocumentCliFieldAttribution {
            owner: DocumentCliFieldOwner::Navigation,
            identity,
            declaration_path: declaration.declaration_path().map(<[String]>::to_vec),
            adapter_id: None,
        });
        self.builder = self.builder.field_declaration(declaration);
        self
    }

    fn adapter_options(
        mut self,
        adapter_id: &str,
        options: &[AdapterOptionSpec],
        operation: Option<Operation>,
    ) -> Result<Self, DocumentCliFieldSetBuildError> {
        for option in options {
            if operation.is_some_and(|operation| !option.applies_to(operation)) {
                continue;
            }
            let declaration = option
                .field_declaration()
                .expect("adapter definition validates native option declarations");
            let attribution = adapter_field_attribution(adapter_id, &declaration);
            validate_adapter_option_config_path(option, &attribution)?;
            self.builder = self.builder.field_declaration(declaration);
            self.adapter_options.push(option.clone());
            self.attributions.push(attribution);
        }
        Ok(self)
    }

    fn all_adapter_options(
        mut self,
        registry: &(impl NavigationAdapterRegistry + ?Sized),
    ) -> Result<Self, NavigationError> {
        for adapter in registry.adapters() {
            self = self
                .adapter_options(
                    adapter.definition.id(),
                    adapter.definition.native_options(),
                    None,
                )
                .map_err(|_| invalid_adapter_option_declaration())?;
        }
        Ok(self)
    }

    fn build(self) -> Result<OperationFieldSet, DocumentCliFieldSetBuildError> {
        let Self {
            builder,
            adapter_options,
            attributions,
        } = self;
        let fields = builder
            .build()
            .map_err(|source| attributed_field_set_error(source, &attributions))?;
        Ok(OperationFieldSet {
            projection: DocumentCliFieldSet {
                fields,
                attributions,
            },
            adapter_options,
        })
    }
}

pub(crate) fn registry_cli_fields(
    operation: Operation,
    registry: &(impl NavigationAdapterRegistry + ?Sized),
) -> Result<DocumentCliFieldSet, DocumentCliFieldSetBuildError> {
    let mut builder = common_operation_fields(operation);
    for adapter in registry.adapters() {
        builder = builder.adapter_options(
            adapter.definition.id(),
            adapter.definition.native_options(),
            Some(operation),
        )?;
    }
    builder.build().map(OperationFieldSet::into_projection)
}

pub(super) fn operation_fields(
    operation: Operation,
    selected_adapter: &AdapterDefinition<'_>,
) -> Result<OperationFieldSet, NavigationError> {
    let builder = common_operation_fields(operation)
        .adapter_options(
            selected_adapter.id(),
            selected_adapter.native_options(),
            Some(operation),
        )
        .map_err(|_| invalid_adapter_option_declaration())?;

    builder
        .build()
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
        )
        .field_with_declaration_path(
            ["output"],
            configurable_output_field::<NavigationOutputMode>(DIRECT_PROCESSING, CONFIG_PROCESSING)
                .default_static(NavigationOutputMode::ReadableView),
            ExpectedFieldShape::required(),
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

    if uses_document_window(operation) {
        builder = builder
            .field_with_declaration_path(
                ["pagination"],
                pagination_enabled_field(DIRECT_PROCESSING, CONFIG_PROCESSING)
                    .default_static(DEFAULT_PAGINATION_ENABLED),
                ExpectedFieldShape::required(),
            )
            .field_with_declaration_path(
                ["page"],
                standard_page_field(DIRECT_PROCESSING).default_static(DEFAULT_PAGE),
                ExpectedFieldShape::required(),
            )
            .field_with_declaration_path(
                ["limit"],
                configurable_limit_field(DIRECT_PROCESSING, CONFIG_PROCESSING)
                    .default_static(DEFAULT_LIMIT),
                ExpectedFieldShape::required(),
            );
    }

    builder
}

pub(super) fn config_inspection_fields(
    registry: &(impl NavigationAdapterRegistry + ?Sized),
) -> Result<OperationFieldSet, NavigationError> {
    OperationFieldSetBuilder::new()
        .field_with_declaration_path(
            ["adapter"],
            adapter_id_field(DIRECT_PROCESSING, CONFIG_PROCESSING),
            ExpectedFieldShape::optional(),
        )
        .field_with_declaration_path(
            ["output"],
            configurable_output_field::<NavigationOutputMode>(DIRECT_PROCESSING, CONFIG_PROCESSING)
                .default_static(NavigationOutputMode::ReadableView),
            ExpectedFieldShape::required(),
        )
        .field_with_declaration_path(
            ["pagination"],
            pagination_enabled_field(DIRECT_PROCESSING, CONFIG_PROCESSING)
                .default_static(DEFAULT_PAGINATION_ENABLED),
            ExpectedFieldShape::required(),
        )
        .field_with_declaration_path(
            ["limit"],
            configurable_limit_field(DIRECT_PROCESSING, CONFIG_PROCESSING)
                .default_static(DEFAULT_LIMIT),
            ExpectedFieldShape::required(),
        )
        .field_with_declaration_path(
            ["invocation_log", "enabled"],
            invocation_log_enabled_field(CONFIG_PROCESSING),
            ExpectedFieldShape::optional(),
        )
        .field_with_declaration_path(
            ["invocation_log", "path"],
            invocation_log_path_field(CONFIG_PROCESSING),
            ExpectedFieldShape::optional(),
        )
        .field_with_declaration_path(
            ["invocation_log", "content_capture", "enabled"],
            invocation_log_content_capture_enabled_field(CONFIG_PROCESSING),
            ExpectedFieldShape::optional(),
        )
        .field_with_declaration_path(
            ["invocation_log", "content_capture", "root"],
            invocation_log_content_capture_root_field(CONFIG_PROCESSING),
            ExpectedFieldShape::optional(),
        )
        .all_adapter_options(registry)?
        .build()
        .map_err(|_| NavigationError::internal("config-inspection-fields-build-failed"))
}

fn validate_adapter_option_config_path(
    option: &AdapterOptionSpec,
    attribution: &DocumentCliFieldAttribution,
) -> Result<(), DocumentCliFieldSetBuildError> {
    let Some(path) = option
        .processing_path(CONFIG_PROCESSING)
        .expect("adapter definition validates native option processing metadata")
    else {
        return Ok(());
    };
    let adapter_id = attribution
        .adapter_id()
        .expect("adapter field attribution has an adapter id");
    if is_adapter_option_config_path(&path, adapter_id, option.key()) {
        Ok(())
    } else {
        Err(DocumentCliFieldSetBuildError::InvalidAdapterOption {
            attribution: attribution.clone(),
            reason: format!(
                "config locator {} must be options.{adapter_id}.{}",
                path.join("."),
                option.key()
            ),
        })
    }
}

fn is_adapter_option_config_path(path: &[String], adapter_id: &str, option_key: &str) -> bool {
    path.len() == 3
        && path.first().is_some_and(|segment| segment == "options")
        && path.get(1).is_some_and(|segment| segment == adapter_id)
        && path.get(2).is_some_and(|segment| segment == option_key)
}

fn invalid_adapter_option_declaration() -> NavigationError {
    NavigationError::internal(ADAPTER_OPTION_DECLARATION_ERROR_ID)
}

fn adapter_field_attribution(
    adapter_id: &str,
    declaration: &FieldDefDeclaration,
) -> DocumentCliFieldAttribution {
    let identity = declaration
        .schema_metadata()
        .expect("adapter definition validates native option declarations")
        .identity;
    DocumentCliFieldAttribution {
        owner: DocumentCliFieldOwner::Adapter,
        identity,
        declaration_path: declaration.declaration_path().map(<[String]>::to_vec),
        adapter_id: Some(adapter_id.to_owned()),
    }
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
    let owner = match attribution.owner {
        DocumentCliFieldOwner::Navigation => "navigation".to_owned(),
        DocumentCliFieldOwner::Adapter => format!(
            "adapter {}",
            attribution.adapter_id.as_deref().unwrap_or("<unknown>")
        ),
    };
    format!(
        "{owner} field {} at {}",
        attribution.identity.as_str(),
        display_declaration_path(attribution.declaration_path())
    )
}

fn display_declaration_path(path: Option<&[String]>) -> String {
    path.map(|path| path.join("."))
        .unwrap_or_else(|| "<unknown>".to_owned())
}
