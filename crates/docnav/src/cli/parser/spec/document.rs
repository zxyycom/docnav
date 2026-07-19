use std::fmt;

use clap::{Arg, ArgAction, Command};
use cli_config_resolution::ProcessingId;
use docnav_navigation::{document_adapter_routing_fields, DocumentParameterCatalog};
use docnav_protocol::Operation;
use docnav_typed_fields::{
    CliBooleanEncoding, DefaultMetadata, FieldIdentity, ProcessingLocator, ProcessingMetadataView,
    ValueKind,
};

use crate::error::{AppError, AppResult};
use crate::parameter_catalog::document_parameter_catalog;

use super::{static_document_clap_command, DocumentCliSpec};

pub(in crate::cli) fn document_clap_command(operation: Operation) -> AppResult<DocumentCliSpec> {
    let routing_fields = document_adapter_routing_fields().map_err(|error| {
        AppError::internal(format!(
            "document-routing-field-set-build-failed:{}:{error}",
            operation.as_str()
        ))
    })?;
    let parameters = document_parameter_catalog().map_err(|error| {
        AppError::internal(format!(
            "document-parameter-catalog-build-failed:{}:{error}",
            operation.as_str()
        ))
    })?;
    let processing_id = ProcessingId::new("cli").expect("document CLI processing id is valid");
    let mut command = static_document_clap_command(operation);
    for metadata in routing_fields.processing_metadata(&processing_id) {
        let argument = parameter_argument(&command, &metadata)
            .map_err(|error| document_projection_error(operation, "navigation", error))?;
        command = command.arg(argument);
    }
    let command = augment_parameter_command(command, &parameters, operation, &processing_id)
        .map_err(|error| document_projection_error(operation, "core-catalog", error))?;
    Ok(DocumentCliSpec {
        command,
        routing_fields,
        parameters,
    })
}

pub(in crate::cli) fn document_projection_error(
    operation: Operation,
    owner: &str,
    error: DocumentProjectionError,
) -> AppError {
    let field = error
        .field()
        .map_or("unknown", |identity| identity.as_str());
    AppError::internal(format!(
        "document-cli-projection-failed:{}:owner={owner}:field={field}:{error}",
        operation.as_str()
    ))
}

#[derive(Debug)]
pub(in crate::cli::parser) struct DocumentProjectionError {
    field: Option<FieldIdentity>,
    message: String,
}

impl DocumentProjectionError {
    pub(in crate::cli::parser) fn for_field(
        field: FieldIdentity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            field: Some(field),
            message: message.into(),
        }
    }

    pub(in crate::cli::parser) fn source(error: cli_config_resolution::SourceError) -> Self {
        Self {
            field: None,
            message: error.to_string(),
        }
    }

    fn field(&self) -> Option<&FieldIdentity> {
        self.field.as_ref()
    }
}

impl fmt::Display for DocumentProjectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

fn augment_parameter_command(
    mut command: Command,
    catalog: &DocumentParameterCatalog,
    operation: Operation,
    processing_id: &ProcessingId,
) -> Result<Command, DocumentProjectionError> {
    for field in catalog.operation_fields(operation) {
        let Some(metadata) = field.processing_metadata(processing_id) else {
            continue;
        };
        let argument = parameter_argument(&command, &metadata)?;
        command = command.arg(argument);
    }
    Ok(command)
}

fn parameter_argument(
    command: &Command,
    metadata: &ProcessingMetadataView<'_>,
) -> Result<Arg, DocumentProjectionError> {
    let flag = cli_flag(metadata)?;
    let long = available_long_flag(command, metadata, flag)?;
    let argument = Arg::new(metadata.identity().as_str().to_owned()).long(long.to_owned());
    let argument = attach_parameter_presentation(argument, metadata);
    configure_parameter_value(argument, metadata)
}

