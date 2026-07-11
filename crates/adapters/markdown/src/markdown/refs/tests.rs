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
fn parse_rejects_leading_zeros() {
    for ref_id in ["H:L01:H2", "H:L1:H02"] {
        assert_eq!(ParsedRef::parse(ref_id), None, "{ref_id}");
    }
}

#[test]
fn parse_rejects_invalid_level() {
    for ref_id in ["H:L1:H0", "H:L1:H7"] {
        assert_eq!(ParsedRef::parse(ref_id), None, "{ref_id}");
    }
}

#[test]
fn parse_rejects_zero_line() {
    assert_eq!(ParsedRef::parse("H:L0:H1"), None);
}

#[test]
fn parse_rejects_non_numeric_fields() {
    for ref_id in ["H:Lx:H1", "H:L1:Hx"] {
        assert_eq!(ParsedRef::parse(ref_id), None, "{ref_id}");
    }
}

#[test]
fn parse_rejects_missing_or_wrong_prefix() {
    for ref_id in [
        // 缺少字段
        "H:L1",
        "H:L1:",
        // 额外字段
        "H:L1:H1:extra",
        // 其他类型
        "X:L1:H1",
        "doc:full",
        "",
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

#[test]
fn doc_full_is_preserved() {
    assert_eq!(FULL_DOCUMENT_REF, "doc:full");
}
