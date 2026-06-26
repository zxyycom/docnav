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
mod tests;
