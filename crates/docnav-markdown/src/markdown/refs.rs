use super::document::Heading;

pub const FULL_DOCUMENT_REF: &str = "doc:full";

/// 生成 canonical heading ref: `H:L{line}:H{level}`。
/// `line` 为 1-based canonical 十进制正整数；`level` 为 1-6。
pub(super) fn heading_ref(heading: &Heading) -> String {
    format!("H:L{}:H{}", heading.line, heading.level)
}

/// 解析 canonical heading ref grammar `H:L{line}:H{level}`。
/// grammar 外输入返回 `None`（上游应映射为 `REF_INVALID`）。
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum ParsedRef {
    Heading { line: usize, level: u8 },
}

impl ParsedRef {
    /// 解析 canonical heading ref。
    /// 合法格式：`H:L` + canonical 十进制正整数 + `:H` + 1-6。
    pub(super) fn parse(ref_id: &str) -> Option<Self> {
        let rest = ref_id.strip_prefix("H:L")?;
        let (line_str, level_str) = rest.split_once(":H")?;

        let line = parse_positive_usize(line_str)?;
        let level = parse_heading_level(level_str)?;

        Some(Self::Heading { line, level })
    }

    /// 两字段精确匹配：line、level 全部相同时匹配。
    pub(super) fn matches(&self, heading: &Heading) -> bool {
        match self {
            Self::Heading { line, level } => heading.line == *line && heading.level == *level,
        }
    }
}

fn parse_positive_usize(value: &str) -> Option<usize> {
    if value.starts_with('0') {
        return None;
    }
    value.parse::<usize>().ok().filter(|parsed| *parsed > 0)
}

fn parse_heading_level(value: &str) -> Option<u8> {
    if value.starts_with('0') {
        return None;
    }
    value
        .parse::<u8>()
        .ok()
        .filter(|parsed| (1..=6).contains(parsed))
}

#[cfg(test)]
mod tests {
    // @case WB-MD-REF-GRAMMAR-001
    use super::*;

    #[test]
    fn canonical_heading_ref_format() {
        let heading = Heading {
            index: 3,
            level: 2,
            title: "Install".into(),
            start: 0,
            end: 0,
            line: 5,
        };
        assert_eq!(heading_ref(&heading), "H:L5:H2");
    }

    #[test]
    fn canonical_ref_uses_structural_coordinates() {
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
}
