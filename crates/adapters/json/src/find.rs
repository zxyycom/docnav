use std::collections::VecDeque;

use docnav_protocol::{positive_result, Location};

use crate::document::{JsonDocument, JsonNode, JsonValue, SourceRegion};
use crate::reference::{canonical_ref, canonical_ref_for_view, RefView};

const MAX_LABEL_CHARS: usize = 96;
const MAX_CONTEXT_SCALARS_PER_SIDE: usize = MAX_LABEL_CHARS + 1;
const TRUNCATION_MARKER: &str = "...";
const MINIMUM_LABEL: &str = ".";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SourceMatch {
    pub(crate) ref_id: String,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FindEntry {
    pub(crate) ref_id: String,
    pub(crate) label: String,
    pub(crate) location: Location,
}

impl JsonDocument {
    fn source_matches_iter<'a>(&'a self, query: &'a str) -> impl Iterator<Item = SourceMatch> + 'a {
        let mut comment_lookup = CommentRefLookup::new(self);
        self.source
            .match_indices(query)
            .take_while(move |_| !query.is_empty())
            .map(move |(start, matched)| {
                let occurrence = SourceRegion {
                    start,
                    end: start + matched.len(),
                };
                self.source_match(occurrence, &mut comment_lookup)
            })
    }

    #[cfg(test)]
    pub(crate) fn source_matches(&self, query: &str) -> Vec<SourceMatch> {
        self.source_matches_iter(query).collect()
    }

    #[cfg(test)]
    pub(crate) fn source_matches_with_lookup_steps(
        &self,
        query: &str,
    ) -> (Vec<SourceMatch>, usize) {
        let mut comment_lookup = CommentRefLookup::new(self);
        let matches = self
            .source
            .match_indices(query)
            .take_while(|_| !query.is_empty())
            .map(|(start, matched)| {
                self.source_match(
                    SourceRegion {
                        start,
                        end: start + matched.len(),
                    },
                    &mut comment_lookup,
                )
            })
            .collect();

        (matches, comment_lookup.steps)
    }

    pub(crate) fn find_entries<'a>(
        &'a self,
        query: &'a str,
    ) -> impl Iterator<Item = FindEntry> + 'a {
        let line_starts = source_line_starts(&self.source);
        self.source_matches_iter(query).map(move |source_match| {
            let (line, excerpt_region) = match_excerpt_region(
                &self.source,
                &line_starts,
                source_match.start,
                source_match.end,
            );
            let compacted = CompactedExcerpt::new(
                &self.source[excerpt_region.start..excerpt_region.end],
                source_match.start - excerpt_region.start,
                source_match.end - excerpt_region.start,
            );

            FindEntry {
                ref_id: source_match.ref_id,
                label: compacted.label(),
                location: line_location(line),
            }
        })
    }

    fn source_match(
        &self,
        occurrence: SourceRegion,
        comment_lookup: &mut CommentRefLookup,
    ) -> SourceMatch {
        let ref_id = comment_lookup
            .ref_for(occurrence)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| {
                let mut path = Vec::new();
                append_deepest_path(&self.root, occurrence, &mut path);
                let tokens = path.iter().map(String::as_str).collect::<Vec<_>>();
                canonical_ref(&tokens)
            });

        SourceMatch {
            ref_id,
            start: occurrence.start,
            end: occurrence.end,
        }
    }

    fn comment_find_refs(&self) -> Vec<CommentFindRef> {
        let mut refs = (0..self.comments.len()).map(|_| None).collect::<Vec<_>>();
        let mut path = Vec::new();
        append_comment_find_refs(self, &self.root, &mut path, &mut refs);

        refs.into_iter()
            .map(|entry| entry.expect("every JSONC comment has exactly one navigation bundle"))
            .collect()
    }
}

struct CommentFindRef {
    span: SourceRegion,
    ref_id: String,
}

struct CommentRefLookup {
    refs: Vec<CommentFindRef>,
    cursor: usize,
    #[cfg(test)]
    steps: usize,
}

impl CommentRefLookup {
    fn new(document: &JsonDocument) -> Self {
        Self {
            refs: document.comment_find_refs(),
            cursor: 0,
            #[cfg(test)]
            steps: 0,
        }
    }

    fn ref_for(&mut self, occurrence: SourceRegion) -> Option<&str> {
        while self
            .refs
            .get(self.cursor)
            .is_some_and(|entry| entry.span.end <= occurrence.start)
        {
            self.cursor += 1;
            #[cfg(test)]
            {
                self.steps += 1;
            }
        }
        #[cfg(test)]
        {
            self.steps += 1;
        }
        self.refs
            .get(self.cursor)
            .filter(|entry| covers(entry.span, occurrence))
            .map(|entry| entry.ref_id.as_str())
    }
}

