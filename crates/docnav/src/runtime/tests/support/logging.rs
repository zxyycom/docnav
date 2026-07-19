use std::fs;
use std::io::{self, Write};
use std::path::Path;

use serde_json::Value;
use sha2::{Digest, Sha256};

pub(in crate::runtime::tests) fn read_jsonl_events(path: &Path) -> Vec<Value> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

pub(in crate::runtime::tests) fn event_named<'a>(events: &'a [Value], event: &str) -> &'a Value {
    events
        .iter()
        .find(|value| value["event"] == event)
        .unwrap_or_else(|| panic!("missing event {event}: {events:#?}"))
}

pub(in crate::runtime::tests) fn is_lower_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(in crate::runtime::tests) fn test_sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut text = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(text, "{byte:02x}");
    }
    text
}

pub(in crate::runtime::tests) struct FailingWriter;

impl Write for FailingWriter {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("stdout closed"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(in crate::runtime::tests) struct LogAbsentWriter<'a> {
    log_path: &'a Path,
    bytes: Vec<u8>,
}

impl<'a> LogAbsentWriter<'a> {
    pub(in crate::runtime::tests) fn new(log_path: &'a Path) -> Self {
        Self {
            log_path,
            bytes: Vec::new(),
        }
    }

    pub(in crate::runtime::tests) fn into_string(self) -> String {
        String::from_utf8(self.bytes).unwrap()
    }
}

impl Write for LogAbsentWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        assert!(
            !self.log_path.exists(),
            "output failure log must be written after fallback output error projection"
        );
        self.bytes.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
