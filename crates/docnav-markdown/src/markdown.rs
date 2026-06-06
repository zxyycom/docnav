use std::collections::HashMap;
use std::fs;
use std::path::Path;

use docnav_adapter_sdk::{AdapterError, AdapterResult};
use docnav_protocol::{Entry, StableError};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::adapter::DEFAULT_MAX_HEADING_LEVEL;

pub const FULL_DOCUMENT_REF: &str = "doc:full";

#[derive(Clone, Debug)]
pub struct MarkdownDocument {
    source: String,
    line_starts: Vec<usize>,
    headings: Vec<Heading>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Heading {
    pub index: usize,
    pub level: u8,
    pub title: String,
    pub path: String,
    pub path_occurrence: usize,
    pub start: usize,
    pub end: usize,
    pub line: usize,
}

#[derive(Clone, Debug)]
struct RawHeading {
    level: u8,
    title: String,
    start: usize,
}

#[derive(Clone, Debug)]
struct OpenHeading {
    level: u8,
    start: usize,
    text: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolvedRef<'a> {
    FullDocument,
    Heading(&'a Heading),
}

impl MarkdownDocument {
    pub fn load(path: &str) -> AdapterResult<Self> {
        let bytes = fs::read(path).map_err(|error| read_error(path, error))?;
        let bytes = bytes
            .strip_prefix(&[0xEF, 0xBB, 0xBF])
            .unwrap_or(bytes.as_slice());
        let source = String::from_utf8(bytes.to_vec()).map_err(|_| {
            AdapterError::from(StableError::document_encoding_unsupported(
                path,
                "non-utf-8",
            ))
        })?;

        Ok(Self::parse(source))
    }

    pub fn parse(source: String) -> Self {
        let line_starts = line_starts(&source);
        let raw_headings = parse_headings(&source);
        let mut headings = materialize_headings(raw_headings, &line_starts, source.len());
        assign_section_ends(&mut headings, source.len());

        Self {
            source,
            line_starts,
            headings,
        }
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn headings(&self) -> &[Heading] {
        &self.headings
    }

    pub fn full_entry(&self) -> Entry {
        Entry {
            ref_id: FULL_DOCUMENT_REF.to_owned(),
            display: format!("full document | {}", cost_for(self.source())),
        }
    }

    pub fn outline_entries(&self, max_heading_level: u8) -> Vec<Entry> {
        let mut entries: Vec<Entry> = self
            .headings
            .iter()
            .filter(|heading| heading.level <= max_heading_level)
            .map(|heading| Entry {
                ref_id: heading_ref(heading),
                display: heading_display(self, heading),
            })
            .collect();

        if entries.is_empty() {
            entries.push(self.full_entry());
        }

        entries
    }

    pub fn find_entries(&self, query: &str, max_heading_level: u8) -> Vec<Entry> {
        let visible_headings: Vec<&Heading> = self
            .headings
            .iter()
            .filter(|heading| heading.level <= max_heading_level)
            .collect();
        let fallback_ref = self.full_entry().ref_id;

        self.source
            .match_indices(query)
            .map(|(offset, _)| {
                let ref_id = nearest_heading(&visible_headings, offset)
                    .map(heading_ref)
                    .unwrap_or_else(|| fallback_ref.clone());
                Entry {
                    ref_id,
                    display: match_display(self, offset),
                }
            })
            .collect()
    }

    pub fn resolve_ref(&self, ref_id: &str) -> AdapterResult<ResolvedRef<'_>> {
        if ref_id == FULL_DOCUMENT_REF {
            return Ok(ResolvedRef::FullDocument);
        }

        let Some(parsed) = ParsedRef::parse(ref_id) else {
            return Err(StableError::ref_not_found(ref_id).into());
        };

        let candidates: Vec<&Heading> = self
            .headings
            .iter()
            .filter(|heading| parsed.matches(heading))
            .collect();

        match candidates.as_slice() {
            [] => Err(StableError::ref_not_found(ref_id).into()),
            [heading] => Ok(ResolvedRef::Heading(heading)),
            many => Err(StableError::ref_ambiguous(ref_id, many.len() as u32).into()),
        }
    }

