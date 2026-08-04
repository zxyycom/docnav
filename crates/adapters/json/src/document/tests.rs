use super::*;

const MIXED_TREE_FIXTURE: &str = include_str!("../../tests/fixtures/mixed-tree.json");
const INVALID_INPUTS_FIXTURE: &str = include_str!("../../tests/fixtures/invalid-inputs.txt");
const COMMENT_NAVIGATION_FIXTURE: &str =
    include_str!("../../tests/fixtures/comment-navigation.jsonc");

#[test]
fn load_tracks_bom_stripped_source_metadata_and_original_bytes() {
    // Keep the BOM byte-generated: a text fixture cannot distinguish it from
    // the BOM-stripped source whose exact metadata this test observes.
    let bytes = b"\xef\xbb\xbf \n{\"alpha\":[true,null],\"text\":\"x\"}\t";

    let document = load(bytes).expect("BOM-prefixed JSON should load");

    assert_eq!(
        document.source,
        " \n{\"alpha\":[true,null],\"text\":\"x\"}\t"
    );
    assert_eq!(document.original_byte_size, bytes.len());
    assert!(!document.has_jsonc_syntax);
    assert_eq!(document.root_kind(), JsonKind::Object);
    assert_eq!(document.root.depth, 0);
    assert_eq!(document.root.region, SourceRegion { start: 0, end: 35 });
    assert_eq!(document.node_count, 5);
    assert_eq!(document.max_depth, 2);
}

#[test]
fn load_rejects_encoding_syntax_trailing_input_and_a_second_bom() {
    // Invalid UTF-8 and a second BOM remain byte-generated because neither is
    // faithfully representable as an ordinary UTF-8 text fixture.
    assert_eq!(
        load(b"{\"key\":\"\xff\"}"),
        Err(LoadError::InvalidUtf8 { valid_up_to: 8 })
    );
    let syntax_failure = INVALID_INPUTS_FIXTURE
        .lines()
        .nth(1)
        .expect("invalid-input fixture should contain a syntax failure");
    assert!(matches!(
        load(syntax_failure.as_bytes()),
        Err(LoadError::InvalidJson { .. })
    ));
    let trailing_input = INVALID_INPUTS_FIXTURE
        .lines()
        .nth(2)
        .expect("invalid-input fixture should contain trailing input");
    assert!(matches!(
        load(trailing_input.as_bytes()),
        Err(LoadError::TrailingInput { .. })
    ));
    assert!(matches!(
        load(b"\xef\xbb\xbf\xef\xbb\xbf{}"),
        Err(LoadError::InvalidJson { .. })
    ));
}

#[test]
fn load_rejects_duplicate_decoded_member_names() {
    let duplicate_decoded_keys = INVALID_INPUTS_FIXTURE
        .lines()
        .next()
        .expect("invalid-input fixture should contain duplicate decoded keys");
    assert_eq!(
        load(duplicate_decoded_keys.as_bytes()),
        Err(LoadError::DuplicateMember {
            name: "a".to_owned()
        })
    );
}

