use docnav_cli_args::{
    scan_arg_boundaries, ArgBoundaryScan, KnownValueFlag as BoundaryKnownValueFlag, MissingValue,
    RejectedArg,
};
use docnav_protocol::Operation;

use super::super::native_options::NativeOptionSpec;
use super::spec::{command_labels, flags};

pub(super) struct BoundaryArgs {
    pub(super) clap_args: Vec<String>,
}

#[derive(Clone, Copy)]
pub(super) enum ArgBoundaryContext<'a> {
    ProtocolOnly { command: &'a str },
    Probe,
    Operation(Operation),
}

impl ArgBoundaryContext<'_> {
    fn accepts_path(self) -> bool {
        !matches!(self, Self::ProtocolOnly { .. })
    }

    fn command_label(&self) -> &str {
        match self {
            Self::ProtocolOnly { command } => command,
            Self::Probe => command_labels::PROBE,
            Self::Operation(operation) => operation.as_str(),
        }
    }

    fn uses_flag(self, flag: KnownValueFlag<'_>) -> bool {
        match self {
            Self::ProtocolOnly { .. } | Self::Probe => matches!(flag, KnownValueFlag::Output),
            Self::Operation(operation) => operation_uses_flag(operation, flag),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum KnownValueFlag<'a> {
    Page,
    Limit,
    Pagination,
    ProjectConfigPath,
    Ref,
    Query,
    Output,
    UserConfigPath,
    Native(&'a NativeOptionSpec),
}

pub(super) fn known_value_flag<'a>(
    token: &str,
    native_options: &'a [NativeOptionSpec],
) -> Option<KnownValueFlag<'a>> {
    let (flag, _value) = split_equals(token);
    match flag {
        flags::PAGE => Some(KnownValueFlag::Page),
        flags::LIMIT => Some(KnownValueFlag::Limit),
        flags::PAGINATION => Some(KnownValueFlag::Pagination),
        flags::PROJECT_CONFIG_PATH => Some(KnownValueFlag::ProjectConfigPath),
        flags::REF => Some(KnownValueFlag::Ref),
        flags::QUERY => Some(KnownValueFlag::Query),
        flags::OUTPUT => Some(KnownValueFlag::Output),
        flags::USER_CONFIG_PATH => Some(KnownValueFlag::UserConfigPath),
        _ => native_options
            .iter()
            .find(|spec| spec.flag == token)
            .map(KnownValueFlag::Native),
    }
}

pub(super) fn operation_uses_flag(operation: Operation, flag: KnownValueFlag<'_>) -> bool {
    match flag {
        KnownValueFlag::Page | KnownValueFlag::Limit | KnownValueFlag::Pagination => {
            operation != Operation::Info
        }
        KnownValueFlag::ProjectConfigPath | KnownValueFlag::UserConfigPath => true,
        KnownValueFlag::Ref => operation == Operation::Read,
        KnownValueFlag::Query => operation == Operation::Find,
        KnownValueFlag::Output => true,
        KnownValueFlag::Native(spec) => spec.supports(operation),
    }
}

pub(super) fn split_equals(token: &str) -> (&str, Option<&str>) {
    token
        .split_once('=')
        .map_or((token, None), |(flag, value)| (flag, Some(value)))
}

pub(super) fn is_flag(value: &str) -> bool {
    value.starts_with("--")
}

pub(super) fn collect_protocol_only_args(
    command: &str,
    args: &[String],
    native_options: &[NativeOptionSpec],
) -> Result<BoundaryArgs, String> {
    let context = ArgBoundaryContext::ProtocolOnly { command };
    collect_boundary_args(context, args, native_options)
}

pub(super) fn collect_probe_args(
    args: &[String],
    native_options: &[NativeOptionSpec],
) -> Result<BoundaryArgs, String> {
    collect_boundary_args(ArgBoundaryContext::Probe, args, native_options)
}

pub(super) fn collect_operation_args(
    operation: Operation,
    args: &[String],
    native_options: &[NativeOptionSpec],
) -> Result<BoundaryArgs, String> {
    collect_boundary_args(
        ArgBoundaryContext::Operation(operation),
        args,
        native_options,
    )
}

fn collect_boundary_args(
    context: ArgBoundaryContext<'_>,
    args: &[String],
    native_options: &[NativeOptionSpec],
) -> Result<BoundaryArgs, String> {
    let known_value_flags = boundary_known_value_flags(context, native_options);
    let positional_limit = usize::from(context.accepts_path());
    let scan = scan_arg_boundaries(
        args,
        &ArgBoundaryScan::new(
            context.command_label(),
            positional_limit,
            &known_value_flags,
        ),
    )
    .map_err(boundary_missing_value_error)?;
    if let Some(rejected) = scan.rejected.into_iter().next() {
        return Err(strict_rejected_arg_error(rejected));
    }
    Ok(BoundaryArgs {
        clap_args: scan.retained_args,
    })
}

fn boundary_known_value_flags<'a>(
    context: ArgBoundaryContext<'a>,
    native_options: &'a [NativeOptionSpec],
) -> Vec<BoundaryKnownValueFlag<'a>> {
    let mut flags = vec![
        BoundaryKnownValueFlag {
            flag: flags::PAGE,
            used: context.uses_flag(KnownValueFlag::Page),
        },
        BoundaryKnownValueFlag {
            flag: flags::LIMIT,
            used: context.uses_flag(KnownValueFlag::Limit),
        },
        BoundaryKnownValueFlag {
            flag: flags::PAGINATION,
            used: context.uses_flag(KnownValueFlag::Pagination),
        },
        BoundaryKnownValueFlag {
            flag: flags::PROJECT_CONFIG_PATH,
            used: context.uses_flag(KnownValueFlag::ProjectConfigPath),
        },
        BoundaryKnownValueFlag {
            flag: flags::REF,
            used: context.uses_flag(KnownValueFlag::Ref),
        },
        BoundaryKnownValueFlag {
            flag: flags::QUERY,
            used: context.uses_flag(KnownValueFlag::Query),
        },
        BoundaryKnownValueFlag {
            flag: flags::OUTPUT,
            used: context.uses_flag(KnownValueFlag::Output),
        },
        BoundaryKnownValueFlag {
            flag: flags::USER_CONFIG_PATH,
            used: context.uses_flag(KnownValueFlag::UserConfigPath),
        },
    ];
    flags.extend(native_options.iter().map(|spec| BoundaryKnownValueFlag {
        flag: spec.flag,
        used: context.uses_flag(KnownValueFlag::Native(spec)),
    }));
    flags
}

fn strict_rejected_arg_error(rejected: RejectedArg) -> String {
    match rejected {
        RejectedArg::UnknownFlag { token } => format!("unknown argument {token}"),
        RejectedArg::ExtraPositional { token } => format!("extra positional argument {token}"),
        RejectedArg::UnusedValueFlag {
            flag,
            value,
            command,
        } => {
            let value = value.map_or(String::new(), |value| format!(" {value}"));
            format!("{flag}{value} is not used by {command} command")
        }
    }
}

fn boundary_missing_value_error(error: MissingValue) -> String {
    format!("{} requires a value", error.flag())
}
