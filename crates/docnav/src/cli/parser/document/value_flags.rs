use docnav_cli_args::KnownValueFlag;
use docnav_protocol::Operation;

use super::super::argument_helpers::{
    boundary_value_flags, known_value_flag, split_equals, ValueFlag,
};
use super::super::ParserContext;

pub(super) fn document_value_flags(
    operation: Operation,
    context: &ParserContext,
) -> Vec<KnownValueFlag<'static>> {
    let mut flags = boundary_value_flags(|flag| document_uses_flag(operation, flag));
    for option in context.native_options().all_document_options() {
        let flag = option.flag();
        let used = option.applies_to(operation);
        if flags.iter().any(|existing| existing.flag == flag) {
            continue;
        }
        flags.push(KnownValueFlag { flag, used });
    }
    flags
}

pub(super) fn is_known_document_value_flag(
    operation: Operation,
    token: &str,
    context: &ParserContext,
) -> bool {
    let (flag, _value) = split_equals(token);
    document_value_flags(operation, context)
        .iter()
        .any(|known| known.flag == flag)
}

pub(super) fn document_uses_flag(operation: Operation, flag: ValueFlag) -> bool {
    match flag {
        ValueFlag::Adapter
        | ValueFlag::InvocationLog
        | ValueFlag::InvocationLogContentRoot
        | ValueFlag::Output => true,
        ValueFlag::ProjectConfig | ValueFlag::UserConfig => true,
        ValueFlag::Page | ValueFlag::Pagination | ValueFlag::Limit => operation != Operation::Info,
        ValueFlag::Ref => operation == Operation::Read,
        ValueFlag::Query => operation == Operation::Find,
        ValueFlag::Path => false,
    }
}

#[derive(Clone, Copy)]
pub(super) struct ValueFlagOccurrence<'a> {
    pub(super) flag: ValueFlag,
    pub(super) flag_token: &'a str,
    pub(super) value: Option<&'a str>,
    pub(super) consumed: usize,
}

pub(super) fn value_flag_occurrence(
    args: &[String],
    index: usize,
) -> Option<ValueFlagOccurrence<'_>> {
    let token = &args[index];
    let flag = known_value_flag(token)?;
    let (flag_token, inline_value) = split_equals(token);
    Some(ValueFlagOccurrence {
        flag,
        flag_token,
        value: inline_value.or_else(|| args.get(index + 1).map(String::as_str)),
        consumed: if inline_value.is_some() { 1 } else { 2 },
    })
}
