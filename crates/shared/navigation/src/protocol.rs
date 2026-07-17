use std::fmt;

use docnav_adapter_contracts::{
    AdapterDefinition, AdapterError, AdapterResult, StandardOperationInput,
};
use docnav_protocol::{
    generate_request_id, Document, FindArguments, InfoArguments, Operation, OperationArguments,
    OperationResult, Options, OutlineArguments, PositiveInteger, ProtocolResponse, ReadArguments,
    RequestEnvelope, PROTOCOL_VERSION,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationInput {
    pub operation: Operation,
    pub document_path: String,
    pub ref_id: Option<String>,
    pub query: Option<String>,
    pub page: Option<PositiveInteger>,
    pub limit: Option<PositiveInteger>,
    pub options: Option<Options>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationInputError {
    field: &'static str,
    operation: Operation,
    argument: &'static str,
}

impl NavigationInputError {
    pub const fn field(&self) -> &'static str {
        self.field
    }

    pub fn reason(&self) -> String {
        format!("{} requires {}", self.operation, self.argument)
    }
}

impl fmt::Display for NavigationInputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.reason())
    }
}

impl std::error::Error for NavigationInputError {}

pub fn protocol_request(input: OperationInput) -> Result<RequestEnvelope, NavigationInputError> {
    let arguments = operation_arguments(&input)?;

    Ok(RequestEnvelope {
        protocol_version: PROTOCOL_VERSION.to_owned(),
        request_id: generate_request_id(),
        operation: input.operation,
        document: Document {
            path: input.document_path,
        },
        arguments,
    })
}

pub fn execute_protocol_request(
    adapter: &AdapterDefinition<'_>,
    request: &RequestEnvelope,
    standard_input: &StandardOperationInput,
) -> ProtocolResponse {
    match execute_operation(adapter, request, standard_input) {
        Ok(result) => ProtocolResponse::success(
            request.protocol_version.clone(),
            request.request_id.clone(),
            result,
        ),
        Err(error) => ProtocolResponse::failure_for_request(request, error.protocol_error()),
    }
}

pub fn execute_operation(
    adapter: &AdapterDefinition<'_>,
    request: &RequestEnvelope,
    standard_input: &StandardOperationInput,
) -> AdapterResult<OperationResult> {
    if request.operation != request.arguments.operation()
        || request.operation != standard_input.operation()
    {
        return Err(AdapterError::invalid_request(
            "arguments",
            format!("arguments do not match operation {}", request.operation),
        ));
    }
    adapter.execute_operation(standard_input)
}

fn operation_arguments(input: &OperationInput) -> Result<OperationArguments, NavigationInputError> {
    match input.operation {
        Operation::Outline => Ok(OperationArguments::Outline(OutlineArguments {
            limit: required_limit(input, "limit")?,
            page: required_page(input, "page")?,
            options: input.options.clone(),
        })),
        Operation::Read => Ok(OperationArguments::Read(ReadArguments {
            ref_id: required_ref_id(input)?,
            limit: required_limit(input, "limit")?,
            page: required_page(input, "page")?,
            options: input.options.clone(),
        })),
        Operation::Find => Ok(OperationArguments::Find(FindArguments {
            query: required_query(input)?,
            limit: required_limit(input, "limit")?,
            page: required_page(input, "page")?,
            options: input.options.clone(),
        })),
        Operation::Info => Ok(OperationArguments::Info(InfoArguments {
            options: input.options.clone(),
        })),
    }
}

fn required_limit(
    input: &OperationInput,
    argument: &'static str,
) -> Result<PositiveInteger, NavigationInputError> {
    input
        .limit
        .ok_or_else(|| missing_argument(input, "limit", argument))
}

fn required_page(
    input: &OperationInput,
    argument: &'static str,
) -> Result<PositiveInteger, NavigationInputError> {
    input
        .page
        .ok_or_else(|| missing_argument(input, "page", argument))
}

fn required_ref_id(input: &OperationInput) -> Result<String, NavigationInputError> {
    input
        .ref_id
        .clone()
        .ok_or_else(|| missing_argument(input, "ref", "ref"))
}

fn required_query(input: &OperationInput) -> Result<String, NavigationInputError> {
    input
        .query
        .clone()
        .ok_or_else(|| missing_argument(input, "query", "query"))
}

fn missing_argument(
    input: &OperationInput,
    field: &'static str,
    argument: &'static str,
) -> NavigationInputError {
    NavigationInputError {
        field,
        operation: input.operation,
        argument,
    }
}