fn append_comment_find_refs(
    document: &JsonDocument,
    node: &JsonNode,
    path: &mut Vec<String>,
    refs: &mut [Option<CommentFindRef>],
) {
    append_comment_bundle_refs(
        document,
        node.direct_comments.as_ref(),
        RefView::DirectComments,
        path,
        refs,
    );
    match &node.value {
        JsonValue::Object(members) => {
            for member in members {
                path.push(member.name.clone());
                append_comment_find_refs(document, &member.value, path, refs);
                path.pop();
            }
        }
        JsonValue::Array(elements) => {
            for (index, element) in elements.iter().enumerate() {
                path.push(index.to_string());
                append_comment_find_refs(document, element, path, refs);
                path.pop();
            }
        }
        JsonValue::String(_) | JsonValue::Number(_) | JsonValue::Boolean(_) | JsonValue::Null => {}
    }
    append_comment_bundle_refs(
        document,
        node.tail_comments.as_ref(),
        RefView::TailComments,
        path,
        refs,
    );
}

fn append_comment_bundle_refs(
    document: &JsonDocument,
    bundle: Option<&crate::document::CommentBundle>,
    view: RefView,
    path: &[String],
    refs: &mut [Option<CommentFindRef>],
) {
    let Some(bundle) = bundle else {
        return;
    };
    let tokens = path.iter().map(String::as_str).collect::<Vec<_>>();
    let ref_id = canonical_ref_for_view(view, &tokens);
    for &index in bundle.indices() {
        refs[index] = Some(CommentFindRef {
            span: document.comments[index].span,
            ref_id: ref_id.clone(),
        });
    }
}

fn append_deepest_path(node: &JsonNode, occurrence: SourceRegion, path: &mut Vec<String>) {
    match &node.value {
        JsonValue::Object(members) => {
            let index = members.partition_point(|member| member.region.end <= occurrence.start);
            if let Some(member) = members
                .get(index)
                .filter(|member| covers(member.region, occurrence))
            {
                path.push(member.name.clone());
                append_deepest_path(&member.value, occurrence, path);
            }
        }
        JsonValue::Array(elements) => {
            let index = elements.partition_point(|element| element.region.end <= occurrence.start);
            if let Some(element) = elements
                .get(index)
                .filter(|element| covers(element.region, occurrence))
            {
                path.push(index.to_string());
                append_deepest_path(element, occurrence, path);
            }
        }
        JsonValue::String(_) | JsonValue::Number(_) | JsonValue::Boolean(_) | JsonValue::Null => {}
    }
}

fn covers(region: SourceRegion, occurrence: SourceRegion) -> bool {
    region.start <= occurrence.start && occurrence.end <= region.end
}

fn source_line_starts(source: &str) -> Vec<usize> {
    let mut starts = vec![0];
    starts.extend(
        source
            .as_bytes()
            .iter()
            .enumerate()
            .filter_map(|(offset, &byte)| (byte == b'\n').then_some(offset + 1)),
    );
    starts
}

fn match_excerpt_region(
    source: &str,
    line_starts: &[usize],
    match_start: usize,
    match_end: usize,
) -> (usize, SourceRegion) {
    let line = line_starts
        .partition_point(|&line_start| line_start <= match_start)
        .saturating_sub(1);
    let line_start = line_starts[line];
    let line_end = if source.as_bytes().get(match_end.saturating_sub(1)) == Some(&b'\n') {
        match_end
    } else {
        let next_line = line_starts.partition_point(|&line_start| line_start <= match_end);
        line_starts
            .get(next_line)
            .map_or(source.len(), |line_start| line_start - 1)
    };

    (
        line,
        SourceRegion {
            start: line_start,
            end: line_end,
        },
    )
}

#[derive(Debug)]
struct CompactedExcerpt {
    collapsed_chars: usize,
    occurrence_start: usize,
    occurrence_end: usize,
    first_chars: BufferedChars,
    last_chars: BufferedChars,
    before_occurrence: BufferedChars,
    from_occurrence: BufferedChars,
}

impl CompactedExcerpt {
    fn new(source: &str, match_start: usize, match_end: usize) -> Self {
        let context_start = bounded_context_start(source, match_start);
        let context_end = bounded_context_end(source, match_end);
        let source = &source[context_start..context_end];
        let match_start = match_start - context_start;
        let match_end = match_end - context_start;
        let mut excerpt = Self {
            collapsed_chars: 0,
            occurrence_start: 0,
            occurrence_end: 0,
            first_chars: BufferedChars::default(),
            last_chars: BufferedChars::default(),
            before_occurrence: BufferedChars::default(),
            from_occurrence: BufferedChars::default(),
        };
        let mut first_span_start = None;
        let mut first_overlap = None;

        visit_compacted_chars(source, |character, span| {
            let index = excerpt.collapsed_chars;
            excerpt.collapsed_chars += 1;
            first_span_start.get_or_insert(span.start);
            excerpt.first_chars.push_first(index, character);
            excerpt.last_chars.push_last(index, character);

            let overlaps = span.end > match_start && span.start < match_end;
            if overlaps {
                first_overlap.get_or_insert(index);
                excerpt.occurrence_end = index + 1;
            } else if first_overlap.is_none() {
                excerpt.before_occurrence.push_last(index, character);
            }

            if first_overlap.is_some() {
                excerpt.from_occurrence.push_first(index, character);
            }
        });

        if let Some(first_overlap) = first_overlap {
            excerpt.occurrence_start = first_overlap;
        } else if first_span_start.is_none_or(|span_start| span_start >= match_end) {
            excerpt.occurrence_start = 0;
            excerpt.occurrence_end = 0;
        } else {
            excerpt.occurrence_start = excerpt.collapsed_chars;
            excerpt.occurrence_end = excerpt.collapsed_chars;
        }

        excerpt
    }

