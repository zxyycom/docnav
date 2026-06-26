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
mod tests;
