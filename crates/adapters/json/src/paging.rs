use std::iter::Peekable;

use docnav_protocol::{positive_result, Cost, PositiveInteger};

use crate::content::ContentFacts;
use crate::find::FindEntry;
use crate::traversal::JsonEntry;

const TRUNCATION_MARKER: &str = "...";
const MINIMUM_LABEL: &str = ".";

struct EntryProjection<T> {
    fields: fn(&T) -> (&str, &str),
    fixed_label: fn(&T) -> bool,
    set_label: fn(&mut T, String),
    summary: fn(&T) -> Option<&str>,
    set_summary: fn(&mut T, Option<String>),
}

impl<T> Clone for EntryProjection<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for EntryProjection<T> {}

#[derive(Debug, PartialEq)]
pub(crate) struct TextPage {
    pub(crate) content: String,
    pub(crate) cost: Cost,
    pub(crate) page: Option<PositiveInteger>,
}

pub(crate) fn paginate_text(
    facts: ContentFacts,
    page: PositiveInteger,
    limit: PositiveInteger,
) -> TextPage {
    let ContentFacts { content, cost } = facts;
    let page_number = page.get() as usize;
    let limit = limit.get() as usize;
    let total = content.chars().count();
    let start = page_number.saturating_sub(1).saturating_mul(limit);

    if start >= total {
        return TextPage {
            content: String::new(),
            cost,
            page: None,
        };
    }

    let end = start.saturating_add(limit).min(total);
    let content = content.chars().skip(start).take(end - start).collect();

    TextPage {
        content,
        cost,
        page: next_page(page, end < total),
    }
}

pub(crate) fn paginate_entries(
    entries: &[JsonEntry],
    page: PositiveInteger,
    limit: PositiveInteger,
) -> (Vec<JsonEntry>, Option<PositiveInteger>) {
    paginate_entry_slice(
        entries,
        page,
        limit,
        EntryProjection {
            fields: json_entry_fields,
            fixed_label: json_entry_has_fixed_label,
            set_label: set_json_entry_label,
            summary: json_entry_summary,
            set_summary: set_json_entry_summary,
        },
    )
}

pub(crate) fn paginate_find_entries(
    entries: impl Iterator<Item = FindEntry>,
    page: PositiveInteger,
    limit: PositiveInteger,
) -> (Vec<FindEntry>, Option<PositiveInteger>) {
    let mut entries = entries.peekable();
    let mut current_page = 1;
    let limit = limit.get() as usize;

    while current_page < page.get() {
        if entries.peek().is_none() {
            return (Vec::new(), None);
        }
        find_entries_page(&mut entries, limit, false);
        current_page += 1;
    }

    if entries.peek().is_none() {
        return (Vec::new(), None);
    }

    let page_entries = find_entries_page(&mut entries, limit, true);
    let next_page = next_page(page, entries.peek().is_some());
    (page_entries, next_page)
}

fn find_entries_page<I>(entries: &mut Peekable<I>, limit: usize, retain: bool) -> Vec<FindEntry>
where
    I: Iterator<Item = FindEntry>,
{
    let mut page_entries = Vec::new();
    let mut used = 0_usize;

    while let Some(entry) = entries.peek() {
        let projection = EntryProjection {
            fields: find_entry_fields,
            fixed_label: no_fixed_label,
            set_label: set_find_entry_label,
            summary: no_entry_summary,
            set_summary: ignore_entry_summary,
        };
        let adjusted = fit_entry(entry, limit, projection);
        let cost = entry_cost(&adjusted, projection);

        if used > 0 && used.saturating_add(cost) > limit {
            break;
        }

        used = used.saturating_add(cost);
        entries
            .next()
            .expect("peeked find entry remains available until consumed");
        if retain {
            page_entries.push(adjusted);
        }

        if used >= limit {
            break;
        }
    }

    page_entries
}

fn paginate_entry_slice<T: Clone>(
    entries: &[T],
    page: PositiveInteger,
    limit: PositiveInteger,
    projection: EntryProjection<T>,
) -> (Vec<T>, Option<PositiveInteger>) {
    let target_page = page.get();
    let limit = limit.get() as usize;
    let mut index = 0;
    let mut current_page = 1;

    while current_page < target_page && index < entries.len() {
        let (_, next_index) = entries_page(entries, index, limit, projection);
        index = next_index;
        current_page += 1;
    }

    if index >= entries.len() {
        return (Vec::new(), None);
    }

    let (page_entries, next_index) = entries_page(entries, index, limit, projection);
    let next_page = next_page(page, next_index < entries.len());

    (page_entries, next_page)
}

