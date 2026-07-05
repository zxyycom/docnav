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
mod tests;
