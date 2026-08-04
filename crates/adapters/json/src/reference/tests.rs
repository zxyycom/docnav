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
fn comment_ref_views_parse_and_generate_canonical_tokens() {
    for (view, root_ref) in [
        (RefView::Base, "json:#"),
        (RefView::DirectComments, "json:comments:#"),
        (RefView::TailComments, "json:tail-comments:#"),
    ] {
        assert_eq!(canonical_ref_for_view(view, &[]), root_ref);
        assert_eq!(
            canonical_ref_for_view(view, &["", "a/b~c", "雪"]),
            format!("{root_ref}//a~1b~0c/%E9%9B%AA")
        );
        assert_eq!(
            ParsedRef::parse(root_ref),
            Ok(ParsedRef {
                view,
                tokens: Vec::new(),
            })
        );
    }

    assert_eq!(
        ParsedRef::parse("json:comments:#//a~1b~0c/%E9%9B%AA"),
        Ok(ParsedRef {
            view: RefView::DirectComments,
            tokens: vec!["".to_owned(), "a/b~c".to_owned(), "雪".to_owned()],
        })
    );
}

#[test]
fn resolve_selection_round_trips_special_object_keys() {
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
        let selection = document
            .resolve_selection(&ref_id)
            .expect("generated ref should resolve");

        assert!(
            std::ptr::eq(selection.frames[0].value, &member.value),
            "{ref_id} should resolve to member {name:?}"
        );
        assert!(ref_id.is_ascii());
        assert!(!ref_id.chars().any(char::is_control));
    }
}

#[test]
fn resolve_selection_rejects_noncanonical_or_malformed_spelling() {
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
            matches!(
                document.resolve_selection(ref_id),
                Err(RefError::Invalid { .. })
            ),
            "{ref_id:?} should be rejected as noncanonical or malformed"
        );
    }
}

#[test]
fn resolve_selection_classifies_context_sensitive_paths() {
    let document = load(
        br#"{
            "items": ["zero", "one"],
            "object": {"01": "member"},
            "scalar": 0
        }"#,
    )
    .expect("test document should load");

    let array_selection = document
        .resolve_selection("json:#/items/0")
        .expect("canonical in-range array index should resolve");
    assert_eq!(
        array_selection.frames[0].value.value,
        JsonValue::String("zero".to_owned())
    );

    let object_selection = document
        .resolve_selection("json:#/object/01")
        .expect("numeric object token should remain an ordinary member name");
    assert_eq!(
        object_selection.frames[0].value.value,
        JsonValue::String("member".to_owned())
    );

    for ref_id in ["json:#/items/01", "json:#/items/-", "json:#/items/"] {
        assert!(
            matches!(
                document.resolve_selection(ref_id),
                Err(RefError::Invalid { .. })
            ),
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
            document.resolve_selection(ref_id),
            Err(RefError::NotFound),
            "{ref_id} should be canonical but absent from this document"
        );
    }

    assert!(std::ptr::eq(
        document
            .resolve_selection(ROOT_REF)
            .expect("root ref should resolve")
            .frames[0]
            .value,
        &document.root
    ));
}

