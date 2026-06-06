use std::io::{Read, Write};

use docnav_adapter_sdk::{emit_diagnostic, invoke_once, run_command, Adapter, SdkCommand};
use docnav_protocol::{
    positive_result, Document, ErrorDetails, InfoArguments, Operation, OperationArguments,
    OperationResult, ProtocolResponse, RequestEnvelope, StableErrorCode, PROTOCOL_VERSION,
};
use serde::Serialize;

use crate::adapter::{
    direct_find_arguments, direct_outline_arguments, MarkdownAdapter, DEFAULT_LIMIT_CHARS,
    DEFAULT_MAX_HEADING_LEVEL,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputMode {
    Text,
    ReadableJson,
    ProtocolJson,
}

#[derive(Clone, Debug)]
struct OperationOptions {
    path: String,
    page: docnav_protocol::PositiveInteger,
    limit_chars: docnav_protocol::PositiveInteger,
    max_heading_level: u8,
    output: OutputMode,
    ref_id: Option<String>,
    query: Option<String>,
}

pub fn run<I, R, W, E>(args: I, stdin: R, mut stdout: W, mut stderr: E) -> i32
where
    I: IntoIterator<Item = String>,
    R: Read,
    W: Write,
    E: Write,
{
    let adapter = MarkdownAdapter;
    let args: Vec<String> = args.into_iter().collect();
    let Some(command) = args.first().map(String::as_str) else {
        return usage(&mut stderr);
    };

    match command {
        "manifest" => match parse_protocol_only_output(&args[1..]) {
            Ok(()) => run_command(
                &adapter,
                SdkCommand::Manifest,
                std::io::empty(),
                stdout,
                stderr,
            ),
            Err(message) => input_error(&mut stderr, &message),
        },
        "probe" => match parse_probe(&args[1..]) {
            Ok(path) => run_command(
                &adapter,
                SdkCommand::Probe { path },
                std::io::empty(),
                stdout,
                stderr,
            ),
            Err(message) => input_error(&mut stderr, &message),
        },
        "invoke" => {
            if args.len() != 1 {
                return input_error(&mut stderr, "invoke does not accept positional arguments");
            }
            run_command(&adapter, SdkCommand::Invoke, stdin, stdout, stderr)
        }
        "outline" => match parse_operation_options(Operation::Outline, &args[1..]) {
            Ok(options) => run_outline(&adapter, options, &mut stdout, &mut stderr),
            Err(message) => input_error(&mut stderr, &message),
        },
        "read" => match parse_operation_options(Operation::Read, &args[1..]) {
            Ok(options) => run_read(&adapter, options, &mut stdout, &mut stderr),
            Err(message) => input_error(&mut stderr, &message),
        },
        "find" => match parse_operation_options(Operation::Find, &args[1..]) {
            Ok(options) => run_find(&adapter, options, &mut stdout, &mut stderr),
            Err(message) => input_error(&mut stderr, &message),
        },
        "info" => match parse_operation_options(Operation::Info, &args[1..]) {
            Ok(options) => run_info(&adapter, options, &mut stdout, &mut stderr),
            Err(message) => input_error(&mut stderr, &message),
        },
        _ => usage(&mut stderr),
    }
}

fn run_outline<W: Write, E: Write>(
    adapter: &MarkdownAdapter,
    options: OperationOptions,
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    let arguments =
        direct_outline_arguments(options.limit_chars, options.page, options.max_heading_level);
    let request = request(
        Operation::Outline,
        options.path,
        OperationArguments::Outline(arguments.clone()),
    );

    if options.output == OutputMode::ProtocolJson {
        return invoke_request(adapter, &request, stdout, stderr);
    }

    match adapter.outline(&request, &arguments) {
        Ok(result) => write_operation_output(
            OperationResult::Outline(result),
            options.output,
            stdout,
            stderr,
        ),
        Err(error) => handler_error(error, options.output, stdout, stderr),
    }
}

fn run_read<W: Write, E: Write>(
    adapter: &MarkdownAdapter,
    options: OperationOptions,
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    let Some(ref_id) = options.ref_id else {
        return input_error(stderr, "read requires --ref <ref>");
    };
    let arguments = docnav_protocol::ReadArguments {
        ref_id,
        limit_chars: options.limit_chars,
        page: options.page,
        options: None,
    };
    let request = request(
        Operation::Read,
        options.path,
        OperationArguments::Read(arguments.clone()),
    );

    if options.output == OutputMode::ProtocolJson {
        return invoke_request(adapter, &request, stdout, stderr);
    }

    match adapter.read(&request, &arguments) {
        Ok(result) => write_operation_output(
            OperationResult::Read(result),
            options.output,
            stdout,
            stderr,
        ),
        Err(error) => handler_error(error, options.output, stdout, stderr),
    }
}

fn run_find<W: Write, E: Write>(
    adapter: &MarkdownAdapter,
    options: OperationOptions,
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    let Some(query) = options.query else {
        return input_error(stderr, "find requires --query <text>");
    };
    let arguments = direct_find_arguments(
        query,
        options.limit_chars,
        options.page,
        options.max_heading_level,
    );
    let request = request(
        Operation::Find,
        options.path,
        OperationArguments::Find(arguments.clone()),
    );

    if options.output == OutputMode::ProtocolJson {
        return invoke_request(adapter, &request, stdout, stderr);
    }

    match adapter.find(&request, &arguments) {
        Ok(result) => write_operation_output(
            OperationResult::Find(result),
            options.output,
            stdout,
            stderr,
        ),
        Err(error) => handler_error(error, options.output, stdout, stderr),
    }
}

fn run_info<W: Write, E: Write>(
    adapter: &MarkdownAdapter,
    options: OperationOptions,
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    let arguments = InfoArguments { options: None };
    let request = request(
        Operation::Info,
        options.path,
        OperationArguments::Info(arguments.clone()),
    );

    if options.output == OutputMode::ProtocolJson {
        return invoke_request(adapter, &request, stdout, stderr);
    }

    match adapter.info(&request, &arguments) {
        Ok(result) => write_operation_output(
            OperationResult::Info(result),
            options.output,
            stdout,
            stderr,
        ),
        Err(error) => handler_error(error, options.output, stdout, stderr),
    }
}

fn request(operation: Operation, path: String, arguments: OperationArguments) -> RequestEnvelope {
    RequestEnvelope {
        protocol_version: PROTOCOL_VERSION.to_owned(),
        request_id: "docnav-markdown-cli".to_owned(),
        operation,
        document: Document { path },
        arguments,
    }
}

fn invoke_request<W: Write, E: Write>(
    adapter: &MarkdownAdapter,
    request: &RequestEnvelope,
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    let input = match serde_json::to_vec(request) {
        Ok(input) => input,
        Err(error) => {
            let _ = emit_diagnostic(stderr, &format!("failed to serialize request: {error}"));
            return docnav_adapter_sdk::AdapterExitCode::InternalError.code();
        }
    };
    invoke_once(adapter, input.as_slice(), stdout, stderr)
}

fn write_operation_output<W: Write, E: Write>(
    result: OperationResult,
    output: OutputMode,
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    match output {
        OutputMode::Text => write_text_result(&result, stdout, stderr),
        OutputMode::ReadableJson => write_json_result(&result, stdout, stderr),
        OutputMode::ProtocolJson => unreachable!("protocol-json is handled before dispatch"),
    }
}

fn write_json_result<W: Write, E: Write, T: Serialize>(
    result: &T,
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    match serde_json::to_writer(stdout, result) {
        Ok(()) => docnav_adapter_sdk::AdapterExitCode::Success.code(),
        Err(error) => {
            let _ = emit_diagnostic(stderr, &format!("failed to write JSON: {error}"));
            docnav_adapter_sdk::AdapterExitCode::IoError.code()
        }
    }
}

fn write_text_result<W: Write, E: Write>(
    result: &OperationResult,
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    let write_result = match result {
        OperationResult::Outline(result) => {
            for entry in &result.entries {
                if writeln!(stdout, "{} | {}", entry.ref_id, entry.display).is_err() {
                    return docnav_adapter_sdk::AdapterExitCode::IoError.code();
                }
            }
            writeln!(stdout, "page: {}", page_label(result.page))
        }
        OperationResult::Read(result) => {
            if writeln!(stdout, "ref: {}", result.ref_id).is_err() {
                return docnav_adapter_sdk::AdapterExitCode::IoError.code();
            }
            if write!(stdout, "{}", result.content).is_err() {
                return docnav_adapter_sdk::AdapterExitCode::IoError.code();
            }
            if !result.content.ends_with('\n') && writeln!(stdout).is_err() {
                return docnav_adapter_sdk::AdapterExitCode::IoError.code();
            }
            writeln!(stdout, "content_type: {}", result.content_type)
                .and_then(|_| writeln!(stdout, "cost: {}", result.cost))
                .and_then(|_| writeln!(stdout, "page: {}", page_label(result.page)))
        }
        OperationResult::Find(result) => {
            for entry in &result.matches {
                if writeln!(stdout, "{} | {}", entry.ref_id, entry.display).is_err() {
                    return docnav_adapter_sdk::AdapterExitCode::IoError.code();
                }
            }
            writeln!(stdout, "page: {}", page_label(result.page))
        }
        OperationResult::Info(result) => writeln!(stdout, "{}", result.display).and_then(|_| {
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
        }),
    };

    match write_result {
        Ok(()) => docnav_adapter_sdk::AdapterExitCode::Success.code(),
        Err(error) => {
            let _ = emit_diagnostic(stderr, &format!("failed to write text output: {error}"));
            docnav_adapter_sdk::AdapterExitCode::IoError.code()
        }
    }
}

fn page_label(page: Option<docnav_protocol::PositiveInteger>) -> String {
    page.map(|page| page.get().to_string())
        .unwrap_or_else(|| "null".to_owned())
}

#[derive(Clone, Debug, Serialize)]
struct ReadableError {
    code: StableErrorCode,
    error: String,
    details: ErrorDetails,
    guidance: Vec<String>,
}

fn handler_error<W: Write, E: Write>(
    error: docnav_adapter_sdk::AdapterError,
    output: OutputMode,
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    let exit_code = error.exit_code();
    let stable = error.error();
    let write_exit = match output {
        OutputMode::Text => write_text_error(stable, stdout, stderr),
        OutputMode::ReadableJson => write_readable_error(stable, stdout, stderr),
        OutputMode::ProtocolJson => unreachable!("protocol-json is handled before dispatch"),
    };
    if write_exit == docnav_adapter_sdk::AdapterExitCode::Success.code() {
        exit_code.code()
    } else {
        write_exit
    }
}

fn write_readable_error<W: Write, E: Write>(
    error: &docnav_protocol::StableError,
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    let readable = ReadableError {
        code: error.code,
        error: error.message.clone(),
        details: error.details.clone(),
        guidance: error.guidance.clone().unwrap_or_default(),
    };
    write_json_result(&readable, stdout, stderr)
}

fn write_text_error<W: Write, E: Write>(
    error: &docnav_protocol::StableError,
    stdout: &mut W,
    stderr: &mut E,
) -> i32 {
    let write_result = writeln!(stdout, "error: {}", error_code_label(error.code))
        .and_then(|_| writeln!(stdout, "message: {}", error.message))
        .and_then(|_| {
            if error.details.is_empty() {
                Ok(())
            } else {
                writeln!(stdout, "details: {}", details_label(&error.details))
            }
        })
        .and_then(|_| {
            let Some(guidance) = &error.guidance else {
                return Ok(());
            };
            for item in guidance {
                writeln!(stdout, "guidance: {item}")?;
            }
            Ok(())
        });

    match write_result {
        Ok(()) => docnav_adapter_sdk::AdapterExitCode::Success.code(),
        Err(error) => {
            let _ = emit_diagnostic(stderr, &format!("failed to write text error: {error}"));
            docnav_adapter_sdk::AdapterExitCode::IoError.code()
        }
    }
}

fn error_code_label(code: StableErrorCode) -> String {
    serde_json::to_value(code)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| format!("{code:?}"))
}

