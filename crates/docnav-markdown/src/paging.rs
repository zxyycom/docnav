use docnav_adapter_sdk::paging as sdk_paging;
use docnav_protocol::{Entry, PositiveInteger};

pub fn paginate_text(
    content: &str,
    page: PositiveInteger,
    limit: PositiveInteger,
) -> (String, Option<PositiveInteger>) {
    sdk_paging::paginate_text(content, page, limit)
}

pub fn paginate_entries(
    entries: &[Entry],
    page: PositiveInteger,
    limit: PositiveInteger,
) -> (Vec<Entry>, Option<PositiveInteger>) {
    sdk_paging::paginate_entries(entries, page, limit)
}

#[cfg(test)]
mod tests;
