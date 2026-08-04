#[cfg(test)]
use std::cell::Cell;
use std::collections::HashSet;
use std::fmt;

use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::Deserialize;
use serde_json::value::RawValue;

#[cfg(test)]
pub(crate) use crate::jsonc::CommentKind;
pub(crate) use crate::jsonc::CommentToken;

pub(crate) const MAX_DEPTH: u8 = 127;
const UTF8_BOM: &[u8] = b"\xef\xbb\xbf";
#[cfg(test)]
pub(crate) const WIDE_COMMENT_ITEM_COUNT: usize = 1_024;

#[cfg(test)]
pub(crate) fn wide_comment_per_item_source() -> String {
    let mut source = String::from("[");
    for index in 0..WIDE_COMMENT_ITEM_COUNT {
        source.push_str("\n  /* item-");
        source.push_str(&index.to_string());
        source.push_str(" */ ");
        source.push_str(&index.to_string());
        source.push(',');
    }
    source.push_str("\n]");
    source
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SourceRegion {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

#[cfg(test)]
impl SourceRegion {
    pub(crate) fn as_str(self, source: &str) -> &str {
        &source[self.start..self.end]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum JsonKind {
    Object,
    Array,
    String,
    Number,
    Boolean,
    Null,
}

#[derive(Debug, PartialEq)]
pub(crate) struct JsonNode {
    pub(crate) depth: u8,
    pub(crate) region: SourceRegion,
    pub(crate) value: JsonValue,
    pub(crate) direct_comments: Option<CommentBundle>,
    pub(crate) tail_comments: Option<CommentBundle>,
}

impl JsonNode {
    pub(crate) fn kind(&self) -> JsonKind {
        match &self.value {
            JsonValue::Object(_) => JsonKind::Object,
            JsonValue::Array(_) => JsonKind::Array,
            JsonValue::String(_) => JsonKind::String,
            JsonValue::Number(_) => JsonKind::Number,
            JsonValue::Boolean(_) => JsonKind::Boolean,
            JsonValue::Null => JsonKind::Null,
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum JsonValue {
    Object(Vec<JsonMember>),
    Array(Vec<JsonNode>),
    String(String),
    Number(String),
    Boolean(bool),
    Null,
}

#[derive(Debug, PartialEq)]
pub(crate) struct JsonMember {
    pub(crate) name: String,
    pub(crate) name_region: SourceRegion,
    pub(crate) region: SourceRegion,
    pub(crate) value: JsonNode,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CommentBundle {
    indices: Vec<usize>,
}

impl CommentBundle {
    pub(crate) fn indices(&self) -> &[usize] {
        &self.indices
    }

    fn push(slot: &mut Option<Self>, index: usize) {
        let bundle = slot.get_or_insert_with(|| Self {
            indices: Vec::new(),
        });
        debug_assert!(bundle.indices().last().is_none_or(|&last| last < index));
        bundle.indices.push(index);
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct JsonDocument {
    pub(crate) source: String,
    pub(crate) original_byte_size: usize,
    pub(crate) root: JsonNode,
    pub(crate) node_count: usize,
    pub(crate) max_depth: u8,
    pub(crate) has_jsonc_syntax: bool,
    pub(crate) comments: Vec<CommentToken>,
    #[cfg(test)]
    pub(crate) scan_steps: usize,
    #[cfg(test)]
    pub(crate) attribution_steps: usize,
    #[cfg(test)]
    comment_bundle_steps: Cell<usize>,
}

impl JsonDocument {
    pub(crate) fn root_kind(&self) -> JsonKind {
        self.root.kind()
    }

    #[cfg(test)]
    pub(crate) fn reset_comment_bundle_steps(&self) {
        self.comment_bundle_steps.set(0);
    }

    #[cfg(test)]
    pub(crate) fn comment_bundle_steps(&self) -> usize {
        self.comment_bundle_steps.get()
    }

    #[cfg(test)]
    pub(crate) fn record_comment_bundle_step(&self) {
        self.comment_bundle_steps
            .set(self.comment_bundle_steps.get().saturating_add(1));
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum LoadError {
    InvalidUtf8 { valid_up_to: usize },
    InvalidJson { message: String },
    TrailingInput { message: String },
    DuplicateMember { name: String },
    MaximumDepthExceeded { maximum: u8, actual: u16 },
}

pub(crate) fn load(bytes: &[u8]) -> Result<JsonDocument, LoadError> {
    let original_byte_size = bytes.len();
    let source_bytes = bytes.strip_prefix(UTF8_BOM).unwrap_or(bytes);
    let source = std::str::from_utf8(source_bytes)
        .map_err(|error| LoadError::InvalidUtf8 {
            valid_up_to: error.valid_up_to(),
        })?
        .to_owned();

    let scan = crate::jsonc::scan(&source).map_err(|error| LoadError::InvalidJson {
        message: error.0.to_owned(),
    })?;

    let mut state = BuildState::new(&scan.parse_view);
    let mut deserializer = serde_json::Deserializer::from_str(&scan.parse_view);
    deserializer.disable_recursion_limit();

    let parsed = NodeSeed {
        state: &mut state,
        depth: 0,
        prefix: ValuePrefix::Root,
    }
    .deserialize(&mut deserializer);
    let mut root = match parsed {
        Ok(root) => root,
        Err(error) => {
            return Err(state
                .failure
                .take()
                .unwrap_or_else(|| LoadError::InvalidJson {
                    message: error.to_string(),
                }));
        }
    };

    if let Err(error) = deserializer.end() {
        return Err(LoadError::TrailingInput {
            message: error.to_string(),
        });
    }
    state.skip_whitespace();
    if state.cursor != scan.parse_view.len() {
        return Err(LoadError::InvalidJson {
            message: "JSON source cursor did not reach the parsed value end".to_owned(),
        });
    }

    let root_syntax_region = root.region;
    let attribution_steps = attribute_comments(
        &mut root,
        root_syntax_region,
        &scan.comments,
        &scan.comment_line_starts,
        &scan.commas,
    )
    .map_err(|message| LoadError::InvalidJson {
        message: message.to_owned(),
    })?;
    debug_assert_eq!(attribution_steps, scan.comments.len());

    root.region = SourceRegion {
        start: 0,
        end: source.len(),
    };
    let node_count = state.node_count;
    let max_depth = state.max_depth;
    drop(deserializer);
    drop(state);
    Ok(JsonDocument {
        source,
        original_byte_size,
        root,
        node_count,
        max_depth,
        has_jsonc_syntax: scan.has_jsonc_syntax,
        comments: scan.comments,
        #[cfg(test)]
        scan_steps: scan.scanned_bytes,
        #[cfg(test)]
        attribution_steps,
        #[cfg(test)]
        comment_bundle_steps: Cell::new(0),
    })
}

fn attribute_comments(
    root: &mut JsonNode,
    root_region: SourceRegion,
    comments: &[CommentToken],
    comment_line_starts: &[usize],
    commas: &[usize],
) -> Result<usize, &'static str> {
    if comments.len() != comment_line_starts.len() {
        return Err("JSONC comment line evidence is incomplete");
    }
    let mut attributor = CommentAttributor {
        comments,
        comment_line_starts,
        commas,
        comment_cursor: 0,
        comma_cursor: 0,
        assignments: 0,
    };
    attributor.take_direct_until(root_region.start, &mut root.direct_comments)?;
    attributor.attribute_node(root, root_region)?;

    let root_token = root_region
        .end
        .checked_sub(1)
        .ok_or("parsed JSON root region is empty")?;
    attributor.take_direct_or_tail_until(
        usize::MAX,
        root_token,
        &mut root.direct_comments,
        &mut root.tail_comments,
    )?;
    if attributor.comment_cursor != comments.len() {
        return Err("JSONC attribution did not consume every comment");
    }
    Ok(attributor.assignments)
}

struct CommentAttributor<'source> {
    comments: &'source [CommentToken],
    comment_line_starts: &'source [usize],
    commas: &'source [usize],
    comment_cursor: usize,
    comma_cursor: usize,
    assignments: usize,
}

impl CommentAttributor<'_> {
    fn attribute_node(
        &mut self,
        node: &mut JsonNode,
        region: SourceRegion,
    ) -> Result<(), &'static str> {
        let closing = region
            .end
            .checked_sub(1)
            .ok_or("parsed JSON node region is empty")?;
        let JsonNode {
            value,
            direct_comments,
            tail_comments,
            ..
        } = node;

        match value {
            JsonValue::Object(members) if members.is_empty() => {
                self.take_direct_until(closing, direct_comments)?;
            }
            JsonValue::Object(members) => {
                self.take_direct_until(
                    members[0].name_region.start,
                    &mut members[0].value.direct_comments,
                )?;
                self.take_direct_until(
                    members[0].value.region.start,
                    &mut members[0].value.direct_comments,
                )?;
                let first_region = members[0].value.region;
                self.attribute_node(&mut members[0].value, first_region)?;

                for index in 1..members.len() {
                    let next_name_start = members[index].name_region.start;
                    let comma = self
                        .peek_comma_before(next_name_start)
                        .ok_or("parsed JSON object member is missing its separator comma")?;
                    let (previous, current) = members.split_at_mut(index);
                    let previous = &mut previous[index - 1].value;
                    let current = &mut current[0].value;
                    self.take_direct_until(comma, &mut previous.direct_comments)?;
                    self.consume_comma(comma)?;
                    self.take_previous_or_next_until(
                        next_name_start,
                        comma,
                        &mut previous.direct_comments,
                        &mut current.direct_comments,
                    )?;
                    self.take_direct_until(current.region.start, &mut current.direct_comments)?;
                    let current_region = current.region;
                    self.attribute_node(current, current_region)?;
                }

                let last = members
                    .last_mut()
                    .expect("non-empty object has a last member");
                if let Some(comma) = self.peek_comma_before(closing) {
                    self.take_direct_until(comma, &mut last.value.direct_comments)?;
                    self.consume_comma(comma)?;
                    self.take_direct_or_tail_until(
                        closing,
                        comma,
                        &mut last.value.direct_comments,
                        tail_comments,
                    )?;
                } else {
                    let last_token = last
                        .value
                        .region
                        .end
                        .checked_sub(1)
                        .ok_or("parsed JSON member value region is empty")?;
                    self.take_direct_or_tail_until(
                        closing,
                        last_token,
                        &mut last.value.direct_comments,
                        tail_comments,
                    )?;
                }
            }
            JsonValue::Array(elements) if elements.is_empty() => {
                self.take_direct_until(closing, direct_comments)?;
            }
            JsonValue::Array(elements) => {
                self.take_direct_until(elements[0].region.start, &mut elements[0].direct_comments)?;
                let first_region = elements[0].region;
                self.attribute_node(&mut elements[0], first_region)?;

                for index in 1..elements.len() {
                    let next_start = elements[index].region.start;
                    let comma = self
                        .peek_comma_before(next_start)
                        .ok_or("parsed JSON array element is missing its separator comma")?;
                    let (previous, current) = elements.split_at_mut(index);
                    let previous = &mut previous[index - 1];
                    let current = &mut current[0];
                    self.take_direct_until(comma, &mut previous.direct_comments)?;
                    self.consume_comma(comma)?;
                    self.take_previous_or_next_until(
                        next_start,
                        comma,
                        &mut previous.direct_comments,
                        &mut current.direct_comments,
                    )?;
                    let current_region = current.region;
                    self.attribute_node(current, current_region)?;
                }

                let last = elements
                    .last_mut()
                    .expect("non-empty array has a last element");
                if let Some(comma) = self.peek_comma_before(closing) {
                    self.take_direct_until(comma, &mut last.direct_comments)?;
                    self.consume_comma(comma)?;
                    self.take_direct_or_tail_until(
                        closing,
                        comma,
                        &mut last.direct_comments,
                        tail_comments,
                    )?;
                } else {
                    let last_token = last
                        .region
                        .end
                        .checked_sub(1)
                        .ok_or("parsed JSON array element region is empty")?;
                    self.take_direct_or_tail_until(
                        closing,
                        last_token,
                        &mut last.direct_comments,
                        tail_comments,
                    )?;
                }
            }
            JsonValue::String(_)
            | JsonValue::Number(_)
            | JsonValue::Boolean(_)
            | JsonValue::Null => {}
        }
        Ok(())
    }

    fn take_direct_until(
        &mut self,
        end: usize,
        direct: &mut Option<CommentBundle>,
    ) -> Result<(), &'static str> {
        while self.next_comment_starts_before(end) {
            let index = self.take_comment_ending_by(end)?;
            CommentBundle::push(direct, index);
        }
        Ok(())
    }

    fn take_previous_or_next_until(
        &mut self,
        end: usize,
        previous_token: usize,
        previous: &mut Option<CommentBundle>,
        next: &mut Option<CommentBundle>,
    ) -> Result<(), &'static str> {
        while self.next_comment_starts_before(end) {
            let same_line = self.comment_line_starts[self.comment_cursor] <= previous_token;
            let index = self.take_comment_ending_by(end)?;
            if same_line {
                CommentBundle::push(previous, index);
            } else {
                CommentBundle::push(next, index);
            }
        }
        Ok(())
    }

    fn take_direct_or_tail_until(
        &mut self,
        end: usize,
        previous_token: usize,
        direct: &mut Option<CommentBundle>,
        tail: &mut Option<CommentBundle>,
    ) -> Result<(), &'static str> {
        while self.next_comment_starts_before(end) {
            let same_line = self.comment_line_starts[self.comment_cursor] <= previous_token;
            let index = self.take_comment_ending_by(end)?;
            if same_line {
                CommentBundle::push(direct, index);
            } else {
                CommentBundle::push(tail, index);
            }
        }
        Ok(())
    }

    fn next_comment_starts_before(&self, end: usize) -> bool {
        self.comments
            .get(self.comment_cursor)
            .is_some_and(|comment| comment.span.start < end)
    }

    fn take_comment_ending_by(&mut self, end: usize) -> Result<usize, &'static str> {
        let index = self.comment_cursor;
        let comment = self
            .comments
            .get(index)
            .ok_or("JSONC attribution comment cursor exceeded evidence")?;
        if comment.span.end > end {
            return Err("JSONC comment crosses a parsed token boundary");
        }
        self.comment_cursor += 1;
        self.assignments += 1;
        Ok(index)
    }

    fn peek_comma_before(&self, end: usize) -> Option<usize> {
        self.commas
            .get(self.comma_cursor)
            .copied()
            .filter(|&comma| comma < end)
    }

    fn consume_comma(&mut self, comma: usize) -> Result<(), &'static str> {
        if self.commas.get(self.comma_cursor) != Some(&comma) {
            return Err("JSONC attribution comma cursor lost source order");
        }
        self.comma_cursor += 1;
        Ok(())
    }
}

struct BuildState<'source> {
    source: &'source str,
    cursor: usize,
    node_count: usize,
    max_depth: u8,
    failure: Option<LoadError>,
}

impl<'source> BuildState<'source> {
    fn new(source: &'source str) -> Self {
        Self {
            source,
            cursor: 0,
            node_count: 0,
            max_depth: 0,
            failure: None,
        }
    }

    fn record_node(&mut self, depth: u8) {
        self.node_count += 1;
        self.max_depth = self.max_depth.max(depth);
    }

    fn prepare_value(&mut self, prefix: ValuePrefix) -> CursorResult<()> {
        self.skip_whitespace();
        match prefix {
            ValuePrefix::Root | ValuePrefix::ArrayElement { first: true } => {}
            ValuePrefix::ArrayElement { first: false } => {
                self.consume_byte(b',', "expected an array element separator")?;
                self.skip_whitespace();
            }
            ValuePrefix::ObjectValue => {
                self.consume_byte(b':', "expected an object member separator")?;
                self.skip_whitespace();
            }
        }
        Ok(())
    }

    fn prepare_member_name(&mut self, first: bool) -> CursorResult<()> {
        self.skip_whitespace();
        if !first {
            self.consume_byte(b',', "expected an object member separator")?;
            self.skip_whitespace();
        }
        Ok(())
    }

    fn open_container(&mut self, opening: u8) -> CursorResult<usize> {
        let start = self.cursor;
        self.consume_byte(opening, "expected a JSON container opening token")?;
        Ok(start)
    }

    fn close_container(&mut self, start: usize, closing: u8) -> CursorResult<SourceRegion> {
        self.skip_whitespace();
        self.consume_byte(closing, "expected a JSON container closing token")?;
        Ok(SourceRegion {
            start,
            end: self.cursor,
        })
    }

    fn consume_string(&mut self) -> CursorResult<SourceRegion> {
        let bytes = self.source.as_bytes();
        let start = self.cursor;
        if bytes.get(self.cursor) != Some(&b'"') {
            return Err(CursorError("expected a JSON string token"));
        }
        self.cursor += 1;

        while let Some(&byte) = bytes.get(self.cursor) {
            match byte {
                b'"' => {
                    self.cursor += 1;
                    return Ok(SourceRegion {
                        start,
                        end: self.cursor,
                    });
                }
                b'\\' => {
                    self.cursor += 1;
                    let escaped = bytes
                        .get(self.cursor)
                        .ok_or(CursorError("unterminated JSON string escape"))?;
                    self.cursor += if *escaped == b'u' { 5 } else { 1 };
                    if self.cursor > bytes.len() {
                        return Err(CursorError("unterminated JSON unicode escape"));
                    }
                }
                _ => self.cursor += 1,
            }
        }

        Err(CursorError("unterminated JSON string token"))
    }

    fn consume_literal(&mut self, literal: &[u8]) -> CursorResult<SourceRegion> {
        let start = self.cursor;
        let end = start.saturating_add(literal.len());
        if self.source.as_bytes().get(start..end) != Some(literal) {
            return Err(CursorError("JSON literal did not match parsed value"));
        }
        self.cursor = end;
        Ok(SourceRegion { start, end })
    }

    fn consume_number(&mut self, parsed: &str) -> CursorResult<(SourceRegion, String)> {
        let start = self.cursor;
        let end = start + parsed.len();
        if self.source.get(start..end) != Some(parsed) {
            return Err(CursorError(
                "parsed JSON number did not match the source cursor",
            ));
        }
        self.cursor = end;
        Ok((SourceRegion { start, end }, parsed.to_owned()))
    }

    fn consume_byte(&mut self, expected: u8, message: &'static str) -> CursorResult<()> {
        if self.source.as_bytes().get(self.cursor) != Some(&expected) {
            return Err(CursorError(message));
        }
        self.cursor += 1;
        Ok(())
    }

    fn skip_whitespace(&mut self) {
        while matches!(
            self.source.as_bytes().get(self.cursor),
            Some(b' ' | b'\t' | b'\r' | b'\n')
        ) {
            self.cursor += 1;
        }
    }

    fn current_byte(&self) -> Option<u8> {
        self.source.as_bytes().get(self.cursor).copied()
    }
}

#[derive(Clone, Copy)]
enum ValuePrefix {
    Root,
    ArrayElement { first: bool },
    ObjectValue,
}

struct NodeSeed<'state, 'source> {
    state: &'state mut BuildState<'source>,
    depth: u16,
    prefix: ValuePrefix,
}

impl<'de> DeserializeSeed<'de> for NodeSeed<'_, '_> {
    type Value = JsonNode;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let Self {
            state,
            depth,
            prefix,
        } = self;
        if depth > u16::from(MAX_DEPTH) {
            state.failure = Some(LoadError::MaximumDepthExceeded {
                maximum: MAX_DEPTH,
                actual: depth,
            });
            return Err(de::Error::custom("JSON maximum depth exceeded"));
        }
        state.prepare_value(prefix).map_err(de::Error::custom)?;

        let depth = depth as u8;
        let node = if matches!(state.current_byte(), Some(b'-' | b'0'..=b'9')) {
            let raw = <&RawValue>::deserialize(deserializer)?;
            NodeVisitor { state, depth }.number(raw.get())?
        } else {
            deserializer.deserialize_any(NodeVisitor { state, depth })?
        };
        state.record_node(depth);
        Ok(node)
    }
}

struct NodeVisitor<'state, 'source> {
    state: &'state mut BuildState<'source>,
    depth: u8,
}

impl<'de> Visitor<'de> for NodeVisitor<'_, '_> {
    type Value = JsonNode;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("one JSON value")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let region = self
            .state
            .consume_literal(b"null")
            .map_err(de::Error::custom)?;
        Ok(self.node(region, JsonValue::Null))
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let literal = if value {
            b"true".as_slice()
        } else {
            b"false".as_slice()
        };
        let region = self
            .state
            .consume_literal(literal)
            .map_err(de::Error::custom)?;
        Ok(self.node(region, JsonValue::Boolean(value)))
    }

    fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.string(value.to_owned())
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.string(value.to_owned())
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.string(value)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let start = self.state.open_container(b'[').map_err(de::Error::custom)?;
        let mut elements = Vec::new();
        loop {
            let first = elements.is_empty();
            let element = sequence.next_element_seed(NodeSeed {
                state: self.state,
                depth: u16::from(self.depth) + 1,
                prefix: ValuePrefix::ArrayElement { first },
            })?;
            match element {
                Some(element) => elements.push(element),
                None => break,
            }
        }
        let region = self
            .state
            .close_container(start, b']')
            .map_err(de::Error::custom)?;
        Ok(self.node(region, JsonValue::Array(elements)))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let start = self.state.open_container(b'{').map_err(de::Error::custom)?;
        let mut members = Vec::new();
        let mut names = HashSet::new();
        loop {
            let first = members.is_empty();
            let name = map.next_key_seed(MemberNameSeed {
                state: self.state,
                first,
            })?;
            let Some(name) = name else {
                break;
            };
            if !names.insert(name.decoded.clone()) {
                self.state.failure = Some(LoadError::DuplicateMember {
                    name: name.decoded.clone(),
                });
                return Err(de::Error::custom("duplicate decoded JSON member name"));
            }

            let value = map.next_value_seed(NodeSeed {
                state: self.state,
                depth: u16::from(self.depth) + 1,
                prefix: ValuePrefix::ObjectValue,
            })?;
            members.push(JsonMember {
                name: name.decoded,
                name_region: name.region,
                region: SourceRegion {
                    start: name.region.start,
                    end: value.region.end,
                },
                value,
            });
        }
        let region = self
            .state
            .close_container(start, b'}')
            .map_err(de::Error::custom)?;
        Ok(self.node(region, JsonValue::Object(members)))
    }
}

