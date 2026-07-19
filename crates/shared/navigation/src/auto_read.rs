use docnav_adapter_contracts::{AdapterDefinition, ReadInput, StandardOperationInput};
use docnav_protocol::{
    AutoReadResult, Operation, OperationResult, OutlineResult, PositiveInteger, ProtocolResponse,
    ReadResult,
};

use crate::{execute_protocol_request, protocol_request, AutoReadMode, OperationInput};

pub(super) fn base_response_has_auto_read(response: &ProtocolResponse) -> bool {
    let ProtocolResponse::Success(success) = response else {
        return false;
    };
    match &success.result {
        OperationResult::Outline(OutlineResult::Structured(result)) => result.auto_read.is_some(),
        OperationResult::Find(result) => result.auto_read.is_some(),
        OperationResult::Outline(OutlineResult::Unstructured(_))
        | OperationResult::Read(_)
        | OperationResult::Info(_) => false,
    }
}

pub(super) fn compose_response(
    mode: Option<AutoReadMode>,
    adapter: &AdapterDefinition<'_>,
    base_input: &StandardOperationInput,
    base_response: ProtocolResponse,
) -> ProtocolResponse {
    if mode != Some(AutoReadMode::UniqueRef) {
        return base_response;
    }

    let Some(ref_id) = candidate_ref(&base_response).map(str::to_owned) else {
        return base_response;
    };
    let Some((document_path, limit)) = read_context(base_input) else {
        return base_response;
    };
    let page = PositiveInteger::new(1).expect("one is a positive integer");
    let standard_input = StandardOperationInput::Read(ReadInput {
        document_path: document_path.to_owned(),
        ref_id: ref_id.clone(),
        page,
        limit,
    });
    let Ok(request) = protocol_request(OperationInput {
        operation: Operation::Read,
        document_path: document_path.to_owned(),
        ref_id: Some(ref_id),
        query: None,
        page: Some(page),
        limit: Some(limit),
        options: None,
    }) else {
        return base_response;
    };
    let nested_response = execute_protocol_request(adapter, &request, &standard_input);
    let Some(read) = validated_read_result(nested_response) else {
        return base_response;
    };

    attach_validated_read(base_response, read)
}

fn candidate_ref(response: &ProtocolResponse) -> Option<&str> {
    let ProtocolResponse::Success(success) = response else {
        return None;
    };
    match &success.result {
        OperationResult::Outline(OutlineResult::Structured(result)) => {
            unique_ref(result.entries.iter().map(|entry| entry.ref_id.as_str()))
        }
        OperationResult::Find(result) => {
            unique_ref(result.matches.iter().map(|entry| entry.ref_id.as_str()))
        }
        OperationResult::Outline(OutlineResult::Unstructured(_))
        | OperationResult::Read(_)
        | OperationResult::Info(_) => None,
    }
}

fn unique_ref<'a>(refs: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    let mut candidate = None;
    for ref_id in refs.into_iter().filter(|ref_id| !ref_id.is_empty()) {
        match candidate {
            None => candidate = Some(ref_id),
            Some(existing) if existing == ref_id => {}
            Some(_) => return None,
        }
    }
    candidate
}

fn read_context(input: &StandardOperationInput) -> Option<(&str, PositiveInteger)> {
    match input {
        StandardOperationInput::Outline(input) => Some((&input.document_path, input.limit)),
        StandardOperationInput::Find(input) => Some((&input.document_path, input.limit)),
        StandardOperationInput::Read(_) | StandardOperationInput::Info(_) => None,
    }
}

fn validated_read_result(response: ProtocolResponse) -> Option<ReadResult> {
    response.validate().ok()?;
    let ProtocolResponse::Success(success) = response else {
        return None;
    };
    let OperationResult::Read(read) = success.result else {
        return None;
    };
    Some(read)
}

fn attach_validated_read(base_response: ProtocolResponse, read: ReadResult) -> ProtocolResponse {
    let mut composed = base_response.clone();
    let ProtocolResponse::Success(success) = &mut composed else {
        return base_response;
    };
    match &mut success.result {
        OperationResult::Outline(OutlineResult::Structured(result)) => {
            result.auto_read = Some(AutoReadResult::unique_ref(read));
        }
        OperationResult::Find(result) => {
            result.auto_read = Some(AutoReadResult::unique_ref(read));
        }
        OperationResult::Outline(OutlineResult::Unstructured(_))
        | OperationResult::Read(_)
        | OperationResult::Info(_) => return base_response,
    }

    if composed.validate().is_ok() {
        composed
    } else {
        base_response
    }
}

#[cfg(test)]
mod tests {
    use docnav_protocol::{
        Cost, OperationResult, OutlineResult, ProtocolResponse, ReadResult, SuccessResponse,
        PROTOCOL_VERSION,
    };

    use super::{attach_validated_read, unique_ref, validated_read_result};

    #[test]
    fn unique_ref_ignores_empty_refs_and_uses_string_exact_deduplication() {
        assert_eq!(unique_ref(["", "opaque:a", "opaque:a"]), Some("opaque:a"));
        assert_eq!(unique_ref(["opaque:a", "opaque:A"]), None);
        assert_eq!(unique_ref(["", ""]), None);
    }

    #[test]
    fn invalid_nested_success_is_not_accepted_as_a_read_result() {
        let response = ProtocolResponse::Success(SuccessResponse {
            protocol_version: PROTOCOL_VERSION.to_owned(),
            request_id: "nested".to_owned(),
            operation: docnav_protocol::Operation::Outline,
            ok: true,
            result: OperationResult::Read(read_result()),
        });

        assert_eq!(validated_read_result(response), None);
    }

    #[test]
    fn invalid_composed_response_falls_back_to_the_original_base() {
        let base = ProtocolResponse::Success(SuccessResponse {
            protocol_version: PROTOCOL_VERSION.to_owned(),
            request_id: "base".to_owned(),
            operation: docnav_protocol::Operation::Find,
            ok: true,
            result: OperationResult::Outline(OutlineResult::structured(Vec::new(), None)),
        });

        assert_eq!(attach_validated_read(base.clone(), read_result()), base);
    }

    fn read_result() -> ReadResult {
        ReadResult {
            ref_id: "opaque:a".to_owned(),
            content: "content".to_owned(),
            content_type: "text/plain".to_owned(),
            cost: Cost {
                measurements: Vec::new(),
            },
            page: None,
        }
    }
}
