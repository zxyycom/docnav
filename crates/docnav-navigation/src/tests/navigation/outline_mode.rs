use std::fs;
use std::sync::atomic::Ordering;

use docnav_protocol::{Cost, ProtocolDiagnosticCode, ProtocolResponse, UnstructuredOutlineReason};
use serde_json::{json, Value};

use crate::execute_loaded_navigation_command;

use super::super::support::{config_sources, navigation_command, temp_workspace_path};

mod support;

use support::{
    command_for, measurement, success_outline, threshold_config, RecordingAdapter, SingleRegistry,
};

// @case WB-NAV-OUTLINE-MODE-001

#[test]
fn project_path_rule_overrides_user_rule_and_uses_default_utf8_fallback() {
    let workspace = temp_workspace_path("outline-mode-path-fallback");
    let document = workspace.join("raw.stub");
    fs::write(&document, "raw note").unwrap();
    let adapter = RecordingAdapter::default();
    adapter.fail_outline.store(true, Ordering::SeqCst);
    let outcome = execute_loaded_navigation_command(
        command_for(document.to_string_lossy()),
        config_sources(
            json!({
                "outline": {
                    "mode_rules": [
                        {"path": ".*raw\\.stub", "mode": "unstructured_full"}
                    ]
                }
            }),
            json!({
                "outline": {
                    "mode_rules": [
                        {"path": ".*raw\\.stub", "mode": "structured"}
                    ]
                }
            }),
        ),
        &SingleRegistry::new(&adapter),
    )
    .expect("path rule outcome");

    let result = success_outline(outcome.response);
    let result = result.as_unstructured().expect("unstructured outline");
    assert_eq!(result.reason, UnstructuredOutlineReason::PathRule);
    assert_eq!(result.content, "raw note");
    assert_eq!(result.content_type, "text/plain; charset=utf-8");
    assert!(result.cost.measurements.is_empty());
    assert_eq!(adapter.outline_calls.load(Ordering::SeqCst), 0);
    let _ = fs::remove_dir_all(workspace);
}

#[test]
fn later_structured_path_rule_opts_out_of_cost_threshold() {
    let mut adapter = RecordingAdapter::with_cost_units(["tokens"]);
    adapter.cost_measurements = vec![measurement("tokens", 1)];
    let outcome = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(
            json!({
                "outline": {
                    "mode_rules": [
                        {"path": "docs/guide\\.stub", "mode": "unstructured_full"},
                        {"path": "docs/guide\\.stub", "mode": "structured"}
                    ],
                    "auto_full_read": {
                        "thresholds": [
                            {"adapter": "docnav-markdown", "unit": "tokens", "value": 100}
                        ]
                    }
                }
            }),
            Value::Null,
        ),
        &SingleRegistry::new(&adapter),
    )
    .expect("structured opt-out outcome");

    let result = success_outline(outcome.response);
    let result = result.as_structured().expect("structured outline");
    assert_eq!(result.entries[0].label, "structured outline");
    assert_eq!(adapter.cost_requests.lock().unwrap().len(), 0);
    assert_eq!(adapter.outline_calls.load(Ordering::SeqCst), 1);
}

#[test]
fn threshold_filtering_and_unit_merge_keep_structured_when_minimum_not_met() {
    let mut adapter = RecordingAdapter::with_cost_units(["tokens"]);
    adapter.cost_measurements = vec![measurement("tokens", 1300)];
    let outcome = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(
            json!({
                "outline": {
                    "auto_full_read": {
                        "thresholds": [
                            {"adapter": "docnav-markdown", "unit": "tokens", "value": 1200}
                        ]
                    }
                }
            }),
            json!({
                "outline": {
                    "auto_full_read": {
                        "thresholds": [
                            {"adapter": "docnav-other", "unit": "tokens", "value": 1},
                            {"adapter": "docnav-markdown", "unit": "tokens", "value": 3000}
                        ]
                    }
                }
            }),
        ),
        &SingleRegistry::new(&adapter),
    )
    .expect("threshold miss outcome");

    let result = success_outline(outcome.response);
    assert!(result.as_structured().is_some());
    assert_eq!(
        *adapter.cost_requests.lock().unwrap(),
        vec![vec!["tokens".to_owned()]]
    );
}

