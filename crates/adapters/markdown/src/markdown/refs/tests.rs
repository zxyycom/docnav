// @case WB-MD-REF-GRAMMAR-001
use super::*;

#[test]
fn canonical_heading_ref_uses_structural_coordinates() {
    let heading = Heading {
        index: 1,
        level: 1,
        title: "长标题测试".into(),
        start: 0,
        end: 0,
        line: 99,
    };
    let ref_id = heading_ref(&heading);
    assert_eq!(ref_id, "H:L99:H1");
}

#[test]
fn parse_canonical_heading_ref() {
    assert_eq!(
        ParsedRef::parse("H:L5:H2"),
        Some(ParsedRef::Heading { line: 5, level: 2 })
    );
}

#[test]
fn parse_rejects_one_representative_per_invalid_grammar_type() {
    for ref_id in [
        "H:Lx:H1",  // 非法字段
        "X:L1:H1",  // 未知 ref 类型
        "H:L01:H1", // 前导零
    ] {
        assert_eq!(ParsedRef::parse(ref_id), None, "{ref_id}");
    }
}

// @case WB-MD-REF-MATCH-001
#[test]
fn matches_exact_line_level() {
    let heading = Heading {
        index: 3,
        level: 2,
        title: "Target".into(),
        start: 0,
        end: 0,
        line: 7,
    };
    let parsed = ParsedRef::parse("H:L7:H2").unwrap();
    assert!(parsed.matches(&heading));
}

#[test]
fn matches_rejects_line_mismatch() {
    let heading = Heading {
        index: 3,
        level: 2,
        title: "Target".into(),
        start: 0,
        end: 0,
        line: 7,
    };
    let parsed = ParsedRef::parse("H:L8:H2").unwrap();
    assert!(!parsed.matches(&heading));
}

#[test]
fn matches_rejects_level_mismatch() {
    let heading = Heading {
        index: 3,
        level: 2,
        title: "Target".into(),
        start: 0,
        end: 0,
        line: 7,
    };
    let parsed = ParsedRef::parse("H:L7:H1").unwrap();
    assert!(!parsed.matches(&heading));
}
