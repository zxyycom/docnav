use cli_config_resolution::{
    ResolutionInputError, Resolver, Source, SourceCandidate, SourceId, SourceKind, SourceLocator,
};
use serde_json::json;

use super::support::{candidate, custom_field_set, identity, source};

#[test]
fn source_rejects_locator_kind_mismatch() {
    let mismatched = Source::new(
        SourceId::new("env").expect("source id"),
        SourceKind::Env,
        10,
        vec![SourceCandidate::value(
            identity("required"),
            SourceLocator::Custom("required".to_owned()),
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
        Resolver::resolve(&fields, &[unknown]),
        Err(ResolutionInputError::UnknownField { .. })
    ));
}