fn cli_flag<'a>(
    metadata: &'a ProcessingMetadataView<'_>,
) -> Result<&'a str, DocumentProjectionError> {
    let ProcessingLocator::CliFlag(flag) = &metadata.locator else {
        return Err(DocumentProjectionError::for_field(
            metadata.identity().clone(),
            format!(
                "field {} uses non-CLI processing locator {:?}",
                metadata.identity().as_str(),
                metadata.locator
            ),
        ));
    };
    Ok(flag)
}

fn available_long_flag<'a>(
    command: &Command,
    metadata: &ProcessingMetadataView<'_>,
    flag: &'a str,
) -> Result<&'a str, DocumentProjectionError> {
    let Some(long) = valid_long_flag(flag) else {
        return Err(DocumentProjectionError::for_field(
            metadata.identity().clone(),
            format!(
                "field {} has invalid CLI flag {flag}",
                metadata.identity().as_str()
            ),
        ));
    };
    if command
        .get_arguments()
        .any(|existing| existing.get_long() == Some(long))
    {
        return Err(DocumentProjectionError::for_field(
            metadata.identity().clone(),
            format!(
                "CLI flag {flag} for field {} conflicts with another clap argument",
                metadata.identity().as_str()
            ),
        ));
    }
    Ok(long)
}

fn attach_parameter_presentation(mut argument: Arg, metadata: &ProcessingMetadataView<'_>) -> Arg {
    if let Some(help) = parameter_help(metadata) {
        argument = argument.help(help);
    }
    if let Some(value_name) = metadata
        .cli
        .as_ref()
        .and_then(|metadata| metadata.value_name.clone())
    {
        argument = argument.value_name(value_name);
    }
    argument
}

fn configure_parameter_value(
    argument: Arg,
    metadata: &ProcessingMetadataView<'_>,
) -> Result<Arg, DocumentProjectionError> {
    let boolean_encoding = metadata
        .cli
        .as_ref()
        .and_then(|metadata| metadata.boolean_encoding.as_ref());
    match (metadata.value_kind(), boolean_encoding) {
        (
            ValueKind::Boolean,
            Some(CliBooleanEncoding::Explicit {
                true_token: Some(_),
                false_token: Some(_),
            }),
        ) => Ok(argument.action(ArgAction::Set).num_args(1)),
        (ValueKind::String, _) => Ok(argument.action(ArgAction::Set).num_args(1)),
        (ValueKind::Integer, _) => Ok(argument
            .action(ArgAction::Set)
            .num_args(1)
            .allow_negative_numbers(true)),
        _ => Err(DocumentProjectionError::for_field(
            metadata.identity().clone(),
            format!(
                "field {} has unsupported CLI value kind {:?}",
                metadata.identity().as_str(),
                metadata.value_kind()
            ),
        )),
    }
}

fn valid_long_flag(flag: &str) -> Option<&str> {
    let long = flag.strip_prefix("--")?;
    (!long.is_empty()
        && !long.starts_with('-')
        && long
            .chars()
            .all(|character| !character.is_whitespace() && character != '='))
    .then_some(long)
}

fn parameter_help(metadata: &ProcessingMetadataView<'_>) -> Option<String> {
    let mut canonical_facts = Vec::new();
    if let Some(accepted_values) = &metadata.constraints().enum_values {
        canonical_facts.push(format!(
            "possible values: {}",
            accepted_values
                .iter()
                .map(display_json_value)
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if let DefaultMetadata::Static(value) = metadata.default() {
        canonical_facts.push(format!("default: {}", display_json_value(value)));
    }
    let help = metadata
        .cli
        .as_ref()
        .and_then(|metadata| metadata.help.as_deref());
    match (help, canonical_facts.is_empty()) {
        (Some(help), true) => Some(help.to_owned()),
        (Some(help), false) => Some(format!("{help} [{}]", canonical_facts.join("; "))),
        (None, false) => Some(format!("[{}]", canonical_facts.join("; "))),
        (None, true) => None,
    }
}

fn display_json_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(value) => value.clone(),
        value => value.to_string(),
    }
}