#[test]
fn load_accepts_closed_jsonc_grammar_and_preserves_primary_model_facts() {
    let bytes = [UTF8_BOM, COMMENT_NAVIGATION_FIXTURE.as_bytes()].concat();

    let document = load(&bytes).expect("JSONC comments and one trailing comma should load");

    assert_eq!(document.source, COMMENT_NAVIGATION_FIXTURE);
    assert_eq!(document.original_byte_size, bytes.len());
    assert!(document.has_jsonc_syntax);
    assert_eq!(document.root_kind(), JsonKind::Object);
    assert_eq!(document.node_count, 4);
    assert_eq!(document.max_depth, 2);
    assert_eq!(
        document.root.region.as_str(&document.source),
        document.source
    );

    let JsonValue::Object(members) = &document.root.value else {
        panic!("JSONC root should retain its object model");
    };
    assert_eq!(
        members
            .iter()
            .map(|member| member.name.as_str())
            .collect::<Vec<_>>(),
        ["number", "items"]
    );
    assert_eq!(
        members[0].value.value,
        JsonValue::Number("-0.50e+02".to_owned())
    );
    assert_eq!(
        members[0].region.as_str(&document.source),
        "\"number\": -0.50e+02"
    );
    assert_eq!(
        members[1].value.region.as_str(&document.source),
        "[\n    /* element header */\n    true,\n  ]"
    );
    assert_eq!(
        document
            .comments
            .iter()
            .map(|comment| (comment.kind, comment.span.as_str(&document.source)))
            .collect::<Vec<_>>(),
        [
            (CommentKind::Line, "// document header"),
            (CommentKind::Line, "// member suffix"),
            (CommentKind::Block, "/* element header */"),
        ]
    );
    assert_eq!(
        document.root.direct_comments.as_ref().unwrap().indices(),
        &[0]
    );
    assert_eq!(
        members[0].value.direct_comments.as_ref().unwrap().indices(),
        &[1]
    );
    let JsonValue::Array(elements) = &members[1].value.value else {
        panic!("items should retain its array model");
    };
    assert_eq!(
        elements[0].direct_comments.as_ref().unwrap().indices(),
        &[2]
    );
    assert!(document.root.tail_comments.is_none());
}

#[test]
fn load_accepts_comment_line_endings_and_rejects_syntax_outside_closed_grammar() {
    let source = "[// lf\n1,// crlf\r\n2,// cr\r3,/* block */4,// eof\n5,]// eof";
    let document = load(source.as_bytes()).expect("all JSONC lexical line endings should load");
    assert_eq!(
        document
            .comments
            .iter()
            .map(|comment| comment.span.as_str(&document.source))
            .collect::<Vec<_>>(),
        [
            "// lf",
            "// crlf",
            "// cr",
            "/* block */",
            "// eof",
            "// eof"
        ]
    );
    let JsonValue::Array(elements) = &document.root.value else {
        panic!("line-ending corpus should retain its array model");
    };
    assert_comment_texts(
        &document,
        elements[0].direct_comments.as_ref(),
        &["// lf", "// crlf"],
    );
    assert_comment_texts(&document, elements[1].direct_comments.as_ref(), &["// cr"]);
    assert_comment_texts(
        &document,
        elements[2].direct_comments.as_ref(),
        &["/* block */"],
    );
    assert_comment_texts(&document, elements[3].direct_comments.as_ref(), &["// eof"]);
    assert_comment_texts(
        &document,
        document.root.direct_comments.as_ref(),
        &["// eof"],
    );

    assert_eq!(
        load(b"// \xff\n1"),
        Err(LoadError::InvalidUtf8 { valid_up_to: 3 })
    );
    assert_eq!(
        load(b"{/* comment */\"a\": 1, \"\\u0061\": 2,}"),
        Err(LoadError::DuplicateMember {
            name: "a".to_owned()
        })
    );
    for source in [b"{,}".as_slice(), b"[,]".as_slice()] {
        assert!(
            matches!(load(source), Err(LoadError::InvalidJson { .. })),
            "an empty root container comma remains invalid syntax: {source:?}"
        );
    }
    for source in [b"1 {,}".as_slice(), b"1 [,]".as_slice()] {
        assert!(
            matches!(load(source), Err(LoadError::TrailingInput { .. })),
            "an invalid empty container after a complete root is trailing input: {source:?}"
        );
    }
    for invalid in INVALID_INPUTS_FIXTURE.lines().skip(3) {
        assert!(
            matches!(
                load(invalid.as_bytes()),
                Err(LoadError::InvalidJson { .. } | LoadError::TrailingInput { .. })
            ),
            "closed JSONC grammar should reject {invalid:?}"
        );
    }
}

