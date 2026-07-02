use std::fmt;

use docnav_adapter_contracts::{Adapter, AdapterResult};
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
    adapter: &dyn Adapter,
    request: &RequestEnvelope,
) -> ProtocolResponse {
    match execute_operation(adapter, request) {
        Ok(result) => ProtocolResponse::success(
            request.protocol_version.clone(),
            request.request_id.clone(),
            result,
        ),
        Err(error) => ProtocolResponse::failure_for_request(request, error.protocol_error()),
    }
}

pub fn execute_operation(
    adapter: &dyn Adapter,
    request: &RequestEnvelope,
) -> AdapterResult<OperationResult> {
    match (&request.operation, &request.arguments) {
        (Operation::Outline, OperationArguments::Outline(arguments)) => adapter
            .outline(request, arguments)
            .map(OperationResult::Outline),
        (Operation::Read, OperationArguments::Read(arguments)) => {
            adapter.read(request, arguments).map(OperationResult::Read)
        }
        (Operation::Find, OperationArguments::Find(arguments)) => {
            adapter.find(request, arguments).map(OperationResult::Find)
        }
        (Operation::Info, OperationArguments::Info(arguments)) => {
            adapter.info(request, arguments).map(OperationResult::Info)
        }
        _ => Err(docnav_adapter_contracts::AdapterError::invalid_request(
            "arguments",
            format!("arguments do not match operation {}", request.operation),
        )),
    }
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

#[cfg(test)]
mod tests {
    use docnav_adapter_contracts::{Adapter, AdapterError, AdapterResult};
    use docnav_protocol::{
        positive_result, AdapterIdentity, Entry, FindArguments, FindResult, FormatDescriptor,
        InfoArguments, InfoResult, Manifest, Operation, OperationArguments, OutlineArguments,
        OutlineResult, ProbeResult, ReadArguments, ReadResult, PROTOCOL_VERSION,
    };

    use super::*;

    #[derive(Clone, Copy)]
    struct StubAdapter;

    impl Adapter for StubAdapter {
        fn adapter_id(&self) -> &str {
            "stub"
        }

        fn manifest(&self) -> Manifest {
            Manifest {
                manifest_version: "0.1".to_owned(),
                adapter: AdapterIdentity {
                    id: "stub".to_owned(),
                    name: "Stub".to_owned(),
                    version: "0.1.0".to_owned(),
                },
                formats: vec![FormatDescriptor {
                    id: "stub".to_owned(),
                    extensions: vec![".stub".to_owned()],
                    content_types: vec!["text/stub".to_owned()],
                }],
            }
        }

        fn probe(&self, _path: &str) -> ProbeResult {
            unreachable!("navigation dispatch test does not probe")
        }

        fn outline(
            &self,
            _request: &RequestEnvelope,
            _arguments: &OutlineArguments,
        ) -> AdapterResult<OutlineResult> {
            Ok(OutlineResult {
                entries: vec![Entry {
                    ref_id: "stub:1".to_owned(),
                    label: "Stub".to_owned(),
                    kind: None,
                    location: None,
                    summary: None,
                    excerpt: None,
                    rank: None,
                    cost: None,
                    metadata: None,
                }],
                page: None,
            })
        }

        fn read(
            &self,
            _request: &RequestEnvelope,
            _arguments: &ReadArguments,
        ) -> AdapterResult<ReadResult> {
            Err(AdapterError::ref_not_found("missing"))
        }

        fn find(
            &self,
            _request: &RequestEnvelope,
            _arguments: &FindArguments,
        ) -> AdapterResult<FindResult> {
            Err(AdapterError::invalid_request(
                "arguments.query",
                "query is not indexed",
            ))
        }

        fn info(
            &self,
            _request: &RequestEnvelope,
            _arguments: &InfoArguments,
        ) -> AdapterResult<InfoResult> {
            Err(AdapterError::internal("stub-info-unimplemented"))
        }
    }

    // @case WB-NAVIGATION-DISPATCH-001
    #[test]
    fn protocol_request_maps_core_inputs_to_operation_arguments() {
        let request = protocol_request(OperationInput {
            operation: Operation::Outline,
            document_path: "docs/guide.md".to_owned(),
            ref_id: None,
            query: None,
            page: Some(positive_result(1).unwrap()),
            limit: Some(positive_result(80).unwrap()),
            options: Some(Options::from_iter([(
                "max_heading_level".to_owned(),
                2.into(),
            )])),
        })
        .expect("outline request");

        assert_eq!(request.protocol_version, PROTOCOL_VERSION);
        assert_eq!(request.operation, Operation::Outline);
        assert_eq!(request.document.path, "docs/guide.md");
        match request.arguments {
            OperationArguments::Outline(arguments) => {
                assert_eq!(arguments.page.get(), 1);
                assert_eq!(arguments.limit.get(), 80);
                assert_eq!(
                    arguments
                        .options
                        .as_ref()
                        .and_then(|options| options.get("max_heading_level"))
                        .and_then(|value| value.as_i64()),
                    Some(2)
                );
            }
            arguments => panic!("expected outline arguments, got {arguments:?}"),
        }
    }

    #[test]
    fn protocol_request_rejects_missing_read_ref() {
        let error = protocol_request(OperationInput {
            operation: Operation::Read,
            document_path: "docs/guide.md".to_owned(),
            ref_id: None,
            query: None,
            page: Some(positive_result(1).unwrap()),
            limit: Some(positive_result(80).unwrap()),
            options: None,
        })
        .expect_err("read ref required");

        assert_eq!(error.field(), "ref");
        assert_eq!(error.reason(), "read requires ref");
    }

    #[test]
    fn execute_protocol_request_dispatches_adapter_library_handle() {
        let request = protocol_request(OperationInput {
            operation: Operation::Outline,
            document_path: "docs/guide.stub".to_owned(),
            ref_id: None,
            query: None,
            page: Some(positive_result(1).unwrap()),
            limit: Some(positive_result(80).unwrap()),
            options: None,
        })
        .expect("outline request");

        let response = execute_protocol_request(&StubAdapter, &request);

        match response {
            ProtocolResponse::Success(success) => {
                assert_eq!(success.operation, Operation::Outline);
                assert!(success.ok);
            }
            ProtocolResponse::Failure(failure) => panic!("expected success, got {failure:?}"),
        }
    }
}
