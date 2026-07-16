use clap::Command;
use docnav_cli_args::KnownValueFlag;
use docnav_navigation::NavigationAdapterRegistry;
use docnav_protocol::Operation;

use super::super::argument_helpers::split_equals;
use super::super::document_clap_command;

const DOCUMENT_OPERATIONS: [Operation; 4] = [
    Operation::Outline,
    Operation::Read,
    Operation::Find,
    Operation::Info,
];

pub(super) struct DocumentValueFlags {
    flags: Vec<DocumentFlag>,
}

struct DocumentFlag {
    flag: String,
    used: bool,
    takes_value: bool,
}

impl DocumentValueFlags {
    pub(super) fn new<R>(operation: Operation, registry: &R, command: &Command) -> Self
    where
        R: NavigationAdapterRegistry + ?Sized,
    {
        let mut flags = Vec::new();
        add_flags(&mut flags, command, true);
        for candidate_operation in DOCUMENT_OPERATIONS {
            if candidate_operation == operation {
                continue;
            }
            if let Ok((command, _)) = document_clap_command(candidate_operation, registry) {
                add_flags(&mut flags, &command, false);
            }
        }
        Self { flags }
    }

    pub(super) fn known_value_flags(&self) -> Vec<KnownValueFlag<'_>> {
        self.flags
            .iter()
            .filter(|flag| flag.takes_value)
            .map(|flag| KnownValueFlag {
                flag: &flag.flag,
                used: flag.used,
            })
            .collect()
    }

    pub(super) fn known_switch_flags(&self) -> Vec<&str> {
        self.flags
            .iter()
            .filter(|flag| !flag.takes_value)
            .map(|flag| flag.flag.as_str())
            .collect()
    }

    pub(super) fn contains(&self, token: &str) -> bool {
        self.flag(token).is_some()
    }

    pub(super) fn takes_value(&self, token: &str) -> Option<bool> {
        self.flag(token).map(|flag| flag.takes_value)
    }

    pub(super) fn is_unused(&self, token: &str) -> bool {
        self.flag(token).is_some_and(|flag| !flag.used)
    }

    fn flag(&self, token: &str) -> Option<&DocumentFlag> {
        let (flag, _) = split_equals(token);
        self.flags.iter().find(|known| known.flag == flag)
    }
}

fn add_flags(flags: &mut Vec<DocumentFlag>, command: &Command, used: bool) {
    for argument in command.get_arguments() {
        let flag_tokens = [
            argument.get_long().map(|long| format!("--{long}")),
            argument.get_short().map(|short| format!("-{short}")),
        ];
        for flag in flag_tokens.into_iter().flatten() {
            if let Some(existing) = flags.iter_mut().find(|known| known.flag == flag) {
                existing.used |= used;
            } else {
                flags.push(DocumentFlag {
                    flag,
                    used,
                    takes_value: argument.get_action().takes_values(),
                });
            }
        }
    }
}
