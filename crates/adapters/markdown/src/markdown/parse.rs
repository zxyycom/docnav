use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use super::document::Heading;
use super::text::{line_for_byte, normalize_heading_text};

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

pub(super) fn parse_headings(
    source: &str,
    line_starts: &[usize],
    source_len: usize,
) -> Vec<Heading> {
    let raw_headings = raw_headings(source);
    let mut headings = materialize_headings(raw_headings, line_starts, source_len);
    assign_section_ends(&mut headings, source_len);
    headings
}

fn raw_headings(source: &str) -> Vec<RawHeading> {
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
    let mut headings = Vec::with_capacity(raw_headings.len());

    for (index, raw) in raw_headings.into_iter().enumerate() {
        headings.push(Heading {
            index: index + 1,
            level: raw.level,
            title: raw.title,
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
