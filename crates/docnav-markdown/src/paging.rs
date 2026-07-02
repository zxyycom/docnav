use docnav_protocol::{positive_result, Entry, PositiveInteger};

const TRUNCATION_MARKER: &str = "...";
const MIN_DISPLAY: &str = ".";

pub trait PageableEntry: Clone {
    fn ref_id(&self) -> &str;

    fn budget_text(&self) -> String;

    fn with_budget_text(&self, text: String) -> Self;
}

impl PageableEntry for Entry {
    fn ref_id(&self) -> &str {
        &self.ref_id
    }

    fn budget_text(&self) -> String {
        let mut parts = vec![self.label.as_str()];
        if let Some(summary) = self.summary.as_deref() {
            parts.push(summary);
        }
        if let Some(excerpt) = self.excerpt.as_deref() {
            parts.push(excerpt);
        }
        parts.join(" | ")
    }

    fn with_budget_text(&self, text: String) -> Self {
        let mut entry = self.clone();
        entry.label = text;
        entry.summary = None;
        entry.excerpt = None;
        entry.cost = None;
        entry
    }
}

pub fn paginate_text(
    content: &str,
    page: PositiveInteger,
    limit: PositiveInteger,
) -> (String, Option<PositiveInteger>) {
    let page_number = page.get() as usize;
    let limit = limit.get() as usize;
    let total = content.chars().count();
    let start = page_number.saturating_sub(1).saturating_mul(limit);

    if start >= total {
        return (String::new(), None);
    }

    let end = start.saturating_add(limit).min(total);
    let page_content = content.chars().skip(start).take(end - start).collect();
    let next_page = next_page(page, end < total);

    (page_content, next_page)
}

pub fn paginate_entries<T: PageableEntry>(
    entries: &[T],
    page: PositiveInteger,
    limit: PositiveInteger,
) -> (Vec<T>, Option<PositiveInteger>) {
    let target_page = page.get();
    let limit = limit.get() as usize;
    let mut index = 0;
    let mut current_page = 1;

    while current_page < target_page && index < entries.len() {
        let (_, next_index) = entries_page(entries, index, limit);
        index = next_index;
        current_page += 1;
    }

    if index >= entries.len() {
        return (Vec::new(), None);
    }

    let (page_entries, next_index) = entries_page(entries, index, limit);
    let next_page = next_page(page, next_index < entries.len());

    (page_entries, next_page)
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

fn entries_page<T: PageableEntry>(entries: &[T], start: usize, limit: usize) -> (Vec<T>, usize) {
    let mut page_entries = Vec::new();
    let mut used: usize = 0;
    let mut index = start;

    while let Some(entry) = entries.get(index) {
        let adjusted = fit_entry(entry, limit);
        let cost = entry_cost(&adjusted);

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

fn fit_entry<T: PageableEntry>(entry: &T, limit: usize) -> T {
    let ref_len = char_count(entry.ref_id());
    let budget_text = entry.budget_text();
    let display_len = char_count(&budget_text);

    if ref_len.saturating_add(display_len) <= limit {
        return entry.clone();
    }

    if ref_len >= limit {
        return entry.with_budget_text(MIN_DISPLAY.to_owned());
    }

    let display_budget = limit - ref_len;
    let marker_len = TRUNCATION_MARKER.chars().count();

    if display_budget > marker_len && display_len > display_budget {
        let content_budget = display_budget - marker_len;
        let clipped = take_chars(&budget_text, content_budget);
        entry.with_budget_text(format!("{clipped}{TRUNCATION_MARKER}"))
    } else {
        entry.with_budget_text(take_chars(&budget_text, display_budget.max(1)))
    }
}

fn entry_cost<T: PageableEntry>(entry: &T) -> usize {
    char_count(entry.ref_id()) + char_count(&entry.budget_text())
}

fn char_count(value: &str) -> usize {
    value.chars().count()
}

fn take_chars(value: &str, count: usize) -> String {
    let clipped: String = value.chars().take(count).collect();
    if clipped.is_empty() {
        MIN_DISPLAY.to_owned()
    } else {
        clipped
    }
}

#[cfg(test)]
mod tests;
