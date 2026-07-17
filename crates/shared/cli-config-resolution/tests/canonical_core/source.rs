use cli_config_resolution::{
    resolve, ResolutionInputError, Source, SourceCandidate, SourceId, SourceKind,
};
use serde_json::json;

use super::support::{candidate, custom_field_set, direct_locator, identity, source};

#[test]
fn source_rejects_locator_kind_mismatch() {
    let mismatched = Source::new(
        SourceId::new("env").expect("source id"),
        SourceKind::Env,
        10,
        vec![SourceCandidate::value(
            identity("required"),
            direct_locator("required"),
            json!("value"),
        )],
    );
    assert!(mismatched.is_err());
}

#[test]
fn resolver_rejects_an_unknown_field_candidate() {
    let fields = custom_field_set("required", true);
    let unknown = source("custom", 10, [candidate("unknown", json!("value"))]);
    assert!(matches!(
        resolve(&fields, &[unknown]),
        Err(ResolutionInputError::UnknownField { .. })
    ));
}
