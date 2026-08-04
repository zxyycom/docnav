use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use serde_json::Value;

pub(super) fn append_json_line(path: &Path, event: &Value) -> std::io::Result<()> {
    let mut line = serde_json::to_vec(event).map_err(std::io::Error::other)?;
    line.push(b'\n');
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut options = OpenOptions::new();
    options.create(true).append(true);
    // Windows cannot lock an append-only handle; read + append preserves append semantics.
    #[cfg(windows)]
    options.read(true);
    let mut file = options.open(path)?;
    file.lock()?;
    file.write_all(&line)
}

pub(super) fn write_content_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)
}
