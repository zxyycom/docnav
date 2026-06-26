use std::cell::Cell;

use super::*;

#[derive(Clone)]
struct NativeRawInput {
    text: String,
}

// @case WB-TYPED-FIELDS-PROCESSING-001
#[test]
fn processing_build_returns_caller_processed_value_for_typed_raw_input() {
    let calls = Cell::new(0);
    let processing = ProcessingBuild::new("native-input", |raw: NativeRawInput| {
        calls.set(calls.get() + 1);
        raw.text.len()
    })
    .expect("processing id is valid");

    let processed = processing.process(NativeRawInput {
        text: "docnav".to_owned(),
    });

    assert_eq!(processing.id().as_str(), "native-input");
    assert_eq!(processed.processing_id().as_str(), "native-input");
    assert_eq!(*processed.value(), 6);
    assert_eq!(processed.into_value(), 6);
    assert_eq!(calls.get(), 1);
}

#[test]
fn processing_build_rejects_empty_processing_id() {
    let error = ProcessingBuild::new(" ", |raw: NativeRawInput| raw.text).unwrap_err();

    assert_eq!(error, InvalidProcessingId);
}
