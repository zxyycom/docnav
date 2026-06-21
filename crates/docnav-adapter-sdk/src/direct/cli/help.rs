use super::super::args::direct_cli_command;
use super::super::native_options::NativeOptionSpec;

pub(super) fn help_text(
    args: &[String],
    program_name: &'static str,
    native_options: &[NativeOptionSpec],
    default_limit_chars: u32,
) -> Option<String> {
    if !args.iter().any(|arg| arg == "--help" || arg == "-h") {
        return None;
    }
    let mut root = direct_cli_command(program_name, native_options, default_limit_chars);
    let Some(first) = args.first().map(String::as_str) else {
        return Some(root.render_long_help().to_string());
    };
    if first == "--help" || first == "-h" {
        return Some(root.render_long_help().to_string());
    }
    root.find_subcommand_mut(first)
        .map(|command| command.render_long_help().to_string())
        .or_else(|| Some(root.render_long_help().to_string()))
}

pub(super) fn is_known_command(
    command: &str,
    program_name: &'static str,
    native_options: &[NativeOptionSpec],
    default_limit_chars: u32,
) -> bool {
    direct_cli_command(program_name, native_options, default_limit_chars)
        .find_subcommand(command)
        .is_some()
}
