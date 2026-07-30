use super::*;
use crate::content::structured_value_facts;
use crate::document::{load, JsonKind};
use crate::find::FindEntry;
use docnav_protocol::{positive_result, Location, PositiveInteger};

fn positive(value: u32) -> PositiveInteger {
    positive_result(value).expect("positive test integer")
}

#[test]
fn text_pages_reassemble_unicode_scalar_content_and_keep_full_cost() {
    let document =
        load(r#""雪🍣界🙂文档导航""#.as_bytes()).expect("Unicode root scalar should load as JSON");
    let complete =
        structured_value_facts(&document.root).expect("root scalar should serialize as JSON");
    let expected_content = complete.content.clone();
    let expected_cost = complete.cost.clone();
    let limit = positive(2);
    let mut requested_page = positive(1);
    let mut reconstructed = String::new();
    let mut page_count = 0;

    loop {
        let facts =
            structured_value_facts(&document.root).expect("root scalar should serialize as JSON");
        let result = paginate_text(facts, requested_page, limit);

        assert_eq!(result.cost, expected_cost);
        assert!(result.content.chars().count() <= limit.get() as usize);
        reconstructed.push_str(&result.content);
        page_count += 1;

        let Some(next_page) = result.page else {
            break;
        };
        assert_eq!(next_page.get(), requested_page.get() + 1);
        requested_page = next_page;
    }

    assert!(page_count > 1);
    assert_eq!(reconstructed, expected_content);
}

#[test]
fn entry_pages_with_tiny_limit_preserve_long_refs_and_make_progress() {
    let document = load(r#"{"a/very-long-雪-key": 1, "next": 2}"#.as_bytes())
        .expect("entry pagination fixture should load");
    let expected = document.preorder_entries();
    let limit = positive(1);
    let mut requested_page = positive(1);
    let mut actual = Vec::new();

    loop {
        let (entries, next_page) = paginate_entries(&expected, requested_page, limit);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].ref_id, expected[actual.len()].ref_id);
        assert_eq!(entries[0].label, ".");
        assert_eq!(entries[0].kind, JsonKind::Number);
        actual.push(entries[0].ref_id.clone());

        let Some(next_page) = next_page else {
            break;
        };
        assert_eq!(next_page.get(), requested_page.get() + 1);
        requested_page = next_page;
    }

    assert_eq!(
        actual,
        expected
            .iter()
            .map(|entry| entry.ref_id.clone())
            .collect::<Vec<_>>()
    );
}

#[test]
fn empty_and_past_end_pages_are_terminal() {
    let document = load(r#""雪""#.as_bytes()).expect("root scalar should load as JSON");
    let facts =
        structured_value_facts(&document.root).expect("root scalar should serialize as JSON");
    let expected_cost = facts.cost.clone();

    let text_page = paginate_text(facts, positive(99), positive(1));
    assert_eq!(text_page.content, "");
    assert_eq!(text_page.cost, expected_cost);
    assert_eq!(text_page.page, None);

    let (empty_entries, empty_next_page) = paginate_entries(&[], positive(1), positive(1));
    assert!(empty_entries.is_empty());
    assert_eq!(empty_next_page, None);

    let entries = document.preorder_entries();
    let (past_end_entries, past_end_next_page) =
        paginate_entries(&entries, positive(2), positive(100));
    assert!(past_end_entries.is_empty());
    assert_eq!(past_end_next_page, None);
}

#[test]
fn find_entry_pages_preserve_occurrences_facts_and_terminal_semantics() {
    let document = load(b"{\n  \"value\": \"hit hit\",\n  \"tail\": \"hit\"\n}")
        .expect("find pagination fixture should load");
    let expected = document.find_entries("hit").collect::<Vec<_>>();

    assert_eq!(
        expected
            .iter()
            .map(|entry| entry.ref_id.as_str())
            .collect::<Vec<_>>(),
        ["json:#/value", "json:#/value", "json:#/tail"]
    );

    let (complete_page, complete_next_page) =
        paginate_find_entries(expected.clone().into_iter(), positive(1), positive(1_000));
    assert_eq!(complete_page, expected);
    assert_eq!(complete_next_page, None);

    let limit = positive(1);
    let mut requested_page = positive(1);
    let mut actual = Vec::<FindEntry>::new();

    loop {
        let (entries, next_page) =
            paginate_find_entries(expected.clone().into_iter(), requested_page, limit);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].ref_id, expected[actual.len()].ref_id);
        assert_eq!(entries[0].label, ".");
        assert_eq!(entries[0].location, expected[actual.len()].location);
        actual.push(entries[0].clone());

        let Some(next_page) = next_page else {
            break;
        };
        assert_eq!(next_page.get(), requested_page.get() + 1);
        requested_page = next_page;
    }

    assert_eq!(
        actual
            .iter()
            .map(|entry| entry.ref_id.as_str())
            .collect::<Vec<_>>(),
        ["json:#/value", "json:#/value", "json:#/tail"]
    );

    let (empty_entries, empty_next_page) =
        paginate_find_entries(std::iter::empty(), positive(1), positive(1));
    assert!(empty_entries.is_empty());
    assert_eq!(empty_next_page, None);

    let (past_end_entries, past_end_next_page) =
        paginate_find_entries(expected.into_iter(), positive(2), positive(1_000));
    assert!(past_end_entries.is_empty());
    assert_eq!(past_end_next_page, None);
}

#[test]
fn find_entry_pagination_pulls_only_the_current_page_and_lookahead() {
    let pulls = std::cell::Cell::new(0_usize);
    let entries = std::iter::from_fn(|| {
        let index = pulls.get();
        (index < 10).then(|| {
            pulls.set(index + 1);
            FindEntry {
                ref_id: format!("json:#/{index}"),
                label: "match".to_owned(),
                location: Location {
                    line_start: positive(1),
                    line_end: None,
                },
            }
        })
    });

    let (page, next_page) = paginate_find_entries(entries, positive(1), positive(1));

    assert_eq!(page.len(), 1);
    assert_eq!(page[0].ref_id, "json:#/0");
    assert_eq!(page[0].label, ".");
    assert_eq!(next_page, Some(positive(2)));
    assert_eq!(
        pulls.get(),
        2,
        "pagination should pull one entry for the page and one to prove continuation"
    );
}