    pub fn section_content(&self, heading: &Heading) -> &str {
        &self.source[heading.start..heading.end]
    }
}

pub fn max_heading_level_from_options(
    options: Option<&docnav_protocol::Options>,
) -> AdapterResult<u8> {
    let Some(options) = options else {
        return Ok(DEFAULT_MAX_HEADING_LEVEL);
    };
    let Some(value) = options.get("max_heading_level") else {
        return Ok(DEFAULT_MAX_HEADING_LEVEL);
    };
    let Some(level) = value.as_u64() else {
        return Err(StableError::invalid_request(
            "arguments.options.max_heading_level",
            "must be an integer from 1 to 6",
        )
        .into());
    };
    if !(1..=6).contains(&level) {
        return Err(StableError::invalid_request(
            "arguments.options.max_heading_level",
            "must be an integer from 1 to 6",
        )
        .into());
    }
    Ok(level as u8)
}

pub fn heading_ref(heading: &Heading) -> String {
    if heading.path_occurrence == 1 {
        format!("L{}:{}", heading.line, heading.path)
    } else {
        format!(
            "L{}#{}:{}",
            heading.line, heading.path_occurrence, heading.path
        )
    }
}

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

pub fn is_markdown_extension(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            let extension = extension.to_ascii_lowercase();
            extension == "md" || extension == "markdown"
        })
        .unwrap_or(false)
}

pub fn is_utf8_markdown_candidate(path: &str) -> Result<bool, std::io::Error> {
    let bytes = fs::read(path)?;
    let bytes = bytes
        .strip_prefix(&[0xEF, 0xBB, 0xBF])
        .unwrap_or(bytes.as_slice());
    Ok(std::str::from_utf8(bytes).is_ok())
}

fn parse_headings(source: &str) -> Vec<RawHeading> {
    let parser = Parser::new_ext(source, Options::empty()).into_offset_iter();
    let frontmatter_end = frontmatter_end(source);
    let mut headings = Vec::new();
    let mut current: Option<OpenHeading> = None;

    for (event, range) in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                current = (range.start >= frontmatter_end).then(|| OpenHeading {
                    level: heading_level(level),
                    start: range.start,
                    text: String::new(),
                });
            }
            Event::Text(text) | Event::Code(text) => {
                if let Some(heading) = &mut current {
                    heading.text.push_str(&text);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some(heading) = &mut current {
                    heading.text.push(' ');
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some(heading) = current.take() {
                    headings.push(RawHeading {
                        level: heading.level,
                        title: normalize_heading_text(&heading.text),
                        start: heading.start,
                    });
                }
            }
            _ => {}
        }
    }

    headings
}

fn materialize_headings(
    raw_headings: Vec<RawHeading>,
    line_starts: &[usize],
    source_len: usize,
) -> Vec<Heading> {
    let mut stack: Vec<Option<String>> = vec![None; 6];
    let mut path_counts: HashMap<String, usize> = HashMap::new();
    let mut headings = Vec::with_capacity(raw_headings.len());

    for (index, raw) in raw_headings.into_iter().enumerate() {
        let level_index = raw.level.saturating_sub(1) as usize;
        stack.truncate(level_index);
        stack.resize(level_index + 1, None);
        stack[level_index] = Some(raw.title.clone());

        let path = stack
            .iter()
            .filter_map(|part| part.as_deref())
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" > ");
        let path = if path.is_empty() {
            "(untitled)".to_owned()
        } else {
            path
        };
        let path_occurrence = path_counts.entry(path.clone()).or_insert(0);
        *path_occurrence += 1;

        headings.push(Heading {
            index: index + 1,
            level: raw.level,
            title: raw.title,
            path,
            path_occurrence: *path_occurrence,
            start: raw.start.min(source_len),
            end: source_len,
            line: line_for_byte(line_starts, raw.start),
        });
    }

    headings
}

fn assign_section_ends(headings: &mut [Heading], source_len: usize) {
    for index in 0..headings.len() {
        let level = headings[index].level;
        let end = headings
            .iter()
            .skip(index + 1)
            .find(|candidate| candidate.level <= level)
            .map(|candidate| candidate.start)
            .unwrap_or(source_len);
        headings[index].end = end;
    }
}

