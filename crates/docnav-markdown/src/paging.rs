use docnav_adapter_sdk::paging as sdk_paging;
use docnav_protocol::{Entry, PositiveInteger};

pub fn paginate_text(
    content: &str,
    page: PositiveInteger,
    limit_chars: PositiveInteger,
) -> (String, Option<PositiveInteger>) {
    sdk_paging::paginate_text(content, page, limit_chars)
}

pub fn paginate_entries(
    entries: &[Entry],
    page: PositiveInteger,
    limit_chars: PositiveInteger,
) -> (Vec<Entry>, Option<PositiveInteger>) {
    sdk_paging::paginate_entries(entries, page, limit_chars)
}

#[cfg(test)]
mod tests {
    // @case WB-MD-PAGING-DISPLAY-001
    use super::*;
    use docnav_protocol::positive_result;

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

    #[test]
    fn truncated_display_includes_ellipsis_marker() {
        // When display is truncated but there is room for a marker (ref alone
        // does not exhaust the budget), the display includes an explicit "..."
        // truncation marker while the ref stays complete.
        let entries = vec![
            Entry {
                ref_id: "H:L1:H1:I1".to_owned(),
                display: "A very long display text that should be truncated".to_owned(),
            },
            Entry {
                ref_id: "X".to_owned(),
                display: "next".to_owned(),
            },
        ];

        let (page, next) = paginate_entries(&entries, positive(1), positive(30));
        assert_eq!(page.len(), 1);
        assert_eq!(page[0].ref_id, "H:L1:H1:I1");
        assert!(
            page[0].display.ends_with("..."),
            "truncated display must end with '...' marker, got: '{}'",
            page[0].display
        );
        // Pagination advances because there are more entries.
        assert_eq!(next, Some(positive(2)));

        // Verify the total cost (ref + display) fits within limit.
        let cost = page[0].ref_id.chars().count() + page[0].display.chars().count();
        assert!(cost <= 30, "truncated entry cost {cost} exceeds limit 30");
    }

    #[test]
    fn unicode_truncated_display_includes_marker() {
        // Unicode characters are counted correctly; marker still appears when
        // there is room after the ref.
        let entries = vec![
            Entry {
                ref_id: "H:L1:H1:I1".to_owned(),
                display: "界世界世界世界世界世界世界世界世界世界世".to_owned(),
            },
            Entry {
                ref_id: "X".to_owned(),
                display: "next".to_owned(),
            },
        ];

        let (page, next) = paginate_entries(&entries, positive(1), positive(25));
        assert_eq!(page.len(), 1);
        assert_eq!(page[0].ref_id, "H:L1:H1:I1");
        assert!(
            page[0].display.ends_with("..."),
            "unicode truncated display must end with '...' marker"
        );
        // Pagination advances because there are more entries.
        assert_eq!(next, Some(positive(2)));

        let cost = page[0].ref_id.chars().count() + page[0].display.chars().count();
        assert!(
            cost <= 25,
            "unicode truncated entry cost {cost} exceeds limit 25"
        );
    }

    #[test]
    fn tiny_budget_no_room_for_marker_still_truncates() {
        // When the display budget is too small for a "...", we still truncate
        // but may not include the marker. With two entries, pagination still
        // advances.
        let entries = vec![
            Entry {
                ref_id: "H:L1:H1:I1".to_owned(),
                display: "a very long display text".to_owned(),
            },
            Entry {
                ref_id: "X".to_owned(),
                display: "next".to_owned(),
            },
        ];

        // ref = 9 chars, limit = 11, so display_budget = 2.
        let (page, next) = paginate_entries(&entries, positive(1), positive(11));
        assert_eq!(page.len(), 1);
        assert_eq!(page[0].ref_id, "H:L1:H1:I1");
        assert!(!page[0].display.is_empty(), "display should not be empty");

        let cost = page[0].ref_id.chars().count() + page[0].display.chars().count();
        assert!(cost <= 11, "tiny budget entry cost {cost} exceeds limit 11");

        // With budget=2 and no marker room, pagination still advances to the next entry.
        assert_eq!(next, Some(positive(2)));
    }
}
