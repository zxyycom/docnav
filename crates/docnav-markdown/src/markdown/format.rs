use std::fs;
use std::path::Path;

pub fn is_markdown_extension(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            let extension = extension.to_ascii_lowercase();
            extension == "md" || extension == "markdown"
        })
        .unwrap_or(false)
}

pub fn is_utf8_markdown_candidate(path: &str) -> Result<bool, std::io::Error> {
    let bytes = fs::read(path)?;
    let bytes = bytes
        .strip_prefix(&[0xEF, 0xBB, 0xBF])
        .unwrap_or(bytes.as_slice());
    Ok(std::str::from_utf8(bytes).is_ok())
}
