pub fn cost_for(content: &str) -> String {
    let lines = line_count(content);
    let line_label = if lines == 1 { "line" } else { "lines" };
    format!(
        "{} {} | {:.1} KB",
        lines,
        line_label,
        content.len() as f64 / 1024.0
    )
}

pub(super) fn match_display(source: &str, line_starts: &[usize], offset: usize) -> String {
    let line = line_for_byte(line_starts, offset);
    let (start, end) = line_bounds(line_starts, source.len(), line);
    let snippet = compact_text(&source[start..end]);
    format!("L{}: {}", line, snippet)
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

fn line_count(content: &str) -> usize {
    if content.is_empty() {
        0
    } else {
        content.bytes().filter(|byte| *byte == b'\n').count() + 1
    }
}

fn compact_text(text: &str) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        ".".to_owned()
    } else if collapsed.chars().count() > 96 {
        let mut value: String = collapsed.chars().take(93).collect();
        value.push_str("...");
        value
    } else {
        collapsed
    }
}
