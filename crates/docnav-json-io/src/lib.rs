use std::fmt;
use std::io::{self, Write};

use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JsonFormat {
    Compact,
    Pretty,
}

#[derive(Debug)]
pub enum JsonIoError {
    Serialization(serde_json::Error),
    Write(io::Error),
}

impl fmt::Display for JsonIoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialization(error) => write!(formatter, "JSON serialization failed: {error}"),
            Self::Write(error) => write!(formatter, "JSON write failed: {error}"),
        }
    }
}

impl std::error::Error for JsonIoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Serialization(error) => Some(error),
            Self::Write(error) => Some(error),
        }
    }
}

pub fn write_json_value<W, T>(
    value: &T,
    writer: &mut W,
    format: JsonFormat,
) -> Result<(), JsonIoError>
where
    W: Write,
    T: Serialize + ?Sized,
{
    let bytes = match format {
        JsonFormat::Compact => serde_json::to_vec(value),
        JsonFormat::Pretty => serde_json::to_vec_pretty(value),
    }
    .map_err(JsonIoError::Serialization)?;
    write_json_bytes_with_newline(&bytes, writer)
}

pub fn write_json_bytes_with_newline<W: Write>(
    bytes: &[u8],
    writer: &mut W,
) -> Result<(), JsonIoError> {
    writer.write_all(bytes).map_err(JsonIoError::Write)?;
    writer.write_all(b"\n").map_err(JsonIoError::Write)
}

pub fn write_json_value_pretty<W, T>(value: &T, writer: &mut W) -> Result<(), JsonIoError>
where
    W: Write,
    T: Serialize + ?Sized,
{
    write_json_value(value, writer, JsonFormat::Pretty)
}

pub fn write_json_value_compact<W, T>(value: &T, writer: &mut W) -> Result<(), JsonIoError>
where
    W: Write,
    T: Serialize + ?Sized,
{
    write_json_value(value, writer, JsonFormat::Compact)
}

#[cfg(test)]
mod tests {
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
}
