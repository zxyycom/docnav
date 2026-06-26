// @case WB-MD-CLI-WRITE-001
use super::*;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

struct ClosedStdout;

impl Write for ClosedStdout {
    fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
        Err(closed_stdout())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn closed_stdout() -> io::Error {
    io::Error::new(io::ErrorKind::BrokenPipe, "stdout closed")
}

#[test]
fn readable_view_output_write_failure_reports_diagnostic() {
    let path = write_doc("stdout-failure.md", "# Guide\n");
    let args = vec!["outline".to_owned(), path.to_string_lossy().into_owned()];
    let mut stdout = ClosedStdout;
    let mut stderr = Vec::new();

    let exit = run(args, io::empty(), &mut stdout, &mut stderr);

    assert_eq!(exit, docnav_adapter_sdk::AdapterExitCode::IoError.code());
    let stderr = String::from_utf8(stderr).expect("stderr utf8");
    assert!(stderr.contains("failed to write readable-view output"));
    assert!(stderr.contains("stdout closed"));
}

fn write_doc(name: &str, content: &str) -> PathBuf {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "docnav-markdown-cli-unit-test-{}-{}",
        std::process::id(),
        id
    ));
    fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join(name);
    fs::write(&path, content).expect("write temp document");
    path
}
