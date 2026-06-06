use std::fs;
use std::path::Path;

use docnav_adapter_sdk::{AdapterError, AdapterResult};
use docnav_protocol::{Entry, StableError};

use super::parse::parse_headings;
use super::refs::{heading_ref, ParsedRef, FULL_DOCUMENT_REF};
use super::text::{cost_for, line_starts, match_display};

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
                let ref_id = containing_heading(&visible_headings, offset)
                    .map(heading_ref)
                    .unwrap_or_else(|| fallback_ref.clone());
                Entry {
                    ref_id,
                    display: match_display(&self.source, &self.line_starts, offset),
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

fn heading_display(document: &MarkdownDocument, heading: &Heading) -> String {
    format!(
        "H{} | {}",
        heading.level,
        cost_for(document.section_content(heading))
    )
}

fn containing_heading<'a>(headings: &[&'a Heading], offset: usize) -> Option<&'a Heading> {
    headings
        .iter()
        .copied()
        .filter(|heading| heading.start <= offset && offset < heading.end)
        .max_by_key(|heading| (heading.start, heading.index))
}

fn read_error(path: &str, error: std::io::Error) -> AdapterError {
    if error.kind() == std::io::ErrorKind::NotFound {
        StableError::document_not_found(path).into()
    } else {
        StableError::document_path_invalid(path, error.to_string()).into()
    }
}
