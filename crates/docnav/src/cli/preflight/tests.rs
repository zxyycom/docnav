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
