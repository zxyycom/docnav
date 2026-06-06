use docnav_protocol::{positive_result, Entry, PositiveInteger};

pub fn paginate_text(
    content: &str,
    page: PositiveInteger,
    limit_chars: PositiveInteger,
) -> (String, Option<PositiveInteger>) {
    let page = page.get() as usize;
    let limit = limit_chars.get() as usize;
    let total = content.chars().count();
    let start = page.saturating_sub(1).saturating_mul(limit);

    if start >= total {
        return (String::new(), None);
    }

    let end = start.saturating_add(limit).min(total);
    let page_content = content.chars().skip(start).take(end - start).collect();
    let next_page = (end < total)
        .then(|| {
            page.checked_add(1)
                .and_then(|value| positive_result(value as u32).ok())
        })
        .flatten();

    (page_content, next_page)
}

pub fn paginate_entries(
    entries: &[Entry],
    page: PositiveInteger,
    limit_chars: PositiveInteger,
) -> (Vec<Entry>, Option<PositiveInteger>) {
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
    let next_page = (next_index < entries.len())
        .then(|| {
            page.get()
                .checked_add(1)
                .and_then(|value| positive_result(value).ok())
        })
        .flatten();

    (page_entries, next_page)
}

fn entries_page(entries: &[Entry], start: usize, limit: usize) -> (Vec<Entry>, usize) {
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

fn fit_entry(entry: &Entry, limit: usize) -> Entry {
    let ref_len = char_count(&entry.ref_id);
    let display_len = char_count(&entry.display);

    if ref_len.saturating_add(display_len) <= limit {
        return entry.clone();
    }

    if ref_len >= limit {
        return Entry {
            ref_id: entry.ref_id.clone(),
            display: ".".to_owned(),
        };
    }

    let display_budget = limit - ref_len;
    Entry {
        ref_id: entry.ref_id.clone(),
        display: take_chars(&entry.display, display_budget.max(1)),
    }
}

fn entry_cost(entry: &Entry) -> usize {
    char_count(&entry.ref_id) + char_count(&entry.display)
}

fn char_count(value: &str) -> usize {
    value.chars().count()
}

fn take_chars(value: &str, count: usize) -> String {
    let clipped: String = value.chars().take(count).collect();
    if clipped.is_empty() {
        ".".to_owned()
    } else {
        clipped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn positive(value: u32) -> PositiveInteger {
        positive_result(value).expect("positive test integer")
    }

    #[test]
    fn read_paging_counts_unicode_characters() {
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
}
