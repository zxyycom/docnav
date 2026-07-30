use std::iter::Peekable;

use docnav_protocol::{positive_result, Cost, PositiveInteger};

use crate::content::ContentFacts;
use crate::find::FindEntry;
use crate::traversal::JsonEntry;

const TRUNCATION_MARKER: &str = "...";
const MINIMUM_LABEL: &str = ".";

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
        json_entry_fields,
        set_json_entry_label,
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
        let adjusted = fit_entry(entry, limit, find_entry_fields, set_find_entry_label);
        let cost = entry_cost(&adjusted, find_entry_fields);

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
    fields: fn(&T) -> (&str, &str),
    set_label: fn(&mut T, String),
) -> (Vec<T>, Option<PositiveInteger>) {
    let target_page = page.get();
    let limit = limit.get() as usize;
    let mut index = 0;
    let mut current_page = 1;

    while current_page < target_page && index < entries.len() {
        let (_, next_index) = entries_page(entries, index, limit, fields, set_label);
        index = next_index;
        current_page += 1;
    }

    if index >= entries.len() {
        return (Vec::new(), None);
    }

    let (page_entries, next_index) = entries_page(entries, index, limit, fields, set_label);
    let next_page = next_page(page, next_index < entries.len());

    (page_entries, next_page)
}

fn entries_page<T: Clone>(
    entries: &[T],
    start: usize,
    limit: usize,
    fields: fn(&T) -> (&str, &str),
    set_label: fn(&mut T, String),
) -> (Vec<T>, usize) {
    let mut page_entries = Vec::new();
    let mut used = 0_usize;
    let mut index = start;

    while let Some(entry) = entries.get(index) {
        let adjusted = fit_entry(entry, limit, fields, set_label);
        let cost = entry_cost(&adjusted, fields);

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

fn fit_entry<T: Clone>(
    entry: &T,
    limit: usize,
    fields: fn(&T) -> (&str, &str),
    set_label: fn(&mut T, String),
) -> T {
    let (ref_id, label) = fields(entry);
    let ref_length = char_count(ref_id);
    let label_length = char_count(label);

    if ref_length.saturating_add(label_length) <= limit {
        return entry.clone();
    }

    let mut adjusted = entry.clone();
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
    set_label(&mut adjusted, adjusted_label);
    adjusted
}

fn entry_cost<T>(entry: &T, fields: fn(&T) -> (&str, &str)) -> usize {
    let (ref_id, label) = fields(entry);
    char_count(ref_id) + char_count(label)
}

fn json_entry_fields(entry: &JsonEntry) -> (&str, &str) {
    (&entry.ref_id, &entry.label)
}

fn set_json_entry_label(entry: &mut JsonEntry, label: String) {
    entry.label = label;
}

fn find_entry_fields(entry: &FindEntry) -> (&str, &str) {
    (&entry.ref_id, &entry.label)
}

fn set_find_entry_label(entry: &mut FindEntry, label: String) {
    entry.label = label;
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
    let clipped = value.chars().take(count).collect::<String>();
    if clipped.is_empty() {
        MINIMUM_LABEL.to_owned()
    } else {
        clipped
    }
}

#[cfg(test)]
mod tests;
