use super::*;
use serde_json::json;

fn option_entry(owner: &str, source: &str, value: Value) -> OptionEntry {
    OptionEntry {
        identity: "docnav.adapters.markdown.options.mode".to_owned(),
        owner: owner.to_owned(),
        namespace: "options.markdown".to_owned(),
        key: "mode".to_owned(),
        source: source.to_owned(),
        type_variant: "string".to_owned(),
        value,
    }
}

#[test]
fn insert_entry_replaces_value_and_metadata_for_the_same_key() {
    let mut options = Options::new();
    options.insert_entry(option_entry("first", "config", json!("compact")));

    let previous = options.insert_entry(option_entry("replacement", "cli", json!("structured")));

    assert_eq!(previous, Some(json!("compact")));
    assert_eq!(options.get("mode"), Some(&json!("structured")));
    assert_eq!(options.entries().len(), 1);
    assert_eq!(options.entry_for_key("mode").unwrap().owner, "replacement");
    assert_eq!(options.entry_for_key("mode").unwrap().source, "cli");
}