impl NodeVisitor<'_, '_> {
    fn number<E>(self, parsed: &str) -> Result<JsonNode, E>
    where
        E: de::Error,
    {
        let (region, raw) = self
            .state
            .consume_number(parsed)
            .map_err(de::Error::custom)?;
        Ok(self.node(region, JsonValue::Number(raw)))
    }

    fn string<E>(self, decoded: String) -> Result<JsonNode, E>
    where
        E: de::Error,
    {
        let region = self.state.consume_string().map_err(de::Error::custom)?;
        Ok(self.node(region, JsonValue::String(decoded)))
    }

    fn node(self, region: SourceRegion, value: JsonValue) -> JsonNode {
        JsonNode {
            depth: self.depth,
            region,
            value,
            direct_comments: None,
            tail_comments: None,
        }
    }
}

struct MemberName {
    decoded: String,
    region: SourceRegion,
}

struct MemberNameSeed<'state, 'source> {
    state: &'state mut BuildState<'source>,
    first: bool,
}

impl<'de> DeserializeSeed<'de> for MemberNameSeed<'_, '_> {
    type Value = MemberName;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        self.state
            .prepare_member_name(self.first)
            .map_err(de::Error::custom)?;
        let decoded = String::deserialize(deserializer)?;
        let region = self.state.consume_string().map_err(de::Error::custom)?;
        Ok(MemberName { decoded, region })
    }
}

#[derive(Debug)]
struct CursorError(&'static str);

impl fmt::Display for CursorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

type CursorResult<T> = Result<T, CursorError>;

#[cfg(test)]
mod tests;
