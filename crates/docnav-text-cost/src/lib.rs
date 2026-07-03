use docnav_protocol::Measurement;
use tiktoken_rs::o200k_base_singleton;

pub fn line_cost(text: &str) -> Measurement {
    measurement("lines", line_count(text))
}

pub fn byte_cost(text: &str) -> Measurement {
    measurement("bytes", text.len() as u64)
}

pub fn token_cost(text: &str) -> Measurement {
    let tokens = o200k_base_singleton().count_ordinary(text);
    measurement("tokens", tokens as u64)
}

fn measurement(unit: &str, value: u64) -> Measurement {
    Measurement {
        unit: unit.to_owned(),
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