#[test]
fn threshold_adapter_mismatch_does_not_request_cost_measurements() {
    let adapter = RecordingAdapter::with_cost_units(["tokens"]);
    let outcome = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(
            json!({
                "outline": {
                    "auto_full_read": {
                        "thresholds": [
                            {"adapter": "docnav-other", "unit": "tokens", "value": 100}
                        ]
                    }
                }
            }),
            Value::Null,
        ),
        &SingleRegistry::new(&adapter),
    )
    .expect("adapter mismatch threshold outcome");

    assert!(success_outline(outcome.response).as_structured().is_some());
    assert_eq!(adapter.cost_requests.lock().unwrap().len(), 0);
    assert_eq!(adapter.outline_calls.load(Ordering::SeqCst), 1);
}

#[test]
fn threshold_missing_measurement_and_runtime_unavailable_fall_back_to_structured() {
    let missing = RecordingAdapter::with_cost_units(["tokens"]);
    let missing_outcome = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(threshold_config("tokens", 100), Value::Null),
        &SingleRegistry::new(&missing),
    )
    .expect("missing measurement outcome");
    assert!(success_outline(missing_outcome.response)
        .as_structured()
        .is_some());
    assert_eq!(missing.cost_requests.lock().unwrap().len(), 1);

    let unavailable = RecordingAdapter::with_cost_units(["tokens"]);
    unavailable.cost_error.store(true, Ordering::SeqCst);
    let unavailable_outcome = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(threshold_config("tokens", 100), Value::Null),
        &SingleRegistry::new(&unavailable),
    )
    .expect("runtime unavailable outcome");
    assert!(success_outline(unavailable_outcome.response)
        .as_structured()
        .is_some());
    assert_eq!(unavailable.cost_requests.lock().unwrap().len(), 1);
}

#[test]
fn cost_threshold_triggers_hook_full_read_and_preserves_selector_cost() {
    let mut adapter = RecordingAdapter::with_cost_units(["tokens"]);
    adapter.content_hook.store(true, Ordering::SeqCst);
    adapter.full_read_content = "hook body".to_owned();
    adapter.full_read_content_type = "text/hook".to_owned();
    adapter.cost_measurements = vec![measurement("tokens", 8)];
    let outcome = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(threshold_config("tokens", 100), Value::Null),
        &SingleRegistry::new(&adapter),
    )
    .expect("cost-triggered full read outcome");

    let result = success_outline(outcome.response);
    let result = result.as_unstructured().expect("unstructured outline");
    assert_eq!(result.reason, UnstructuredOutlineReason::CostThreshold);
    assert_eq!(result.content, "hook body");
    assert_eq!(result.content_type, "text/hook");
    assert_eq!(result.cost.measurements, vec![measurement("tokens", 8)]);
    assert_eq!(adapter.outline_calls.load(Ordering::SeqCst), 0);
}

#[test]
fn path_triggered_hook_result_facts_are_used() {
    let mut adapter = RecordingAdapter::default();
    adapter.content_hook.store(true, Ordering::SeqCst);
    adapter.result_facts_hook.store(true, Ordering::SeqCst);
    adapter.full_read_content = "hook path body".to_owned();
    adapter.full_read_content_type = "text/hook".to_owned();
    adapter.facts_cost = Some(Cost {
        measurements: vec![measurement("lines", 1)],
    });
    let outcome = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(
            json!({
                "outline": {
                    "mode_rules": [
                        {"path": "docs/guide\\.stub", "mode": "unstructured_full"}
                    ]
                }
            }),
            Value::Null,
        ),
        &SingleRegistry::new(&adapter),
    )
    .expect("path-triggered hook outcome");

    let result = success_outline(outcome.response);
    let result = result.as_unstructured().expect("unstructured outline");
    assert_eq!(result.reason, UnstructuredOutlineReason::PathRule);
    assert_eq!(result.content, "hook path body");
    assert_eq!(result.content_type, "text/hook");
    assert_eq!(result.cost.measurements, vec![measurement("lines", 1)]);
    assert_eq!(adapter.outline_calls.load(Ordering::SeqCst), 0);
}