fn details_label(details: &ErrorDetails) -> String {
    details
        .iter()
        .map(|(key, value)| {
            value
                .as_str()
                .map(|value| format!("{key}={value}"))
                .unwrap_or_else(|| format!("{key}={value}"))
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn parse_protocol_only_output(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Ok(());
    }
    if args.len() == 2 && args[0] == "--output" && args[1] == "protocol-json" {
        return Ok(());
    }
    Err("only --output protocol-json is supported for this command".to_owned())
}

fn parse_probe(args: &[String]) -> Result<String, String> {
    let Some(path) = args.first() else {
        return Err("probe requires <path>".to_owned());
    };
    parse_protocol_only_output(&args[1..])?;
    Ok(path.clone())
}

fn parse_operation_options(
    operation: Operation,
    args: &[String],
) -> Result<OperationOptions, String> {
    let Some(path) = args.first() else {
        return Err(format!("{operation} requires <path>"));
    };
    let mut options = OperationOptions {
        path: path.clone(),
        page: positive_result(1).expect("static positive integer"),
        limit_chars: positive_result(DEFAULT_LIMIT_CHARS).expect("static positive integer"),
        max_heading_level: DEFAULT_MAX_HEADING_LEVEL,
        output: OutputMode::Text,
        ref_id: None,
        query: None,
    };

    let mut index = 1;
    while index < args.len() {
        let flag = args[index].as_str();
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag {
            "--page" if operation != Operation::Info => {
                options.page = parse_positive(value, "--page")?;
            }
            "--limit-chars" if operation != Operation::Info => {
                options.limit_chars = parse_positive(value, "--limit-chars")?;
            }
            "--max-heading-level" if matches!(operation, Operation::Outline | Operation::Find) => {
                options.max_heading_level = parse_heading_level(value)?;
            }
            "--ref" if operation == Operation::Read => {
                if value.is_empty() {
                    return Err("--ref must not be empty".to_owned());
                }
                options.ref_id = Some(value.clone());
            }
            "--query" if operation == Operation::Find => {
                if value.is_empty() {
                    return Err("--query must not be empty".to_owned());
                }
                options.query = Some(value.clone());
            }
            "--output" => {
                options.output = parse_output(value)?;
            }
            _ => return Err(format!("unknown or unsupported flag {flag}")),
        }
        index += 2;
    }

    Ok(options)
}

fn parse_output(value: &str) -> Result<OutputMode, String> {
    match value {
        "text" => Ok(OutputMode::Text),
        "readable-json" => Ok(OutputMode::ReadableJson),
        "protocol-json" => Ok(OutputMode::ProtocolJson),
        _ => Err(format!("invalid --output {value:?}")),
    }
}

fn parse_positive(value: &str, flag: &str) -> Result<docnav_protocol::PositiveInteger, String> {
    let raw = value
        .parse::<u32>()
        .map_err(|_| format!("{flag} must be a positive integer"))?;
    positive_result(raw).map_err(|_| format!("{flag} must be a positive integer"))
}

fn parse_heading_level(value: &str) -> Result<u8, String> {
    let level = value
        .parse::<u8>()
        .map_err(|_| "--max-heading-level must be an integer from 1 to 6".to_owned())?;
    if (1..=6).contains(&level) {
        Ok(level)
    } else {
        Err("--max-heading-level must be an integer from 1 to 6".to_owned())
    }
}

fn usage<E: Write>(stderr: &mut E) -> i32 {
    input_error(
        stderr,
        "usage: docnav-markdown <outline|read|find|info|manifest|probe|invoke> ...",
    )
}

fn input_error<E: Write>(stderr: &mut E, message: &str) -> i32 {
    let _ = emit_diagnostic(stderr, message);
    docnav_adapter_sdk::AdapterExitCode::ProtocolError.code()
}

#[allow(dead_code)]
fn _assert_protocol_response_is_used(response: ProtocolResponse) -> ProtocolResponse {
    response
}