    fn label(&self) -> String {
        if self.collapsed_chars == 0 {
            return MINIMUM_LABEL.to_owned();
        }
        if self.collapsed_chars <= MAX_LABEL_CHARS {
            return self.text(0, self.collapsed_chars);
        }

        self.bounded_label()
    }

    fn bounded_label(&self) -> String {
        let marker_chars = TRUNCATION_MARKER.chars().count();
        let centered_budget = MAX_LABEL_CHARS - marker_chars * 2;
        let one_sided_budget = MAX_LABEL_CHARS - marker_chars;
        let occurrence_chars = self.occurrence_end - self.occurrence_start;
        let context_before = centered_budget.saturating_sub(occurrence_chars) / 2;
        let mut window_start = self
            .occurrence_start
            .saturating_sub(context_before)
            .min(self.collapsed_chars - centered_budget);
        let mut window_end = window_start + centered_budget;

        if occurrence_chars <= centered_budget && self.occurrence_end > window_end {
            window_end = self.occurrence_end;
            window_start = window_end - centered_budget;
        }
        if window_start == 0 {
            window_end = one_sided_budget;
        } else if window_end == self.collapsed_chars {
            window_start = self.collapsed_chars - one_sided_budget;
        }

        let mut label = String::new();
        if window_start > 0 {
            label.push_str(TRUNCATION_MARKER);
        }
        label.push_str(&self.text(window_start, window_end));
        if window_end < self.collapsed_chars {
            label.push_str(TRUNCATION_MARKER);
        }
        label
    }

    fn text(&self, start: usize, end: usize) -> String {
        (start..end)
            .map(|index| {
                self.first_chars
                    .get(index)
                    .or_else(|| self.last_chars.get(index))
                    .or_else(|| self.before_occurrence.get(index))
                    .or_else(|| self.from_occurrence.get(index))
                    .expect("bounded excerpt buffers cover the selected label window")
            })
            .collect()
    }

    #[cfg(test)]
    fn buffered_chars(&self) -> usize {
        self.first_chars.len()
            + self.last_chars.len()
            + self.before_occurrence.len()
            + self.from_occurrence.len()
    }
}

fn bounded_context_start(source: &str, match_start: usize) -> usize {
    source[..match_start]
        .char_indices()
        .rev()
        .nth(MAX_CONTEXT_SCALARS_PER_SIDE - 1)
        .map_or(0, |(offset, _)| offset)
}

fn bounded_context_end(source: &str, match_end: usize) -> usize {
    source[match_end..]
        .char_indices()
        .nth(MAX_CONTEXT_SCALARS_PER_SIDE - 1)
        .map_or(source.len(), |(offset, character)| {
            match_end + offset + character.len_utf8()
        })
}

#[derive(Debug, Default)]
struct BufferedChars {
    start: usize,
    chars: VecDeque<char>,
}

impl BufferedChars {
    fn push_first(&mut self, index: usize, character: char) {
        if self.chars.len() == MAX_LABEL_CHARS {
            return;
        }
        if self.chars.is_empty() {
            self.start = index;
        }
        self.chars.push_back(character);
    }

    fn push_last(&mut self, index: usize, character: char) {
        if self.chars.is_empty() {
            self.start = index;
        } else if self.chars.len() == MAX_LABEL_CHARS {
            self.chars.pop_front();
            self.start += 1;
        }
        self.chars.push_back(character);
    }

    fn get(&self, index: usize) -> Option<char> {
        index
            .checked_sub(self.start)
            .and_then(|offset| self.chars.get(offset))
            .copied()
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.chars.len()
    }
}

fn visit_compacted_chars(source: &str, mut visit: impl FnMut(char, SourceRegion)) {
    let mut has_text = false;
    let mut pending_whitespace: Option<SourceRegion> = None;

    for (offset, character) in source.char_indices() {
        let character_end = offset + character.len_utf8();
        if character.is_whitespace() {
            if has_text {
                match pending_whitespace.as_mut() {
                    Some(region) => region.end = character_end,
                    None => {
                        pending_whitespace = Some(SourceRegion {
                            start: offset,
                            end: character_end,
                        });
                    }
                }
            }
            continue;
        }

        if let Some(region) = pending_whitespace.take() {
            visit(' ', region);
        }
        visit(
            character,
            SourceRegion {
                start: offset,
                end: character_end,
            },
        );
        has_text = true;
    }
}

fn line_location(line: usize) -> Location {
    let line = line.saturating_add(1);
    let line = u32::try_from(line).unwrap_or(u32::MAX);

    Location {
        line_start: positive_result(line).expect("source line numbers are positive"),
        line_end: None,
    }
}

#[cfg(test)]
mod tests;
