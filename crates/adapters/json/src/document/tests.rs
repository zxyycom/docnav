use super::*;

const MIXED_TREE_FIXTURE: &str = include_str!("../../tests/fixtures/mixed-tree.json");
const INVALID_INPUTS_FIXTURE: &str = include_str!("../../tests/fixtures/invalid-inputs.txt");

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
fn load_preserves_order_raw_numbers_and_source_regions() {
    let document = load(MIXED_TREE_FIXTURE.as_bytes()).expect("mixed-tree fixture should load");
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
