use docnav_protocol::{
    OperationResult, PositiveInteger, ProtocolResponse, ReadResult, PROTOCOL_VERSION,
};

use crate::{AdapterDefinition, AdapterDocument, ReadInput};

/// Proves an opaque ref can be read from the current document and a freshly prepared document.
///
/// This test helper deliberately treats the ref as an opaque string. It normalizes both reads to
/// page one, requires exact ref echo, and validates each result through the public protocol
/// contract. Adapter-owned tests remain responsible for pagination and selection correspondence.
#[track_caller]
pub fn assert_ref_round_trip(
    definition: &AdapterDefinition<'_>,
    same_document: &mut dyn AdapterDocument,
    input: &ReadInput,
) -> (ReadResult, ReadResult) {
    let mut page_one_input = input.clone();
    page_one_input.page = PositiveInteger::new(1).expect("one is a positive integer");

    let same = same_document
        .read(&page_one_input)
        .unwrap_or_else(|error| panic!("same-document ref read failed: {error:?}"));
    assert_valid_read(&page_one_input.ref_id, &same, "same-document");

    let mut fresh_document = definition.create_document(page_one_input.document_path.clone());
    let fresh = fresh_document
        .read(&page_one_input)
        .unwrap_or_else(|error| panic!("fresh-document ref read failed: {error:?}"));
    assert_valid_read(&page_one_input.ref_id, &fresh, "fresh-document");

    (same, fresh)
}

#[track_caller]
fn assert_valid_read(expected_ref: &str, result: &ReadResult, context: &str) {
    assert_eq!(
        result.ref_id, expected_ref,
        "{context} read must echo the opaque ref exactly"
    );
    ProtocolResponse::success(
        PROTOCOL_VERSION.to_owned(),
        format!("ref-conformance-{context}"),
        OperationResult::Read(result.clone()),
    )
    .validate()
    .unwrap_or_else(|error| panic!("{context} read result is not protocol-valid: {error}"));
}
