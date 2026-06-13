use super::document::Heading;

pub const FULL_DOCUMENT_REF: &str = "doc:full";

/// 生成 canonical heading ref: `H:L{line}:H{level}:I{index}`
/// `line` 和 `index` 为 1-based 十进制正整数，无前导零；`level` 为 1-6。
pub(super) fn heading_ref(heading: &Heading) -> String {
    format!("H:L{}:H{}:I{}", heading.line, heading.level, heading.index)
}

/// 解析 canonical heading ref grammar `H:L{line}:H{level}:I{index}`。
/// 不符合 grammar 时返回 `None`（上游应映射为 `REF_INVALID`）。
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum ParsedRef {
    Heading {
        line: usize,
        level: u8,
        index: usize,
    },
}

impl ParsedRef {
    /// 解析 canonical heading ref。
    /// 合法格式：`H:L` + 正整数无前导零 + `:H` + 1-6 + `:I` + 正整数无前导零。
    pub(super) fn parse(ref_id: &str) -> Option<Self> {
        let rest = ref_id.strip_prefix("H:L")?;
        let (line_str, rest) = rest.split_once(":H")?;
        let (level_str, index_str) = rest.split_once(":I")?;

        // 拒绝前导零
        if line_str.starts_with('0') || level_str.starts_with('0') || index_str.starts_with('0') {
            return None;
        }

        let line = line_str.parse::<usize>().ok()?;
        let level = level_str.parse::<u8>().ok()?;
        let index = index_str.parse::<usize>().ok()?;

        // 正整数 + level 1-6
        if line == 0 || level == 0 || level > 6 || index == 0 {
            return None;
        }

        Some(Self::Heading { line, level, index })
    }

    /// 三字段精确匹配：line、level、index 全部相同时匹配。
    pub(super) fn matches(&self, heading: &Heading) -> bool {
        match self {
            Self::Heading { line, level, index } => {
                heading.line == *line && heading.level == *level && heading.index == *index
            }
        }
    }
}

#[cfg(test)]
mod tests {
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
        assert_eq!(heading_ref(&heading), "H:L5:H2:I3");
    }

    #[test]
    fn canonical_ref_does_not_include_title_or_breadcrumb() {
        let heading = Heading {
            index: 1,
            level: 1,
            title: "长标题测试".into(),
            start: 0,
            end: 0,
            line: 99,
        };
        let ref_id = heading_ref(&heading);
        assert_eq!(ref_id, "H:L99:H1:I1");
        assert!(!ref_id.contains("长标题"));
        assert!(!ref_id.contains("A > B"));
    }

    #[test]
    fn parse_canonical_heading_ref() {
        assert_eq!(
            ParsedRef::parse("H:L5:H2:I3"),
            Some(ParsedRef::Heading {
                line: 5,
                level: 2,
                index: 3,
            })
        );
    }

    #[test]
    fn parse_rejects_leading_zeros() {
        for ref_id in ["H:L01:H2:I1", "H:L1:H02:I1", "H:L1:H2:I01"] {
            assert_eq!(ParsedRef::parse(ref_id), None, "{ref_id}");
        }
    }

    #[test]
    fn parse_rejects_invalid_level() {
        for ref_id in ["H:L1:H0:I1", "H:L1:H7:I1"] {
            assert_eq!(ParsedRef::parse(ref_id), None, "{ref_id}");
        }
    }

    #[test]
    fn parse_rejects_zero_line_or_index() {
        for ref_id in ["H:L0:H1:I1", "H:L1:H1:I0"] {
            assert_eq!(ParsedRef::parse(ref_id), None, "{ref_id}");
        }
    }

    #[test]
    fn parse_rejects_non_numeric_fields() {
        for ref_id in ["H:Lx:H1:I1", "H:L1:Hx:I1", "H:L1:H1:Ix"] {
            assert_eq!(ParsedRef::parse(ref_id), None, "{ref_id}");
        }
    }

    #[test]
    fn parse_rejects_missing_or_wrong_prefix() {
        for ref_id in [
            // 旧格式
            "L1:Guide",
            "L5#2:Guide > Install",
            "L2#1:A > B",
            // 缺少字段
            "H:L1:H2",
            "H:L1",
            "H:L1:",
            // 其他类型
            "X:L1:H1:I1",
            "doc:full",
            "",
        ] {
            assert_eq!(ParsedRef::parse(ref_id), None, "{ref_id}");
        }
    }

    #[test]
    fn matches_exact_line_level_index() {
        let heading = Heading {
            index: 3,
            level: 2,
            title: "Target".into(),
            start: 0,
            end: 0,
            line: 7,
        };
        let parsed = ParsedRef::parse("H:L7:H2:I3").unwrap();
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
        let parsed = ParsedRef::parse("H:L8:H2:I3").unwrap();
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
        let parsed = ParsedRef::parse("H:L7:H1:I3").unwrap();
        assert!(!parsed.matches(&heading));
    }

    #[test]
    fn matches_rejects_index_mismatch() {
        let heading = Heading {
            index: 3,
            level: 2,
            title: "Target".into(),
            start: 0,
            end: 0,
            line: 7,
        };
        let parsed = ParsedRef::parse("H:L7:H2:I4").unwrap();
        assert!(!parsed.matches(&heading));
    }

    #[test]
    fn doc_full_is_preserved() {
        assert_eq!(FULL_DOCUMENT_REF, "doc:full");
    }
}