#[test]
fn resolve_selection_preserves_selected_first_binding_and_comment_context() {
    let source = r#"/* root direct */
{
  // empty-key direct
  "": [
    // index direct
    {
      "leaf": true
      // index tail
    }
    // empty-key tail
  ]
  // root tail
}
// document tail"#;
    let document = load(source.as_bytes()).expect("comment context fixture should load");

    let selection = document
        .resolve_selection("json:comments:#//0")
        .expect("array-element direct-comment ref should resolve");

    assert_eq!(selection.view, RefView::DirectComments);
    assert_eq!(selection.frames.len(), 3);
    assert!(matches!(
        selection.frames[0].binding,
        RefBinding::ArrayElement { canonical_index: 0 }
    ));
    assert!(matches!(
        selection.frames[1].binding,
        RefBinding::ObjectMember { decoded_key: "" }
    ));
    assert_eq!(selection.frames[2].binding, RefBinding::Root);
    assert_eq!(
        selection.frames[0].value.kind(),
        crate::document::JsonKind::Object
    );
    assert_eq!(
        selection.frames[1].value.kind(),
        crate::document::JsonKind::Array
    );
    assert_eq!(
        selection.frames[2].value.kind(),
        crate::document::JsonKind::Object
    );

    for frame in &selection.frames {
        assert!(frame.direct_comments.is_some());
        assert!(frame.tail_comments.is_some());
    }
    assert!(std::ptr::eq(
        selection.frames[0].direct_comments.unwrap(),
        selection.frames[0].value.direct_comments.as_ref().unwrap()
    ));
    assert!(std::ptr::eq(
        selection.frames[0].tail_comments.unwrap(),
        selection.frames[0].value.tail_comments.as_ref().unwrap()
    ));

    let tail_selection = document
        .resolve_selection("json:tail-comments:#//0")
        .expect("the same array element should also be a tail anchor");
    assert_eq!(tail_selection.view, RefView::TailComments);
    assert!(std::ptr::eq(
        tail_selection.frames[0].value,
        selection.frames[0].value
    ));
}

#[test]
fn resolve_comment_views_support_root_scalar_array_index_and_coexistence() {
    let scalar = load(b"/* root direct */ 1\n// root tail")
        .expect("comment-bearing root scalar should load");
    for ref_id in ["json:#", "json:comments:#", "json:tail-comments:#"] {
        let selection = scalar
            .resolve_selection(ref_id)
            .expect("every root view should resolve when its bundle exists");
        assert_eq!(selection.frames.len(), 1);
        assert_eq!(selection.frames[0].binding, RefBinding::Root);
        assert!(std::ptr::eq(selection.frames[0].value, &scalar.root));
    }

    let array = load(b"[/* index direct */ {\"value\": 1\n// index tail\n}]")
        .expect("comment-bearing array index should load");
    let direct = array
        .resolve_selection("json:comments:#/0")
        .expect("array index direct comments should resolve");
    let tail = array
        .resolve_selection("json:tail-comments:#/0")
        .expect("array index tail comments should resolve");
    assert!(matches!(
        direct.frames[0].binding,
        RefBinding::ArrayElement { canonical_index: 0 }
    ));
    assert!(std::ptr::eq(direct.frames[0].value, tail.frames[0].value));
    assert!(direct.frames[0].direct_comments.is_some());
    assert!(tail.frames[0].tail_comments.is_some());
}

#[test]
fn resolve_comment_refs_distinguishes_invalid_spelling_from_missing_selection() {
    let strict =
        load(br#"{"items":[0],"object":{"01":1}}"#).expect("strict JSON fixture should load");

    for ref_id in [
        "json:comment:#",
        "json:comments#",
        "json:tail:#",
        "json:tail-comments:/items",
        "json:comments:#items",
        "json:comments:#/items/01",
        "json:tail-comments:#/items/-",
        "json:comments:#/items/~2",
    ] {
        assert!(
            matches!(
                strict.resolve_selection(ref_id),
                Err(RefError::Invalid { .. })
            ),
            "{ref_id:?} should be invalid"
        );
    }

    for ref_id in [
        "json:comments:#",
        "json:tail-comments:#",
        "json:comments:#/items/0",
        "json:tail-comments:#/items/0",
        "json:comments:#/object/01",
        "json:tail-comments:#/missing",
    ] {
        assert_eq!(
            strict.resolve_selection(ref_id),
            Err(RefError::NotFound),
            "{ref_id:?} is canonical but has no current comment selection"
        );
    }

    assert_eq!(
        strict
            .resolve_selection("json:#/object/01")
            .expect("base ref compatibility should remain intact")
            .frames[0]
            .value
            .value,
        JsonValue::Number("1".to_owned())
    );
}
