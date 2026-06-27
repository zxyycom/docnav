// @case WB-SDK-DIRECT-OUTPUT-001
use super::*;
use docnav_protocol::{OperationResult, ReadResult};

struct FailingStdout {
    attempted: bool,
}

impl Write for FailingStdout {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.attempted = true;
        assert!(
            !buffer.is_empty(),
            "stdout write should carry rendered output"
        );
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "stdout closed"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn read_result() -> OperationResult {
    OperationResult::Read(ReadResult {
        ref_id: "ok".into(),
        content: "body".into(),
        content_type: "text/plain".into(),
        cost: "1 lines | 4 bytes".into(),
        page: None,
    })
}

#[test]
fn readable_view_stdout_write_failure_is_io_error_with_diagnostic() {
    let mut stdout = FailingStdout { attempted: false };
    let mut stderr = Vec::new();

    let exit = write_operation_output(
        read_result(),
        DirectOutputMode::ReadableView,
        &[],
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, AdapterExitCode::IoError.code());
    assert!(
        stdout.attempted,
        "rendered readable-view should be written to stdout"
    );
    let stderr = String::from_utf8(stderr).expect("stderr utf8");
    assert!(stderr.contains("failed to write readable-view output"));
    assert!(stderr.contains("stdout closed"));
}

#[test]
fn readable_json_success_uses_shared_document_output() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = write_operation_output(
        read_result(),
        DirectOutputMode::ReadableJson,
        &[],
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, AdapterExitCode::Success.code());
    assert!(stderr.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(value["ref"], "ok");
    assert!(value.get("protocol_version").is_none());
}

#[test]
fn direct_cli_warning_flush_projects_diagnostic_records_to_stderr() {
    let mut stderr = Vec::new();
    let warning = DirectCliWarning::unknown_flag("--future");

    let exit =
        append_cli_warnings_to_stderr(AdapterExitCode::Success.code(), &[warning], &mut stderr);

    assert_eq!(exit, AdapterExitCode::Success.code());
    let stderr = String::from_utf8(stderr).expect("stderr utf8");
    assert!(stderr.contains("cli_argv_ignored"));
    assert!(stderr.contains("--future"));
}
