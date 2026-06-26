// @case WB-JSONIO-WRITE-001
use super::*;
use serde::ser::{Error as _, SerializeStruct};
use serde::Serializer;
use serde_json::json;

struct FailingSerialize;

impl Serialize for FailingSerialize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("FailingSerialize", 1)?;
        state.serialize_field("field", &AlwaysFails)?;
        state.end()
    }
}

struct AlwaysFails;

impl Serialize for AlwaysFails {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Err(S::Error::custom("synthetic serialization failure"))
    }
}

struct FailingWriter;

impl Write for FailingWriter {
    fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "stdout closed"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn compact_json_writes_value_and_newline() {
    let mut output = Vec::new();
    write_json_value_compact(&json!({"ok": true}), &mut output).unwrap();
    assert_eq!(
        output,
        br#"{"ok":true}"#.iter().copied().chain([b'\n']).collect::<Vec<_>>()
    );
}

#[test]
fn pretty_json_writes_value_and_newline() {
    let mut output = Vec::new();
    write_json_value_pretty(&json!({"ok": true}), &mut output).unwrap();
    let output = String::from_utf8(output).unwrap();
    assert!(output.ends_with('\n'));
    assert!(output.contains("\"ok\": true"));
}

#[test]
fn serialization_failures_are_distinct_from_write_failures() {
    let mut output = Vec::new();
    let error = write_json_value_compact(&FailingSerialize, &mut output).unwrap_err();
    assert!(matches!(error, JsonIoError::Serialization(_)));
    assert!(output.is_empty());
}

#[test]
fn write_failures_are_reported() {
    let mut output = FailingWriter;
    let error = write_json_value_compact(&json!({"ok": true}), &mut output).unwrap_err();
    assert!(matches!(error, JsonIoError::Write(_)));
}
