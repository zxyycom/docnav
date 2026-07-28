use super::*;

fn assert_measurement(measurement: Measurement, unit: &str, value: u64) {
    assert_eq!(measurement.unit, unit);
    assert_eq!(measurement.value, value);
    assert_eq!(measurement.scope, None);
}

#[test]
fn line_cost_counts_empty_unicode_and_trailing_newline() {
    assert_measurement(line_cost(""), "lines", 0);
    assert_measurement(line_cost("one\n二\n"), "lines", 3);
}

#[test]
fn byte_cost_counts_utf8_bytes() {
    let text = "a界\n";

    assert_measurement(byte_cost(text), "bytes", text.len() as u64);
}

#[test]
fn token_cost_uses_o200k_base_ordinary_text() {
    let cases = [("", 0), ("plain text", 2), ("<|endoftext|>\nplain text", 9)];

    for (text, expected) in cases {
        assert_measurement(token_cost(text), "tokens", expected);
    }
}
