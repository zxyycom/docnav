// @case WB-CORE-PREFLIGHT-001
use super::*;

fn strings(args: &[&str]) -> Vec<String> {
    args.iter().map(|arg| (*arg).to_owned()).collect()
}

#[test]
fn detects_space_separated_protocol_json_output() {
    let context = output_context(&strings(&["read", "doc.md", "--output", "protocol-json"]));

    assert_eq!(context.output_mode, OutputMode::ProtocolJson);
    assert_eq!(context.operation, Some(Operation::Read));
}

#[test]
fn detects_equals_protocol_json_output() {
    let context = output_context(&strings(&["read", "doc.md", "--output=protocol-json"]));

    assert_eq!(context.output_mode, OutputMode::ProtocolJson);
    assert_eq!(context.operation, Some(Operation::Read));
}

#[test]
fn document_without_output_defaults_to_readable_view() {
    let context = output_context(&strings(&["read", "doc.md", "--ref", "R1"]));

    assert_eq!(context.output_mode, OutputMode::ReadableView);
    assert_eq!(context.operation, Some(Operation::Read));
}

#[test]
fn non_document_output_context_keeps_plain_command_semantics() {
    for args in [
        vec!["--help"],
        vec!["version"],
        vec!["config", "inspect"],
        vec!["adapter", "list"],
        vec!["init"],
        vec!["doctor"],
    ] {
        let context = output_context(&strings(&args));
        assert_eq!(context.output_mode, OutputMode::ReadableView);
        assert_eq!(context.operation, None);
    }
}

#[test]
fn non_document_protocol_json_hint_uses_core_output_flag() {
    let context = output_context(&strings(&[
        "config",
        "set",
        "defaults.output",
        "protocol-json",
        "--output",
        "protocol-json",
    ]));

    assert_eq!(context.output_mode, OutputMode::ProtocolJson);
    assert_eq!(context.operation, None);
}

#[test]
fn legacy_config_failure_uses_protocol_json_framing() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = crate::run(
        [
            "config",
            "set",
            "defaults.output",
            "protocol-json",
            "--output",
            "protocol-json",
        ],
        std::io::empty(),
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, crate::error::DocnavExitCode::InputError.code());
    assert!(stderr.is_empty());
    let output: serde_json::Value = serde_json::from_slice(&stdout).expect("protocol failure");
    assert_eq!(output["operation"], serde_json::Value::Null);
    assert_eq!(output["error"]["details"]["field"], "config");
}

#[test]
fn projected_output_locator_frames_document_structural_failure() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = crate::run(
        [
            "outline",
            "docs/navigation.md",
            "--future",
            "--output",
            "protocol-json",
        ],
        std::io::empty(),
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, crate::error::DocnavExitCode::InputError.code());
    assert!(stderr.is_empty());
    let output: serde_json::Value = serde_json::from_slice(&stdout).expect("protocol failure");
    assert_eq!(output["operation"], "outline");
    assert_eq!(output["error"]["details"]["reason"], "unknown_argument");
}
