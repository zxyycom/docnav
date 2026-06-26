// @case WB-MD-PAGING-DISPLAY-001
use super::*;
use docnav_protocol::positive_result;

fn positive(value: u32) -> PositiveInteger {
    positive_result(value).expect("positive test integer")
}

#[test]
fn read_paging_counts_unicode_characters() {
    let (page, next) = paginate_text("界abc", positive(1), positive(3));
    assert_eq!(page, "界ab");
    assert_eq!(next, Some(positive(2)));
}

#[test]
fn entry_paging_preserves_ref_and_truncates_display() {
    let entries = vec![
        Entry {
            ref_id: "R".to_owned(),
            display: "abcdef".to_owned(),
        },
        Entry {
            ref_id: "N".to_owned(),
            display: "next".to_owned(),
        },
    ];

    let (page, next) = paginate_entries(&entries, positive(1), positive(5));
    assert_eq!(page.len(), 1);
    assert_eq!(page[0].ref_id, "R");
    assert_eq!(page[0].display, "a...");
    assert_eq!(next, Some(positive(2)));
}