fn heading_display(document: &MarkdownDocument, heading: &Heading) -> String {
    format!(
        "H{} | {}",
        heading.level,
        cost_for(document.section_content(heading))
    )
}

fn frontmatter_end(source: &str) -> usize {
    let Some(first_line_end) = source.find('\n') else {
        return 0;
    };
    if source[..first_line_end].trim_end_matches('\r') != "---" {
        return 0;
    }

    let mut offset = first_line_end + 1;
    while offset < source.len() {
        let line_end = source[offset..]
            .find('\n')
            .map(|relative| offset + relative)
            .unwrap_or(source.len());
        let line = source[offset..line_end].trim_end_matches('\r');
        let next_offset = (line_end + 1).min(source.len());
        if line == "---" || line == "..." {
            return next_offset;
        }
        offset = next_offset;
    }

    0
}

fn nearest_heading<'a>(headings: &[&'a Heading], offset: usize) -> Option<&'a Heading> {
    headings.iter().copied().min_by_key(|heading| {
        let distance = heading.start.abs_diff(offset);
        let follows_match = usize::from(heading.start > offset);
        (distance, follows_match, heading.start)
    })
}

fn match_display(document: &MarkdownDocument, offset: usize) -> String {
    let line = line_for_byte(&document.line_starts, offset);
    let (start, end) = line_bounds(&document.line_starts, document.source.len(), line);
    let snippet = compact_text(&document.source[start..end]);
    format!("L{}: {}", line, snippet)
}

fn line_starts(source: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (index, byte) in source.bytes().enumerate() {
        if byte == b'\n' {
            starts.push(index + 1);
        }
    }
    starts
}

