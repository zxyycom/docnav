use std::fs;

use docnav_adapter_sdk::{AdapterError, AdapterResult};
use docnav_protocol::{positive_result, Entry, Location};
use serde_json::json;

use super::parse::parse_headings;
use super::refs::{heading_ref, ParsedRef, FULL_DOCUMENT_REF};
use super::text::{line_starts, match_facts, scoped_cost_for};

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
    pub start: usize,
    pub end: usize,
    pub line: usize,
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
        let source = String::from_utf8(bytes.to_vec())
            .map_err(|_| AdapterError::document_encoding_unsupported(path, "non-utf-8"))?;

        Ok(Self::parse(source))
    }

    pub fn parse(source: String) -> Self {
        let line_starts = line_starts(&source);
        let headings = parse_headings(&source, &line_starts, source.len());

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
            label: "full document".to_owned(),
            kind: Some("document".to_owned()),
            location: None,
            summary: None,
            excerpt: None,
            rank: None,
            cost: Some(scoped_cost_for(self.source(), "entry")),
            metadata: None,
        }
    }

    pub fn outline_entries(&self, max_heading_level: u8) -> Vec<Entry> {
        let mut entries: Vec<Entry> = self
            .headings
            .iter()
            .filter(|heading| heading.level <= max_heading_level)
            .map(|heading| Entry {
                ref_id: heading_ref(heading),
                label: heading.title.clone(),
                kind: Some("heading".to_owned()),
                location: Some(Location {
                    line_start: positive_line(heading.line),
                    line_end: None,
                }),
                summary: None,
                excerpt: None,
                rank: None,
                cost: Some(scoped_cost_for(self.section_content(heading), "entry")),
                metadata: Some(heading_metadata(heading)),
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
                let ref_id = containing_heading(&visible_headings, offset)
                    .map(heading_ref)
                    .unwrap_or_else(|| fallback_ref.clone());
                let (line, label) = match_facts(&self.source, &self.line_starts, offset);
                Entry {
                    ref_id,
                    label,
                    kind: Some("match".to_owned()),
                    location: Some(Location {
                        line_start: positive_line(line),
                        line_end: None,
                    }),
                    summary: None,
                    excerpt: None,
                    rank: None,
                    cost: None,
                    metadata: None,
                }
            })
            .collect()
    }

    /// 按 Markdown adapter 私有契约解析并匹配 ref。
    ///
    /// - `doc:full` → `FullDocument`
    /// - canonical heading ref 无匹配 → `REF_NOT_FOUND`
    /// - 其它非空输入 → `REF_INVALID`
    pub fn resolve_ref(&self, ref_id: &str) -> AdapterResult<ResolvedRef<'_>> {
        if ref_id == FULL_DOCUMENT_REF {
            return Ok(ResolvedRef::FullDocument);
        }

        let Some(parsed) = ParsedRef::parse(ref_id) else {
            return Err(AdapterError::ref_invalid(
                ref_id,
                "expected H:L{line}:H{level} or doc:full",
            ));
        };

        match self.headings.iter().find(|h| parsed.matches(h)) {
            Some(heading) => Ok(ResolvedRef::Heading(heading)),
            None => Err(AdapterError::ref_not_found(ref_id)),
        }
    }

    pub fn section_content(&self, heading: &Heading) -> &str {
        &self.source[heading.start..heading.end]
    }
}

fn containing_heading<'a>(headings: &[&'a Heading], offset: usize) -> Option<&'a Heading> {
    headings
        .iter()
        .copied()
        .filter(|heading| heading.start <= offset && offset < heading.end)
        .max_by_key(|heading| (heading.start, heading.index))
}

fn heading_metadata(heading: &Heading) -> serde_json::Map<String, serde_json::Value> {
    let mut metadata = serde_json::Map::new();
    metadata.insert("heading_level".to_owned(), json!(heading.level));
    metadata
}

fn positive_line(line: usize) -> docnav_protocol::PositiveInteger {
    positive_result(u32::try_from(line).unwrap_or(u32::MAX))
        .expect("markdown line numbers are positive")
}

fn read_error(path: &str, error: std::io::Error) -> AdapterError {
    if error.kind() == std::io::ErrorKind::NotFound {
        AdapterError::document_not_found(path)
    } else {
        AdapterError::document_path_invalid(path, error.to_string())
    }
}