#[test]
fn load_attributes_direct_empty_and_tail_comments_once_in_source_order() {
    let source = r#"/* root leading */
{
  // first leading
  "first" /* first header */: 1 /* first suffix */,
  "items": [
    // element zero leading
    0, // element zero suffix
    // element one leading
    1
    /**/
  ],
  "empty-object": {
    // empty object
  },
  "empty-array": [/**/],
  "nested": {
    "plain": true
    // nested tail
  }
  // root internal tail
} // root suffix
/* document tail */"#;

    let document = load(source.as_bytes()).expect("placement corpus should load");
    let JsonValue::Object(members) = &document.root.value else {
        panic!("placement root should be an object");
    };
    let first = &members[0].value;
    let JsonValue::Array(items) = &members[1].value.value else {
        panic!("items should be an array");
    };
    let empty_object = &members[2].value;
    let empty_array = &members[3].value;
    let nested = &members[4].value;
    let JsonValue::Object(nested_members) = &nested.value else {
        panic!("nested should be an object");
    };

    assert_comment_texts(
        &document,
        document.root.direct_comments.as_ref(),
        &["/* root leading */", "// root suffix"],
    );
    assert_comment_texts(
        &document,
        first.direct_comments.as_ref(),
        &[
            "// first leading",
            "/* first header */",
            "/* first suffix */",
        ],
    );
    assert_comment_texts(
        &document,
        items[0].direct_comments.as_ref(),
        &["// element zero leading", "// element zero suffix"],
    );
    assert_comment_texts(
        &document,
        items[1].direct_comments.as_ref(),
        &["// element one leading"],
    );
    assert_comment_texts(
        &document,
        members[1].value.tail_comments.as_ref(),
        &["/**/"],
    );
    assert_comment_texts(
        &document,
        empty_object.direct_comments.as_ref(),
        &["// empty object"],
    );
    assert_comment_texts(&document, empty_array.direct_comments.as_ref(), &["/**/"]);
    assert!(nested_members[0].value.direct_comments.is_none());
    assert_comment_texts(
        &document,
        nested.tail_comments.as_ref(),
        &["// nested tail"],
    );
    assert_comment_texts(
        &document,
        document.root.tail_comments.as_ref(),
        &["// root internal tail", "/* document tail */"],
    );

    let mut attributed = Vec::new();
    collect_comment_indices(&document.root, &mut attributed);
    attributed.sort_unstable();
    assert_eq!(attributed, (0..document.comments.len()).collect::<Vec<_>>());
    assert_eq!(document.attribution_steps, document.comments.len());
    assert_nonempty_bundles(&document.root);
}

#[test]
fn load_keeps_jsonc_depth_and_comment_evidence_bounded_for_hostile_input() {
    let allowed = format!("{}0{}", "[/* level */".repeat(127), "]".repeat(127));
    let document = load(allowed.as_bytes()).expect("commented depth 127 should be accepted");
    assert_eq!(document.max_depth, 127);
    assert_eq!(document.comments.len(), 127);
    assert_eq!(document.scan_steps, allowed.len());
    assert_eq!(document.attribution_steps, document.comments.len());

    let rejected = format!("{}0{}", "[/* level */".repeat(128), "]".repeat(128));
    assert_eq!(
        load(rejected.as_bytes()),
        Err(LoadError::MaximumDepthExceeded {
            maximum: MAX_DEPTH,
            actual: 128
        })
    );

    let mut wide = String::from("[");
    for index in 0..4_096 {
        if index > 0 {
            wide.push(',');
        }
        wide.push_str("/*x*/0");
    }
    wide.push_str(",]");
    let document = load(wide.as_bytes()).expect("wide comment corpus should load");
    assert_eq!(document.comments.len(), 4_096);
    assert_eq!(document.scan_steps, wide.len());
    assert_eq!(document.attribution_steps, document.comments.len());
    assert!(document.comments.len() * 5 < document.source.len());
}

