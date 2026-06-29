use docnav_cli_args::{
    scan_loose_args, IgnoredArg, KnownValueFlag as LooseKnownValueFlag, LooseArgScan, MissingValue,
};
use docnav_protocol::Operation;

use super::super::native_options::NativeOptionSpec;
use super::super::warnings::DirectCliWarning;
use super::spec::{command_labels, flags};

pub(super) struct LooseArgs {
    pub(super) clap_args: Vec<String>,
    pub(super) warnings: Vec<DirectCliWarning>,
}

#[derive(Clone, Copy)]
pub(super) enum LooseArgContext<'a> {
    ProtocolOnly { command: &'a str },
    Probe,
    Operation(Operation),
}

impl LooseArgContext<'_> {
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
        KnownValueFlag::Page | KnownValueFlag::Limit => operation != Operation::Info,
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
) -> Result<LooseArgs, String> {
    let context = LooseArgContext::ProtocolOnly { command };
    collect_loose_args(context, args, native_options)
}

pub(super) fn collect_probe_args(
    args: &[String],
    native_options: &[NativeOptionSpec],
) -> Result<LooseArgs, String> {
    collect_loose_args(LooseArgContext::Probe, args, native_options)
}

pub(super) fn collect_operation_args(
    operation: Operation,
    args: &[String],
    native_options: &[NativeOptionSpec],
) -> Result<LooseArgs, String> {
    collect_loose_args(LooseArgContext::Operation(operation), args, native_options)
}

fn collect_loose_args(
    context: LooseArgContext<'_>,
    args: &[String],
    native_options: &[NativeOptionSpec],
) -> Result<LooseArgs, String> {
    let known_value_flags = loose_known_value_flags(context, native_options);
    let positional_limit = usize::from(context.accepts_path());
    let scan = scan_loose_args(
        args,
        &LooseArgScan::new(
            context.command_label(),
            positional_limit,
            &known_value_flags,
        ),
    )
    .map_err(loose_missing_value_error)?;
    Ok(LooseArgs {
        clap_args: scan.retained_args,
        warnings: scan
            .ignored
            .into_iter()
            .map(warning_from_ignored_arg)
            .collect(),
    })
}

fn loose_known_value_flags<'a>(
    context: LooseArgContext<'a>,
    native_options: &'a [NativeOptionSpec],
) -> Vec<LooseKnownValueFlag<'a>> {
    let mut flags = vec![
        LooseKnownValueFlag {
            flag: flags::PAGE,
            used: context.uses_flag(KnownValueFlag::Page),
        },
        LooseKnownValueFlag {
            flag: flags::LIMIT,
            used: context.uses_flag(KnownValueFlag::Limit),
        },
        LooseKnownValueFlag {
            flag: flags::PROJECT_CONFIG_PATH,
            used: context.uses_flag(KnownValueFlag::ProjectConfigPath),
        },
        LooseKnownValueFlag {
            flag: flags::REF,
            used: context.uses_flag(KnownValueFlag::Ref),
        },
        LooseKnownValueFlag {
            flag: flags::QUERY,
            used: context.uses_flag(KnownValueFlag::Query),
        },
        LooseKnownValueFlag {
            flag: flags::OUTPUT,
            used: context.uses_flag(KnownValueFlag::Output),
        },
        LooseKnownValueFlag {
            flag: flags::USER_CONFIG_PATH,
            used: context.uses_flag(KnownValueFlag::UserConfigPath),
        },
    ];
    flags.extend(native_options.iter().map(|spec| LooseKnownValueFlag {
        flag: spec.flag,
        used: context.uses_flag(KnownValueFlag::Native(spec)),
    }));
    flags
}

fn warning_from_ignored_arg(ignored: IgnoredArg) -> DirectCliWarning {
    match ignored {
        IgnoredArg::UnknownFlag { token } => DirectCliWarning::unknown_flag(&token),
        IgnoredArg::ExtraPositional { token } => DirectCliWarning::extra_positional(&token),
        IgnoredArg::UnusedValueFlag {
            flag,
            value,
            command,
        } => DirectCliWarning::unused_operation_flag(&flag, value.as_deref(), &command),
    }
}

fn loose_missing_value_error(error: MissingValue) -> String {
    format!("{} requires a value", error.flag())
}
