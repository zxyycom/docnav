use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use docnav_adapter_contracts::{
    Adapter, AdapterDefinition, AdapterDocument, AdapterError, AdapterResult, FindInput, InfoInput,
    OutlineInput, ReadInput,
};
use docnav_protocol::{
    AdapterIdentity, AutoReadReason, AutoReadResult, Cost, Entry, FindResult, FormatDescriptor,
    InfoResult, Manifest, Measurement, OperationResult, OutlineResult, ProtocolResponse,
    ReadResult, UnstructuredOutlineReason,
};
use serde_json::json;

use crate::{
    execute_loaded_navigation_command, NavigationAdapterRegistry, NavigationCommand,
    NavigationFailureLayer,
};

use super::super::support::{
    cli_value_candidate, config_sources, document_parameter_catalog, navigation_command,
};

mod find;
mod outline;
mod support;

use support::*;

#[test]
fn request_construction_failure_does_not_create_an_adapter_document() {
    let adapter = RecordingAdapter::new(OutlineResult::structured(Vec::new(), None), None);
    let command = command(docnav_protocol::Operation::Read, Vec::new());

    let error = execute_loaded_navigation_command(
        command,
        config_sources(json!({}), json!({})),
        &document_parameter_catalog(),
        &SingleRegistry::new(&adapter),
    )
    .expect_err("missing ref fails request construction");

    assert_eq!(
        error.failure_layer(),
        Some(NavigationFailureLayer::RequestConstruction)
    );
    assert_eq!(adapter.document_creations.load(Ordering::SeqCst), 0);
    assert_eq!(adapter.source_acquisitions.load(Ordering::SeqCst), 0);
    assert_eq!(adapter.source_decodes.load(Ordering::SeqCst), 0);
    assert_eq!(adapter.model_builds.load(Ordering::SeqCst), 0);
    assert_eq!(adapter.ref_productions.load(Ordering::SeqCst), 0);
    assert_eq!(adapter.read_resolutions.load(Ordering::SeqCst), 0);
    assert_eq!(adapter.peak_live_documents.load(Ordering::SeqCst), 0);
    assert_eq!(adapter.live_documents.load(Ordering::SeqCst), 0);
    assert_eq!(adapter.document_drops.load(Ordering::SeqCst), 0);
    assert_eq!(adapter.model_drops.load(Ordering::SeqCst), 0);
}

#[test]
fn adapter_document_is_dropped_during_handler_unwind() {
    let adapter = RecordingAdapter::new(OutlineResult::structured(Vec::new(), None), None)
        .panicking_on_outline();

    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = execute(
            &adapter,
            command(docnav_protocol::Operation::Outline, Vec::new()),
        );
    }));

    assert!(unwind.is_err());
    assert_eq!(adapter.document_creations.load(Ordering::SeqCst), 1);
    assert_eq!(adapter.source_acquisitions.load(Ordering::SeqCst), 1);
    assert_eq!(adapter.source_decodes.load(Ordering::SeqCst), 1);
    assert_eq!(adapter.model_builds.load(Ordering::SeqCst), 1);
    assert_eq!(adapter.ref_productions.load(Ordering::SeqCst), 0);
    assert_eq!(adapter.read_resolutions.load(Ordering::SeqCst), 0);
    assert_eq!(adapter.peak_live_documents.load(Ordering::SeqCst), 1);
    assert_eq!(adapter.live_documents.load(Ordering::SeqCst), 0);
    assert_eq!(adapter.document_drops.load(Ordering::SeqCst), 1);
    assert_eq!(adapter.model_drops.load(Ordering::SeqCst), 1);
}
