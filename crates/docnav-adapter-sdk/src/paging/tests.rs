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
