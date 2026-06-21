use docnav_protocol::{positive_result, Entry, PositiveInteger};

const TRUNCATION_MARKER: &str = "...";
const MIN_DISPLAY: &str = ".";

pub trait PageableEntry: Clone {
    fn ref_id(&self) -> &str;

    fn display(&self) -> &str;

    fn with_display(&self, display: String) -> Self;
}

impl PageableEntry for Entry {
    fn ref_id(&self) -> &str {
        &self.ref_id
    }

    fn display(&self) -> &str {
        &self.display
    }

    fn with_display(&self, display: String) -> Self {
        Self {
            ref_id: self.ref_id.clone(),
            display,
        }
    }
}

pub fn paginate_text(
    content: &str,
    page: PositiveInteger,
    limit_chars: PositiveInteger,
) -> (String, Option<PositiveInteger>) {
    let page_number = page.get() as usize;
    let limit = limit_chars.get() as usize;
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
    limit_chars: PositiveInteger,
) -> (Vec<T>, Option<PositiveInteger>) {
    let target_page = page.get();
    let limit = limit_chars.get() as usize;
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
    let display_len = char_count(entry.display());

    if ref_len.saturating_add(display_len) <= limit {
        return entry.clone();
    }

    if ref_len >= limit {
        return entry.with_display(MIN_DISPLAY.to_owned());
    }

    let display_budget = limit - ref_len;
    let marker_len = TRUNCATION_MARKER.chars().count();

    if display_budget > marker_len && display_len > display_budget {
        let content_budget = display_budget - marker_len;
        let clipped = take_chars(entry.display(), content_budget);
        entry.with_display(format!("{clipped}{TRUNCATION_MARKER}"))
    } else {
        entry.with_display(take_chars(entry.display(), display_budget.max(1)))
    }
}

fn entry_cost<T: PageableEntry>(entry: &T) -> usize {
    char_count(entry.ref_id()) + char_count(entry.display())
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
mod tests {
    // @case WB-SDK-PAGE-001
    use super::*;

    fn positive(value: u32) -> PositiveInteger {
        positive_result(value).expect("positive test integer")
    }

    #[test]
    fn text_paging_counts_unicode_characters() {
        let (page, next) = paginate_text("a界b", positive(1), positive(2));
        assert_eq!(page, "a界");
        assert_eq!(next, Some(positive(2)));

        let (page, next) = paginate_text("a界b", positive(2), positive(2));
        assert_eq!(page, "b");
        assert_eq!(next, None);
    }

    #[test]
    fn oversized_entry_keeps_full_ref_and_still_advances() {
        let entries = vec![
            Entry {
                ref_id: "L1:very-long-reference".to_owned(),
                display: "very long display text".to_owned(),
            },
            Entry {
                ref_id: "L2:next".to_owned(),
                display: "next".to_owned(),
            },
        ];

        let (page, next) = paginate_entries(&entries, positive(1), positive(5));
        assert_eq!(page.len(), 1);
        assert_eq!(page[0].ref_id, "L1:very-long-reference");
        assert_eq!(page[0].display, ".");
        assert_eq!(next, Some(positive(2)));
    }

    #[test]
    fn truncated_display_includes_ellipsis_marker() {
        let entries = vec![
            Entry {
                ref_id: "R:longref1".to_owned(),
                display: "A very long display text that should be truncated".to_owned(),
            },
            Entry {
                ref_id: "X".to_owned(),
                display: "next".to_owned(),
            },
        ];

        let (page, next) = paginate_entries(&entries, positive(1), positive(30));
        assert_eq!(page.len(), 1);
        assert_eq!(page[0].ref_id, "R:longref1");
        assert!(
            page[0].display.ends_with("..."),
            "truncated display must end with '...' marker, got: '{}'",
            page[0].display
        );
        assert_eq!(next, Some(positive(2)));

        let cost = page[0].ref_id.chars().count() + page[0].display.chars().count();
        assert!(cost <= 30, "truncated entry cost {cost} exceeds limit 30");
    }

    #[test]
    fn unicode_truncated_display_includes_marker() {
        let entries = vec![
            Entry {
                ref_id: "R:longref1".to_owned(),
                display: "界世界世界世界世界世界世界世界世界世界世".to_owned(),
            },
            Entry {
                ref_id: "X".to_owned(),
                display: "next".to_owned(),
            },
        ];

        let (page, next) = paginate_entries(&entries, positive(1), positive(25));
        assert_eq!(page.len(), 1);
        assert_eq!(page[0].ref_id, "R:longref1");
        assert!(
            page[0].display.ends_with("..."),
            "unicode truncated display must end with '...' marker"
        );
        assert_eq!(next, Some(positive(2)));

        let cost = page[0].ref_id.chars().count() + page[0].display.chars().count();
        assert!(
            cost <= 25,
            "unicode truncated entry cost {cost} exceeds limit 25"
        );
    }

    #[test]
    fn tiny_budget_no_room_for_marker_still_truncates() {
        let entries = vec![
            Entry {
                ref_id: "R:longref1".to_owned(),
                display: "a very long display text".to_owned(),
            },
            Entry {
                ref_id: "X".to_owned(),
                display: "next".to_owned(),
            },
        ];

        let (page, next) = paginate_entries(&entries, positive(1), positive(11));
        assert_eq!(page.len(), 1);
        assert_eq!(page[0].ref_id, "R:longref1");
        assert!(!page[0].display.is_empty(), "display should not be empty");

        let cost = page[0].ref_id.chars().count() + page[0].display.chars().count();
        assert!(cost <= 11, "tiny budget entry cost {cost} exceeds limit 11");
        assert_eq!(next, Some(positive(2)));
    }
}