fn entries_page<T: Clone>(
    entries: &[T],
    start: usize,
    limit: usize,
    projection: EntryProjection<T>,
) -> (Vec<T>, usize) {
    let mut page_entries = Vec::new();
    let mut used = 0_usize;
    let mut index = start;

    while let Some(entry) = entries.get(index) {
        let adjusted = fit_entry(entry, limit, projection);
        let cost = entry_cost(&adjusted, projection);

        if !page_entries.is_empty() && used.saturating_add(cost) > limit {
            break;
        }

        used = used.saturating_add(cost);
        page_entries.push(adjusted);
        index += 1;

        if used >= limit {
            break;
        }
    }

    (page_entries, index)
}

fn fit_entry<T: Clone>(entry: &T, limit: usize, projection: EntryProjection<T>) -> T {
    let (ref_id, label) = (projection.fields)(entry);
    let ref_length = char_count(ref_id);
    let label_length = char_count(label);
    let summary = (projection.summary)(entry);
    let summary_length = summary.map(char_count).unwrap_or_default();
    let required_length = ref_length.saturating_add(label_length);

    if required_length.saturating_add(summary_length) <= limit {
        return entry.clone();
    }

    let mut adjusted = entry.clone();
    if required_length <= limit {
        let summary_budget = limit - required_length;
        (projection.set_summary)(&mut adjusted, fit_summary(summary, summary_budget));
        return adjusted;
    }
    (projection.set_summary)(&mut adjusted, None);
    if (projection.fixed_label)(entry) {
        return adjusted;
    }

    let adjusted_label = if ref_length >= limit {
        MINIMUM_LABEL.to_owned()
    } else {
        let label_budget = limit - ref_length;
        let marker_length = char_count(TRUNCATION_MARKER);
        if label_budget > marker_length && label_length > label_budget {
            let content_budget = label_budget - marker_length;
            format!("{}{TRUNCATION_MARKER}", take_chars(label, content_budget))
        } else {
            take_chars(label, label_budget.max(1))
        }
    };
    (projection.set_label)(&mut adjusted, adjusted_label);
    adjusted
}

fn fit_summary(summary: Option<&str>, budget: usize) -> Option<String> {
    let summary = summary?;
    if char_count(summary) <= budget {
        return Some(summary.to_owned());
    }
    let marker_length = char_count(TRUNCATION_MARKER);
    (budget > marker_length).then(|| {
        let content_budget = budget - marker_length;
        format!(
            "{}{TRUNCATION_MARKER}",
            take_chars_exact(summary, content_budget)
        )
    })
}

fn entry_cost<T>(entry: &T, projection: EntryProjection<T>) -> usize {
    let (ref_id, label) = (projection.fields)(entry);
    char_count(ref_id)
        .saturating_add(char_count(label))
        .saturating_add(
            (projection.summary)(entry)
                .map(char_count)
                .unwrap_or_default(),
        )
}

fn json_entry_fields(entry: &JsonEntry) -> (&str, &str) {
    (&entry.ref_id, &entry.label)
}

fn json_entry_has_fixed_label(entry: &JsonEntry) -> bool {
    entry.label == "\"\"" && entry.ref_id.ends_with('/')
}

fn set_json_entry_label(entry: &mut JsonEntry, label: String) {
    entry.label = label;
}

fn json_entry_summary(entry: &JsonEntry) -> Option<&str> {
    entry.summary.as_deref()
}

fn set_json_entry_summary(entry: &mut JsonEntry, summary: Option<String>) {
    entry.summary = summary;
}

fn find_entry_fields(entry: &FindEntry) -> (&str, &str) {
    (&entry.ref_id, &entry.label)
}

fn no_fixed_label<T>(_entry: &T) -> bool {
    false
}

fn set_find_entry_label(entry: &mut FindEntry, label: String) {
    entry.label = label;
}

fn no_entry_summary<T>(_entry: &T) -> Option<&str> {
    None
}

fn ignore_entry_summary<T>(_entry: &mut T, summary: Option<String>) {
    debug_assert!(summary.is_none());
}

fn next_page(current: PositiveInteger, has_more: bool) -> Option<PositiveInteger> {
    has_more
        .then(|| {
            current
                .get()
                .checked_add(1)
                .and_then(|value| positive_result(value).ok())
        })
        .flatten()
}

fn char_count(value: &str) -> usize {
    value.chars().count()
}

fn take_chars(value: &str, count: usize) -> String {
    let clipped = take_chars_exact(value, count);
    if clipped.is_empty() {
        MINIMUM_LABEL.to_owned()
    } else {
        clipped
    }
}

fn take_chars_exact(value: &str, count: usize) -> String {
    value.chars().take(count).collect()
}

#[cfg(test)]
mod tests;