#[test]
fn path_triggered_default_fallback_reports_non_utf8_failure() {
    let workspace = temp_workspace_path("outline-mode-non-utf8");
    let document = workspace.join("raw.stub");
    fs::write(&document, [0xff, 0xfe]).unwrap();
    let adapter = RecordingAdapter::default();
    adapter.fail_outline.store(true, Ordering::SeqCst);
    let outcome = execute_loaded_navigation_command(
        command_for(document.to_string_lossy()),
        config_sources(
            json!({
                "outline": {
                    "mode_rules": [
                        {"path": ".*raw\\.stub", "mode": "unstructured_full"}
                    ]
                }
            }),
            Value::Null,
        ),
        &SingleRegistry::new(&adapter),
    )
    .expect("non-utf8 outcome");

    let ProtocolResponse::Failure(failure) = outcome.response else {
        panic!("expected failure response");
    };
    assert_eq!(
        failure.error.code(),
        ProtocolDiagnosticCode::DocumentEncodingUnsupported
    );
    assert_eq!(adapter.outline_calls.load(Ordering::SeqCst), 0);
    let _ = fs::remove_dir_all(workspace);
}

#[test]
fn invalid_path_rule_returns_source_scoped_diagnostic() {
    let error = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(
            json!({
                "outline": {
                    "mode_rules": [
                        {"path": "(", "mode": "unstructured_full"}
                    ]
                }
            }),
            Value::Null,
        ),
        &SingleRegistry::new(&RecordingAdapter::default()),
    )
    .expect_err("invalid rule should fail");
    let protocol_error = super::protocol_error(error.diagnostic());

    assert_eq!(
        protocol_error.code(),
        ProtocolDiagnosticCode::InvalidRequest
    );
    assert_eq!(
        protocol_error
            .details()
            .get("field")
            .and_then(Value::as_str),
        Some("outline.mode_rules[0].path")
    );
    assert_eq!(
        protocol_error
            .details()
            .get("reason")
            .and_then(Value::as_str),
        Some("invalid_path_pattern")
    );
}

#[test]
fn unregistered_outline_config_key_returns_source_scoped_diagnostic() {
    let error = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(
            json!({
                "outline": {
                    "bogus": true
                }
            }),
            Value::Null,
        ),
        &SingleRegistry::new(&RecordingAdapter::default()),
    )
    .expect_err("unregistered outline config key should fail");
    let protocol_error = super::protocol_error(error.diagnostic());

    assert_eq!(
        protocol_error.code(),
        ProtocolDiagnosticCode::InvalidRequest
    );
    assert_eq!(
        protocol_error
            .details()
            .get("field")
            .and_then(Value::as_str),
        Some("outline.bogus")
    );
    assert_eq!(
        protocol_error
            .details()
            .get("reason")
            .and_then(Value::as_str),
        Some("unknown_config_field")
    );
}

#[test]
fn unregistered_outline_rule_key_is_rejected_before_rule_parsing() {
    let error = execute_loaded_navigation_command(
        navigation_command(Vec::new()),
        config_sources(
            json!({
                "outline": {
                    "mode_rules": [
                        {"bogus": true, "path": "(", "mode": "structured"}
                    ]
                }
            }),
            Value::Null,
        ),
        &SingleRegistry::new(&RecordingAdapter::default()),
    )
    .expect_err("unregistered outline rule key should fail before regex parsing");
    let protocol_error = super::protocol_error(error.diagnostic());

    assert_eq!(
        protocol_error.code(),
        ProtocolDiagnosticCode::InvalidRequest
    );
    assert_eq!(
        protocol_error
            .details()
            .get("field")
            .and_then(Value::as_str),
        Some("outline.mode_rules[0].bogus")
    );
    assert_eq!(
        protocol_error
            .details()
            .get("reason")
            .and_then(Value::as_str),
        Some("unknown_config_field")
    );
}
