//! Integration tests that load committed conformance vector fixtures and
//! verify renderer output against order-independent assertions.
//!
//! Each fixture file under `tests/fixtures/conformance/` is loaded at compile
//! time via `include_str!`.  The test harness:
//!
//! 1. Deserializes the fixture into a [`ConformanceVector`].
//! 2. Builds a renderer config (default or overridden).
//! 3. Runs the renderer or verifies the expected failure.
//! 4. Checks every assertion against the output.
//!
//! # Stable assertion scope
//!
//! Assertions focus on **block pointer**, **byte length**, and **block
//! payload**.  Header key order, multi-block output order, and byte-for-byte
//! consistency are intentionally excluded from the stable contract.

use docnav_readable::conformance::ConformanceVector;

#[path = "conformance_support/mod.rs"]
mod conformance_support;

use conformance_support::run_vector;

// ── Fixture loading ──────────────────────────────────────────────────────

/// Macro to load a fixture file at compile time and deserialize it.
macro_rules! load_vector {
    ($path:literal) => {
        serde_json::from_str::<ConformanceVector>(include_str!($path))
            .expect(concat!("failed to parse conformance vector: ", $path))
    };
}

// ── Individual tests — one per committed fixture ─────────────────────────
//
// Each test loads its fixture at compile time via `include_str!` so the
// vector file is a committed, auditable acceptance artifact, not an ad-hoc
// in-test construction.
// @case WB-READABLE-VIEW-001

#[test]
fn conformance_01_no_block_outline() {
    run_vector(&load_vector!(
        "fixtures/conformance/01_no_block_outline.json"
    ));
}

#[test]
fn conformance_04_single_block() {
    run_vector(&load_vector!("fixtures/conformance/04_single_block.json"));
}

#[test]
fn conformance_07_chinese() {
    run_vector(&load_vector!("fixtures/conformance/07_chinese.json"));
}

#[test]
fn conformance_10_crlf_payload() {
    run_vector(&load_vector!("fixtures/conformance/10_crlf_payload.json"));
}

#[test]
fn conformance_11_no_trailing_newline() {
    run_vector(&load_vector!(
        "fixtures/conformance/11_no_trailing_newline.json"
    ));
}

#[test]
fn conformance_12_block_marker_in_body() {
    run_vector(&load_vector!(
        "fixtures/conformance/12_block_marker_in_body.json"
    ));
}

#[test]
fn conformance_13_warning() {
    run_vector(&load_vector!("fixtures/conformance/13_warning.json"));
}

#[test]
fn conformance_14_readable_error() {
    run_vector(&load_vector!("fixtures/conformance/14_readable_error.json"));
}

#[test]
fn conformance_15_error_guidance_array() {
    run_vector(&load_vector!(
        "fixtures/conformance/15_error_guidance_array.json"
    ));
}

#[test]
fn conformance_16_undeclared_extension_fields() {
    run_vector(&load_vector!(
        "fixtures/conformance/16_undeclared_extension_fields.json"
    ));
}

#[test]
fn conformance_17_order_independent_assertions() {
    run_vector(&load_vector!(
        "fixtures/conformance/17_order_independent_assertions.json"
    ));
}

#[test]
fn conformance_18_renderer_failure_missing_pointer() {
    run_vector(&load_vector!(
        "fixtures/conformance/18_renderer_failure_missing_pointer.json"
    ));
}

#[test]
fn conformance_19_renderer_failure_non_string() {
    run_vector(&load_vector!(
        "fixtures/conformance/19_renderer_failure_non_string.json"
    ));
}
