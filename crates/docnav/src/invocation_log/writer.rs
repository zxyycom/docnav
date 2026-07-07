use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use serde_json::Value;

pub(super) fn append_json_line(path: &Path, event: &Value) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    serde_json::to_writer(&mut file, event).map_err(std::io::Error::other)?;
    file.write_all(b"\n")
}

pub(super) fn write_content_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)
}
