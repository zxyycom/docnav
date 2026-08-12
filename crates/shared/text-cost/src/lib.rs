use docnav_protocol::{CostUnit, Measurement};
use tiktoken_rs::o200k_base_singleton;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use = "the meter flow determines whether another fragment can affect the result"]
pub enum TextMeterFlow {
    Continue,
    ProvenExceedsThreshold,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundedTextCost {
    Fits(u64),
    ExceedsThreshold,
}

#[derive(Debug)]
pub struct TextMeter {
    threshold: u64,
    state: MeterState,
}

#[derive(Debug)]
enum MeterState {
    Lines { has_text: bool, value: u64 },
    Bytes { value: u64 },
    Tokens { text: String },
    Exceeded,
}

impl TextMeter {
    pub fn new(unit: CostUnit, threshold: u64) -> Self {
        let state = match unit {
            CostUnit::Lines => MeterState::Lines {
                has_text: false,
                value: 0,
            },
            CostUnit::Bytes => MeterState::Bytes { value: 0 },
            CostUnit::Tokens => MeterState::Tokens {
                text: String::new(),
            },
        };
        Self { threshold, state }
    }

    pub fn consume(&mut self, fragment: &str) -> TextMeterFlow {
        if matches!(self.state, MeterState::Exceeded) {
            return TextMeterFlow::ProvenExceedsThreshold;
        }

        if self.state.consume(fragment, self.threshold) {
            self.state = MeterState::Exceeded;
            TextMeterFlow::ProvenExceedsThreshold
        } else {
            TextMeterFlow::Continue
        }
    }

    pub fn finish(self) -> BoundedTextCost {
        let value = match self.state {
            MeterState::Exceeded => return BoundedTextCost::ExceedsThreshold,
            MeterState::Lines { value, .. } | MeterState::Bytes { value } => value,
            MeterState::Tokens { text } => {
                let tokens = o200k_base_singleton().count_ordinary(&text);
                let Ok(tokens) = u64::try_from(tokens) else {
                    return BoundedTextCost::ExceedsThreshold;
                };
                tokens
            }
        };

        if value <= self.threshold {
            BoundedTextCost::Fits(value)
        } else {
            BoundedTextCost::ExceedsThreshold
        }
    }
}

impl MeterState {
    fn consume(&mut self, fragment: &str, threshold: u64) -> bool {
        match self {
            Self::Lines { has_text, value } => {
                consume_line_fragment(has_text, value, fragment, threshold)
            }
            Self::Bytes { value } => consume_byte_fragment(value, fragment, threshold),
            Self::Tokens { text } => {
                text.push_str(fragment);
                false
            }
            Self::Exceeded => true,
        }
    }
}

fn consume_line_fragment(
    has_text: &mut bool,
    value: &mut u64,
    fragment: &str,
    threshold: u64,
) -> bool {
    for byte in fragment.bytes() {
        let increment = u64::from(!*has_text) + u64::from(byte == b'\n');
        *has_text = true;
        match value.checked_add(increment) {
            Some(next) if next <= threshold => *value = next,
            _ => return true,
        }
    }
    false
}

fn consume_byte_fragment(value: &mut u64, fragment: &str, threshold: u64) -> bool {
    let fragment_cost = u64::try_from(fragment.len());
    match fragment_cost
        .ok()
        .and_then(|fragment_cost| value.checked_add(fragment_cost))
    {
        Some(next) if next <= threshold => {
            *value = next;
            false
        }
        _ => true,
    }
}

pub fn text_cost(unit: CostUnit, text: &str) -> Measurement {
    let value = match unit {
        CostUnit::Lines => line_count(text),
        CostUnit::Bytes => text.len() as u64,
        CostUnit::Tokens => o200k_base_singleton().count_ordinary(text) as u64,
    };
    measurement(unit, value)
}

pub fn line_cost(text: &str) -> Measurement {
    text_cost(CostUnit::Lines, text)
}

pub fn byte_cost(text: &str) -> Measurement {
    text_cost(CostUnit::Bytes, text)
}

pub fn token_cost(text: &str) -> Measurement {
    text_cost(CostUnit::Tokens, text)
}

fn measurement(unit: CostUnit, value: u64) -> Measurement {
    Measurement {
        unit: unit.to_string(),
        value,
        scope: None,
    }
}

fn line_count(text: &str) -> u64 {
    if text.is_empty() {
        0
    } else {
        text.bytes().filter(|byte| *byte == b'\n').count() as u64 + 1
    }
}

#[cfg(test)]
mod tests;