#[test]
fn load_preserves_order_raw_numbers_and_source_regions() {
    let document = load(MIXED_TREE_FIXTURE.as_bytes()).expect("mixed-tree fixture should load");
    assert!(!document.has_jsonc_syntax);
    assert!(document.comments.is_empty());
    assert!(document.root.direct_comments.is_none());
    assert!(document.root.tail_comments.is_none());
    let JsonValue::Object(members) = &document.root.value else {
        panic!("root should be an object");
    };

    assert_eq!(
        members
            .iter()
            .map(|member| member.name.as_str())
            .collect::<Vec<_>>(),
        [
            "z",
            "a",
            "true-value",
            "null-value",
            "string-value",
            "huge",
            "~",
            "/",
            "",
            "\0control",
            "雪",
            "01",
            "array-context",
            "cross-node",
            "empty-object",
            "empty-array",
            "key-hit",
        ]
    );
    assert_eq!(members[0].name_region.as_str(&document.source), r#""z""#);
    assert_eq!(
        members[0].region.as_str(&document.source),
        r#""z": -0.50e+02"#
    );
    assert_eq!(
        members[0].value.region.as_str(&document.source),
        "-0.50e+02"
    );
    assert_eq!(
        members[0].value.value,
        JsonValue::Number("-0.50e+02".to_owned())
    );
    assert_eq!(
        members[1].name_region.as_str(&document.source),
        r#""\u0061""#
    );
    assert_eq!(
        members[1].region.as_str(&document.source),
        r#""\u0061": [1, {"b": 2}]"#
    );

    let JsonValue::Array(elements) = &members[1].value.value else {
        panic!("second member should contain an array");
    };
    assert_eq!(elements[0].region.as_str(&document.source), "1");
    assert_eq!(elements[1].region.as_str(&document.source), r#"{"b": 2}"#);
    let JsonValue::Object(nested_members) = &elements[1].value else {
        panic!("second array element should be an object");
    };
    assert_eq!(
        nested_members[0].region.as_str(&document.source),
        r#""b": 2"#
    );
    assert_eq!(
        document.root.region.as_str(&document.source),
        document.source
    );
    assert_eq!(
        members[5].value.value,
        JsonValue::Number("1e9999".to_owned())
    );
}

#[test]
fn load_accepts_depth_127_and_rejects_depth_128() {
    // Keep this boundary generated so the maximum and first rejected depth
    // remain explicit instead of hiding them in opaque bracket-only fixtures.
    let allowed = nested_empty_array(127);
    let document = load(allowed.as_bytes()).expect("depth 127 should be accepted");
    assert_eq!(document.max_depth, 127);
    assert_eq!(document.node_count, 128);

    let rejected = nested_empty_array(128);
    assert_eq!(
        load(rejected.as_bytes()),
        Err(LoadError::MaximumDepthExceeded {
            maximum: MAX_DEPTH,
            actual: 128
        })
    );
}

fn nested_empty_array(depth: usize) -> String {
    format!("{}[]{}", "[".repeat(depth), "]".repeat(depth))
}

fn assert_comment_texts(
    document: &JsonDocument,
    bundle: Option<&CommentBundle>,
    expected: &[&str],
) {
    let bundle = bundle.expect("comment bundle should be present");
    assert_eq!(
        bundle
            .indices()
            .iter()
            .map(|&index| document.comments[index].span.as_str(&document.source))
            .collect::<Vec<_>>(),
        expected
    );
}

fn collect_comment_indices(node: &JsonNode, output: &mut Vec<usize>) {
    for bundle in [&node.direct_comments, &node.tail_comments]
        .into_iter()
        .flatten()
    {
        output.extend_from_slice(bundle.indices());
    }
    match &node.value {
        JsonValue::Object(members) => {
            for member in members {
                collect_comment_indices(&member.value, output);
            }
        }
        JsonValue::Array(elements) => {
            for element in elements {
                collect_comment_indices(element, output);
            }
        }
        JsonValue::String(_) | JsonValue::Number(_) | JsonValue::Boolean(_) | JsonValue::Null => {}
    }
}

fn assert_nonempty_bundles(node: &JsonNode) {
    for bundle in [&node.direct_comments, &node.tail_comments]
        .into_iter()
        .flatten()
    {
        assert!(!bundle.indices().is_empty());
        assert!(bundle.indices().windows(2).all(|pair| pair[0] < pair[1]));
    }
    match &node.value {
        JsonValue::Object(members) => {
            for member in members {
                assert_nonempty_bundles(&member.value);
            }
        }
        JsonValue::Array(elements) => {
            for element in elements {
                assert_nonempty_bundles(element);
            }
        }
        JsonValue::String(_) | JsonValue::Number(_) | JsonValue::Boolean(_) | JsonValue::Null => {}
    }
}
