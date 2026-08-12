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

#[test]
fn requested_text_cost_dispatches_one_unit() {
    let text = "one\n二";

    for (unit, expected) in [
        (CostUnit::Lines, line_cost(text)),
        (CostUnit::Bytes, byte_cost(text)),
        (CostUnit::Tokens, token_cost(text)),
    ] {
        assert_eq!(text_cost(unit, text), expected);
    }
}

#[test]
fn bounded_meter_matches_logically_joined_fragments() {
    let cases = [
        (CostUnit::Lines, vec!["", "one", "\n", "二", "\n"]),
        (CostUnit::Bytes, vec!["a", "", "界", "\n"]),
        (CostUnit::Tokens, vec!["", "pl", "ain ", "te", "xt"]),
    ];

    for (unit, fragments) in cases {
        let joined = fragments.concat();
        let expected = text_cost(unit, &joined).value;
        let mut meter = TextMeter::new(unit, expected);
        for fragment in fragments {
            assert_eq!(meter.consume(fragment), TextMeterFlow::Continue);
        }
        assert_eq!(meter.finish(), BoundedTextCost::Fits(expected));
    }
}

#[test]
fn bounded_meter_stops_only_after_proven_exceed() {
    let mut bytes = TextMeter::new(CostUnit::Bytes, 3);
    assert_eq!(bytes.consume("abc"), TextMeterFlow::Continue);
    assert_eq!(bytes.consume("界"), TextMeterFlow::ProvenExceedsThreshold);
    assert_eq!(bytes.finish(), BoundedTextCost::ExceedsThreshold);

    let mut lines = TextMeter::new(CostUnit::Lines, 1);
    assert_eq!(lines.consume("one"), TextMeterFlow::Continue);
    assert_eq!(
        lines.consume("\nignored"),
        TextMeterFlow::ProvenExceedsThreshold
    );
    assert_eq!(lines.finish(), BoundedTextCost::ExceedsThreshold);

    let mut tokens = TextMeter::new(CostUnit::Tokens, 1);
    assert_eq!(tokens.consume("plain"), TextMeterFlow::Continue);
    assert_eq!(tokens.consume(" text"), TextMeterFlow::Continue);
    assert_eq!(tokens.finish(), BoundedTextCost::ExceedsThreshold);
}
