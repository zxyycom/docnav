// @case WB-MD-PAGING-DISPLAY-001
use super::*;
use docnav_protocol::positive_result;

fn positive(value: u32) -> PositiveInteger {
    positive_result(value).expect("positive test integer")
}

fn entry(ref_id: &str, label: &str) -> Entry {
    Entry {
        ref_id: ref_id.to_owned(),
        label: label.to_owned(),
        kind: None,
        location: None,
        summary: None,
        excerpt: None,
        rank: None,
        cost: None,
        metadata: None,
    }
}

#[test]
fn read_paging_counts_unicode_characters() {
    let (page, next) = paginate_text("界abc", positive(1), positive(3));
    assert_eq!(page, "界ab");
    assert_eq!(next, Some(positive(2)));
}

#[test]
fn entry_paging_preserves_ref_and_truncates_display() {
    let entries = vec![entry("R", "abcdef"), entry("N", "next")];

    let (page, next) = paginate_entries(&entries, positive(1), positive(5));
    assert_eq!(page.len(), 1);
    assert_eq!(page[0].ref_id, "R");
    assert_eq!(page[0].label, "a...");
    assert_eq!(next, Some(positive(2)));
}
