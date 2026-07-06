use docnav_adapter_contracts::AdapterOptionSpec;
use docnav_protocol::Operation;

use crate::registry;

const DOCUMENT_OPERATIONS: [Operation; 4] = [
    Operation::Outline,
    Operation::Read,
    Operation::Find,
    Operation::Info,
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct NativeOptionCatalog {
    entries: Vec<OperationNativeOptions>,
}

impl NativeOptionCatalog {
    pub(super) fn from_static_registry() -> Self {
        let entries = DOCUMENT_OPERATIONS
            .into_iter()
            .map(OperationNativeOptions::from_static_registry)
            .collect();
        Self { entries }
    }

    pub(super) fn for_operation(&self, operation: Operation) -> &[NativeOptionCliMetadata] {
        self.entry(operation)
            .map(|entry| entry.options.as_slice())
            .unwrap_or(&[])
    }

    pub(super) fn arg_ids_for_operation(&self, operation: Operation) -> &[&'static str] {
        self.entry(operation)
            .map(|entry| entry.arg_ids.as_slice())
            .unwrap_or(&[])
    }

    pub(super) fn all_document_options(&self) -> Vec<&NativeOptionCliMetadata> {
        let mut options = Vec::new();
        for entry in &self.entries {
            for option in &entry.options {
                if options
                    .iter()
                    .any(|existing: &&NativeOptionCliMetadata| existing.flag() == option.flag())
                {
                    continue;
                }
                options.push(option);
            }
        }
        options
    }

    fn entry(&self, operation: Operation) -> Option<&OperationNativeOptions> {
        self.entries
            .iter()
            .find(|entry| entry.operation == operation)
    }
}

impl Default for NativeOptionCatalog {
    fn default() -> Self {
        Self::from_static_registry()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OperationNativeOptions {
    operation: Operation,
    options: Vec<NativeOptionCliMetadata>,
    arg_ids: Vec<&'static str>,
}

impl OperationNativeOptions {
    fn from_static_registry(operation: Operation) -> Self {
        let mut options = Vec::new();
        let mut arg_ids = Vec::new();
        for option in registry::native_options_for(operation) {
            if let Some(arg_id) = option.cli_arg_id() {
                push_unique_arg_id(&mut arg_ids, arg_id);
            }
            if let Some(metadata) = cli_metadata(option) {
                push_unique_by_flag(&mut options, metadata);
            }
        }
        Self {
            operation,
            options,
            arg_ids,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct NativeOptionCliMetadata {
    flag: &'static str,
    arg_id: &'static str,
    operations: Vec<Operation>,
}

impl NativeOptionCliMetadata {
    pub(super) fn flag(&self) -> &'static str {
        self.flag
    }

    pub(super) fn arg_id(&self) -> &'static str {
        self.arg_id
    }

    pub(super) fn applies_to(&self, operation: Operation) -> bool {
        self.operations.contains(&operation)
    }
}

fn cli_metadata(option: AdapterOptionSpec) -> Option<NativeOptionCliMetadata> {
    let flag = option.cli_flag()?;
    let arg_id = option.cli_arg_id()?;
    Some(NativeOptionCliMetadata {
        flag,
        arg_id,
        operations: option.operations,
    })
}

fn push_unique_by_flag(
    options: &mut Vec<NativeOptionCliMetadata>,
    metadata: NativeOptionCliMetadata,
) {
    if options
        .iter()
        .any(|existing| existing.flag() == metadata.flag())
    {
        return;
    }
    options.push(metadata);
}

fn push_unique_arg_id(arg_ids: &mut Vec<&'static str>, arg_id: &'static str) {
    if arg_ids.contains(&arg_id) {
        return;
    }
    arg_ids.push(arg_id);
}
