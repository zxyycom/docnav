use docnav_protocol::{Cost, Measurement};
use docnav_text_cost::{byte_cost, line_cost, token_cost};

pub fn cost_for(content: &str) -> Cost {
    scoped_cost_for(content, "selection")
}

pub fn scoped_cost_for(content: &str, scope: &str) -> Cost {
    Cost {
        measurements: vec![
            scoped(line_cost(content), scope),
            scoped(byte_cost(content), scope),
            scoped(token_cost(content), scope),
        ],
    }
}

fn scoped(mut measurement: Measurement, scope: &str) -> Measurement {
    measurement.scope = Some(scope.to_owned());
    measurement
}

pub(super) fn match_facts(source: &str, line_starts: &[usize], offset: usize) -> (usize, String) {
    let line = line_for_byte(line_starts, offset);
    let (start, end) = line_bounds(line_starts, source.len(), line);
    let snippet = compact_text(&source[start..end]);
    (line, snippet)
}

pub(super) fn line_starts(source: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (index, byte) in source.bytes().enumerate() {
        if byte == b'\n' {
            starts.push(index + 1);
        }
    }
    starts
}

pub(super) fn line_for_byte(starts: &[usize], byte: usize) -> usize {
    match starts.binary_search(&byte) {
        Ok(index) => index + 1,
        Err(index) => index,
    }
    .max(1)
}

pub(super) fn normalize_heading_text(text: &str) -> String {
    let text = compact_text(text);
    if text.is_empty() {
        "(untitled)".to_owned()
    } else {
        text
    }
}

fn line_bounds(starts: &[usize], source_len: usize, line: usize) -> (usize, usize) {
    let start = starts
        .get(line.saturating_sub(1))
        .copied()
        .unwrap_or(source_len);
    let end = starts
        .get(line)
        .copied()
        .unwrap_or(source_len)
        .min(source_len);
    (start, end)
}

fn compact_text(text: &str) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        return ".".to_owned();
    }

    let max_chars = 96;
    let ellipsis = "...";
    if collapsed.chars().count() > max_chars {
        let content_budget = max_chars - ellipsis.chars().count();
        let mut value: String = collapsed.chars().take(content_budget).collect();
        value.push_str(ellipsis);
        value
    } else {
        collapsed
    }
}
