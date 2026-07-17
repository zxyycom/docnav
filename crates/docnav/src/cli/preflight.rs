use docnav_protocol::Operation;

use super::command_model::{OutputMode, DOCUMENT_OUTPUT_FIELD_ID};
use super::flags;
use super::parser::{command_names, document_clap_command};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CliOutputContext {
    pub output_mode: OutputMode,
    pub operation: Option<Operation>,
}

pub fn output_context(args: &[String]) -> CliOutputContext {
    let operation = args.first().and_then(|command| operation(command));
    let output_argument = match operation {
        Some(operation) => projected_output_argument(operation),
        None => Some(core_output_argument()),
    };
    CliOutputContext {
        output_mode: output_argument
            .and_then(|argument| output_mode(args, &argument))
            .unwrap_or(OutputMode::ReadableView),
        operation,
    }
}

struct OutputArgument {
    flag: String,
    takes_value: bool,
}

fn projected_output_argument(operation: Operation) -> Option<OutputArgument> {
    let spec = document_clap_command(operation).ok()?;
    let argument = spec
        .command
        .get_arguments()
        .find(|argument| argument.get_id().as_str() == DOCUMENT_OUTPUT_FIELD_ID)?;
    let flag = argument
        .get_long()
        .map(|long| format!("--{long}"))
        .or_else(|| argument.get_short().map(|short| format!("-{short}")))?;
    Some(OutputArgument {
        flag,
        takes_value: argument.get_action().takes_values(),
    })
}

fn core_output_argument() -> OutputArgument {
    OutputArgument {
        flag: flags::OUTPUT.to_owned(),
        takes_value: true,
    }
}

fn output_mode(args: &[String], argument: &OutputArgument) -> Option<OutputMode> {
    if !argument.takes_value {
        return None;
    }
    let mut output = None;
    let mut index = 0;
    while index < args.len() {
        let (flag, inline_value) = args[index]
            .split_once('=')
            .map_or((args[index].as_str(), None), |(flag, value)| {
                (flag, Some(value))
            });
        if flag != argument.flag {
            index += 1;
            continue;
        }
        if let Some(value) = inline_value {
            if let Ok(mode) = value.parse::<OutputMode>() {
                output = Some(mode);
            }
            index += 1;
        } else {
            if let Some(value) = args.get(index + 1) {
                if let Ok(mode) = value.parse::<OutputMode>() {
                    output = Some(mode);
                }
                index += 2;
            } else {
                index += 1;
            }
        }
    }
    output
}

fn operation(command: &str) -> Option<Operation> {
    match command {
        command_names::OUTLINE => Some(Operation::Outline),
        command_names::READ => Some(Operation::Read),
        command_names::FIND => Some(Operation::Find),
        command_names::INFO => Some(Operation::Info),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
