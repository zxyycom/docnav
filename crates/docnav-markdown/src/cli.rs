use std::io::{self, Read, Write};

use docnav_adapter_sdk::{
    run_direct_cli, DirectCliConfig, DirectTextFormatter, NativeOptionDefault, NativeOptionSpec,
    NativeOptionValueSpec,
};
use docnav_protocol::{Operation, OperationResult};

use crate::adapter::{
    MarkdownAdapter, DEFAULT_LIMIT_CHARS, DEFAULT_MAX_HEADING_LEVEL, MAX_HEADING_LEVEL_OPTION,
};

const REQUEST_ID: &str = "docnav-markdown-cli";
const USAGE: &str = "usage: docnav-markdown <outline|read|find|info|manifest|probe|invoke> ...";
const MAX_HEADING_LEVEL_OPERATIONS: &[Operation] = &[Operation::Outline, Operation::Find];
const NATIVE_OPTIONS: &[NativeOptionSpec] = &[NativeOptionSpec {
    flag: "--max-heading-level",
    option_key: MAX_HEADING_LEVEL_OPTION,
    operations: MAX_HEADING_LEVEL_OPERATIONS,
    value: NativeOptionValueSpec::IntegerRange { min: 1, max: 6 },
    default: Some(NativeOptionDefault::Integer(
        DEFAULT_MAX_HEADING_LEVEL as u64,
    )),
}];

pub fn run<I, R, W, E>(args: I, stdin: R, stdout: W, stderr: E) -> i32
where
    I: IntoIterator<Item = String>,
    R: Read,
    W: Write,
    E: Write,
{
    let adapter = MarkdownAdapter;
    run_direct_cli(
        &adapter,
        args,
        stdin,
        stdout,
        stderr,
        DirectCliConfig {
            program_name: "docnav-markdown",
            usage: USAGE,
            request_id: REQUEST_ID,
            default_limit_chars: DEFAULT_LIMIT_CHARS,
            native_options: NATIVE_OPTIONS,
            text_formatter: MarkdownTextOutput,
        },
    )
}

#[derive(Clone, Copy, Debug, Default)]
struct MarkdownTextOutput;

impl DirectTextFormatter for MarkdownTextOutput {
    fn write_text_result<W: Write>(
        &self,
        result: &OperationResult,
        stdout: &mut W,
    ) -> io::Result<()> {
        match result {
            OperationResult::Outline(result) => {
                for entry in &result.entries {
                    writeln!(stdout, "{} | {}", entry.ref_id, entry.display)?;
                }
                writeln!(stdout, "page: {}", page_label(result.page))
            }
            OperationResult::Read(result) => {
                writeln!(stdout, "ref: {}", result.ref_id)?;
                write!(stdout, "{}", result.content)?;
                if !result.content.ends_with('\n') {
                    writeln!(stdout)?;
                }
                writeln!(stdout, "content_type: {}", result.content_type)?;
                writeln!(stdout, "cost: {}", result.cost)?;
                writeln!(stdout, "page: {}", page_label(result.page))
            }
            OperationResult::Find(result) => {
                for entry in &result.matches {
                    writeln!(stdout, "{} | {}", entry.ref_id, entry.display)?;
                }
                writeln!(stdout, "page: {}", page_label(result.page))
            }
            OperationResult::Info(result) => {
                writeln!(stdout, "{}", result.display)?;
                writeln!(
                    stdout,
                    "capabilities: {}",
                    result
                        .capabilities
                        .iter()
                        .map(Operation::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
    }
}

fn page_label(page: Option<docnav_protocol::PositiveInteger>) -> String {
    page.map(|page| page.get().to_string())
        .unwrap_or_else(|| "null".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ID: AtomicU64 = AtomicU64::new(1);

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
    fn text_output_write_failure_reports_diagnostic() {
        let path = write_doc("stdout-failure.md", "# Guide\n");
        let args = vec![
            "outline".to_owned(),
            path.to_string_lossy().into_owned(),
            "--output".to_owned(),
            "text".to_owned(),
        ];
        let mut stdout = FailingWriter;
        let mut stderr = Vec::new();

        let exit = run(args, io::empty(), &mut stdout, &mut stderr);

        assert_eq!(exit, docnav_adapter_sdk::AdapterExitCode::IoError.code());
        let stderr = String::from_utf8(stderr).expect("stderr utf8");
        assert!(stderr.contains("failed to write text output"));
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
}
