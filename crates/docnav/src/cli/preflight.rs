use docnav_protocol::Operation;

use super::flags;
use super::types::OutputMode;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CliOutputContext {
    pub output_mode: OutputMode,
    pub operation: Option<Operation>,
}

pub fn output_context(args: &[String]) -> CliOutputContext {
    CliOutputContext {
        output_mode: output_mode(args).unwrap_or(OutputMode::ReadableView),
        operation: args.first().and_then(|command| operation(command)),
    }
}

fn output_mode(args: &[String]) -> Option<OutputMode> {
    let mut output = None;
    let mut index = 0;
    while index < args.len() {
        if let Some(value) = args[index].strip_prefix("--output=") {
            if let Ok(mode) = value.parse::<OutputMode>() {
                output = Some(mode);
            }
            index += 1;
        } else if args[index] == flags::OUTPUT {
            if let Some(value) = args.get(index + 1) {
                if let Ok(mode) = value.parse::<OutputMode>() {
                    output = Some(mode);
                }
                index += 2;
            } else {
                index += 1;
            }
        } else {
            index += 1;
        }
    }
    output
}

fn operation(command: &str) -> Option<Operation> {
    match command {
        "outline" => Some(Operation::Outline),
        "read" => Some(Operation::Read),
        "find" => Some(Operation::Find),
        "info" => Some(Operation::Info),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
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
}
