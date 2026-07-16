#[path = "canonical_core/facade.rs"]
mod facade;

// @case WB-PARAM-SOURCE-EXTRACTION-001
#[path = "canonical_core/env.rs"]
mod env;

// @case WB-PARAM-RESOLVE-001
#[path = "canonical_core/resolution.rs"]
mod resolution;

#[path = "canonical_core/source.rs"]
mod source;
#[path = "canonical_core/support.rs"]
mod support;
