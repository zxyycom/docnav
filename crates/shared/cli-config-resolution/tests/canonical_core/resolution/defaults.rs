use cli_config_resolution::{
    Resolver, Source, SourceCandidate, SourceId, SourceKind, SourceLocator, TypedValue,
};
use serde_json::json;

use crate::support::{identity, mode_field_set};

#[test]
fn static_default_fills_a_missing_source_value() {
    let defaults = Resolver::resolve(&mode_field_set(), &[]).expect("valid input");
    assert_eq!(
        defaults.materialize().expect("static default")[&identity("mode")],
        TypedValue::String("default".to_owned())
    );
    assert_eq!(
        defaults
            .trace(&identity("mode"))
            .expect("trace")
            .default_fallback
            .as_ref()
            .expect("default fact")
            .source_kind,
        SourceKind::Default
    );
}

#[test]
fn dynamic_default_remains_an_observable_source_fact() {
    let fields = mode_field_set();
    let dynamic_default = Source::new(
        SourceId::new("dynamic-default").expect("source id"),
        SourceKind::Default,
        0,
        vec![SourceCandidate::value(
            identity("mode"),
            SourceLocator::Default("runtime-mode".to_owned()),
            json!("dynamic"),
        )],
    )
    .expect("dynamic default source");
    let dynamic =
        Resolver::resolve(&fields, &[dynamic_default]).expect("valid dynamic default input");
    assert_eq!(
        dynamic.materialize().expect("dynamic default")[&identity("mode")],
        TypedValue::String("dynamic".to_owned())
    );
    let dynamic_fallback = dynamic
        .trace(&identity("mode"))
        .expect("dynamic trace")
        .default_fallback
        .as_ref()
        .expect("dynamic default source fact");
    assert_eq!(dynamic_fallback.source_id.as_str(), "dynamic-default");
    assert_eq!(dynamic_fallback.source_kind, SourceKind::Default);
    assert_eq!(
        dynamic_fallback.locator,
        SourceLocator::Default("runtime-mode".to_owned())
    );
    assert_eq!(dynamic_fallback.raw, json!("dynamic"));
}
