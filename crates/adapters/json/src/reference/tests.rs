use super::*;
use crate::document::{load, JsonValue};

const MIXED_TREE_FIXTURE: &str = include_str!("../../tests/fixtures/mixed-tree.json");

#[test]
fn canonical_ref_encodes_root_and_special_tokens() {
    assert_eq!(canonical_ref(&[]), "json:#");
    assert_eq!(canonical_ref(&[""]), "json:#/");
    assert_eq!(canonical_ref(&["a/b~c"]), "json:#/a~1b~0c");
    assert_eq!(canonical_ref(&["\0control"]), "json:#/%00control");
    assert_eq!(canonical_ref(&["雪"]), "json:#/%E9%9B%AA");
    assert_eq!(canonical_ref(&["# %"]), "json:#/%23%20%25");
    assert_eq!(
        canonical_ref(&["!$&'()*+,:;=@?", "nested"]),
        "json:#/!$&'()*+,:;=@?/nested"
    );
}

#[test]
fn resolve_ref_round_trips_special_object_keys() {
    let document = load(MIXED_TREE_FIXTURE.as_bytes()).expect("mixed-tree fixture should load");
    let JsonValue::Object(members) = &document.root.value else {
        panic!("root should be an object");
    };

    for name in ["~", "/", "", "\0control", "雪", "01"] {
        let member = members
            .iter()
            .find(|member| member.name == name)
            .expect("fixture should contain the special member");
        let ref_id = canonical_ref(&[name]);
        let resolved = document
            .resolve_ref(&ref_id)
            .expect("generated ref should resolve");

        assert!(
            std::ptr::eq(resolved, &member.value),
            "{ref_id} should resolve to member {name:?}"
        );
        assert!(ref_id.is_ascii());
        assert!(!ref_id.chars().any(char::is_control));
    }
}

#[test]
fn resolve_ref_rejects_noncanonical_or_malformed_spelling() {
    let document = load(br#"{"A": 1, "~": 2}"#).expect("test document should load");
    let invalid_refs = [
        "",
        "json:",
        "yaml:#/A",
        "json:#A",
        "json:#/%",
        "json:#/%0",
        "json:#/%GG",
        "json:#/%e9%9B%AA",
        "json:#/%41",
        "json:#/%7E0",
        "json:#/%2F",
        "json:#/A B",
        "json:#/#",
        "json:#/雪",
        "json:#/\0",
        "json:#/~",
        "json:#/~2",
        "json:#/%FF",
        "json:#/%C3%28",
    ];

    for ref_id in invalid_refs {
        assert!(
            matches!(document.resolve_ref(ref_id), Err(RefError::Invalid { .. })),
            "{ref_id:?} should be rejected as noncanonical or malformed"
        );
    }
}

#[test]
fn resolve_ref_classifies_context_sensitive_paths() {
    let document = load(
        br#"{
            "items": ["zero", "one"],
            "object": {"01": "member"},
            "scalar": 0
        }"#,
    )
    .expect("test document should load");

    let array_value = document
        .resolve_ref("json:#/items/0")
        .expect("canonical in-range array index should resolve");
    assert_eq!(array_value.value, JsonValue::String("zero".to_owned()));

    let object_value = document
        .resolve_ref("json:#/object/01")
        .expect("numeric object token should remain an ordinary member name");
    assert_eq!(object_value.value, JsonValue::String("member".to_owned()));

    for ref_id in ["json:#/items/01", "json:#/items/-", "json:#/items/"] {
        assert!(
            matches!(document.resolve_ref(ref_id), Err(RefError::Invalid { .. })),
            "{ref_id} should be invalid in array context"
        );
    }

    for ref_id in [
        "json:#/missing",
        "json:#/items/9",
        "json:#/items/184467440737095516160",
        "json:#/scalar/child",
        "json:#/missing/01",
    ] {
        assert_eq!(
            document.resolve_ref(ref_id),
            Err(RefError::NotFound),
            "{ref_id} should be canonical but absent from this document"
        );
    }

    assert!(std::ptr::eq(
        document
            .resolve_ref(ROOT_REF)
            .expect("root ref should resolve"),
        &document.root
    ));
}