fn line_for_byte(starts: &[usize], byte: usize) -> usize {
    match starts.binary_search(&byte) {
        Ok(index) => index + 1,
        Err(index) => index,
    }
    .max(1)
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

fn normalize_heading_text(text: &str) -> String {
    let text = compact_text(text);
    if text.is_empty() {
        "(untitled)".to_owned()
    } else {
        text
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

fn heading_level(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn read_error(path: &str, error: std::io::Error) -> AdapterError {
    if error.kind() == std::io::ErrorKind::NotFound {
        StableError::document_not_found(path).into()
    } else {
        StableError::document_path_invalid(path, error.to_string()).into()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ParsedRef {
    Heading {
        line: usize,
        path: String,
        occurrence: usize,
    },
}

impl ParsedRef {
    fn parse(ref_id: &str) -> Option<Self> {
        let rest = ref_id.strip_prefix('L')?;
        let (prefix, path) = rest.split_once(':')?;
        let (line, occurrence) = match prefix.split_once('#') {
            Some((line, occurrence)) => (line, occurrence.parse::<usize>().ok()?),
            None => (prefix, 1),
        };
        let line = line.parse::<usize>().ok()?;

        if line == 0 || path.is_empty() || occurrence == 0 {
            return None;
        }

        Some(Self::Heading {
            line,
            path: path.to_owned(),
            occurrence,
        })
    }

    fn matches(&self, heading: &Heading) -> bool {
        match self {
            Self::Heading {
                line,
                path,
                occurrence,
            } => {
                heading.line == *line
                    && heading.path == *path
                    && heading.path_occurrence == *occurrence
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_ignores_code_fence_pseudo_heading_and_invalid_heading() {
        let document = MarkdownDocument::parse(
            "# Real\n\n```\n# Not real\n```\n\n#NoSpace\n\n## Child\n".to_owned(),
        );

        let titles: Vec<&str> = document
            .headings()
            .iter()
            .map(|heading| heading.title.as_str())
            .collect();
        assert_eq!(titles, vec!["Real", "Child"]);
    }

    #[test]
    fn duplicate_full_paths_receive_unique_refs() {
        let document = MarkdownDocument::parse("# A\n## B\n# A\n## B\n".to_owned());

        let entries = document.outline_entries(3);
        let duplicate_refs: Vec<&str> = entries
            .iter()
            .filter(|entry| entry.ref_id.contains("A > B"))
            .map(|entry| entry.ref_id.as_str())
            .collect();

        assert_eq!(duplicate_refs, vec!["L2:A > B", "L4#2:A > B"]);
    }

    #[test]
    fn heading_refs_omit_default_occurrence() {
        let document = MarkdownDocument::parse("# Guide\n\n## Install\n".to_owned());
        let entries = document.outline_entries(3);
        let refs: Vec<&str> = entries.iter().map(|entry| entry.ref_id.as_str()).collect();

        assert_eq!(refs, vec!["L1:Guide", "L3:Guide > Install"]);
    }

    #[test]
    fn explicit_default_occurrence_resolves_first_heading() {
        let document = MarkdownDocument::parse("# Guide\nOne\n# Guide\nTwo\n".to_owned());

        let resolved = document.resolve_ref("L1#1:Guide").unwrap();

        assert_eq!(resolved, ResolvedRef::Heading(&document.headings()[0]));
    }

    #[test]
    fn canonical_duplicate_occurrence_resolves_matching_heading() {
        let document = MarkdownDocument::parse("# Repeat\nOne\n# Repeat\nTwo\n".to_owned());

        let resolved = document.resolve_ref("L3#2:Repeat").unwrap();

        assert_eq!(resolved, ResolvedRef::Heading(&document.headings()[1]));
    }

    #[test]
    fn old_bracketed_occurrence_suffix_is_path_text_not_metadata() {
        let document = MarkdownDocument::parse("# Guide\nBody\n".to_owned());
        let suffix = [" [", "docnav", ":", "1", "]"].concat();
        let old_ref = format!("L1:Guide{suffix}");

        assert_eq!(
            ParsedRef::parse(&old_ref),
            Some(ParsedRef::Heading {
                line: 1,
                path: format!("Guide{suffix}"),
                occurrence: 1,
            })
        );
        assert!(document.resolve_ref(&old_ref).is_err());
    }

    #[test]
    fn heading_path_may_end_with_legacy_marker_like_text() {
        let suffix = [" [", "docnav", ":", "1", "]"].concat();
        let title = format!("Guide{suffix}");
        let document = MarkdownDocument::parse(format!("# {title}\nBody\n"));
        let entries = document.outline_entries(3);

        assert_eq!(entries[0].ref_id, format!("L1:{title}"));
        assert_eq!(
            document.resolve_ref(&entries[0].ref_id).unwrap(),
            ResolvedRef::Heading(&document.headings()[0])
        );
    }

    #[test]
    fn parsed_ref_accepts_extra_colons_in_path() {
        assert_eq!(
            ParsedRef::parse("L7#2:Guide: Install"),
            Some(ParsedRef::Heading {
                line: 7,
                path: "Guide: Install".to_owned(),
                occurrence: 2,
            })
        );
    }

    #[test]
    fn parsed_ref_rejects_invalid_line_ordinal_and_path() {
        for ref_id in [
            "L0:Guide",
            "Lx:Guide",
            "L1#:Guide",
            "L1#0:Guide",
            "L1#x:Guide",
            "L1:",
        ] {
            assert_eq!(ParsedRef::parse(ref_id), None, "{ref_id}");
        }
    }

    #[test]
    fn doc_full_still_resolves_to_full_document() {
        let document = MarkdownDocument::parse("# Guide\nBody\n".to_owned());

        assert_eq!(
            document.resolve_ref(FULL_DOCUMENT_REF).unwrap(),
            ResolvedRef::FullDocument
        );
    }

    #[test]
    fn deep_heading_can_be_filtered_to_full_document() {
        let document = MarkdownDocument::parse("Intro\n\n#### Deep\nBody\n".to_owned());

        let entries = document.outline_entries(3);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].ref_id, FULL_DOCUMENT_REF);
    }

    #[test]
    fn read_section_ends_at_next_same_or_higher_heading() {
        let document = MarkdownDocument::parse("# A\nIntro\n## B\nNested\n# C\nEnd\n".to_owned());
        let heading = &document.headings()[0];

        assert_eq!(
            document.section_content(heading),
            "# A\nIntro\n## B\nNested\n"
        );
    }

    #[test]
    fn frontmatter_does_not_create_outline_heading() {
        let document = MarkdownDocument::parse("---\ntitle: Sample\n---\n\n# Real\n".to_owned());

        assert_eq!(document.headings().len(), 1);
        assert_eq!(document.headings()[0].title, "Real");
    }
}
