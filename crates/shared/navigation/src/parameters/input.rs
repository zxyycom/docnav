use std::collections::BTreeMap;

use docnav_adapter_contracts::AdapterOptionSpec;
use serde_json::{json, Map, Value};

use crate::NavigationCommand;

pub(super) struct DirectInput {
    common: Value,
    native: BTreeMap<String, Value>,
}

impl DirectInput {
    pub(super) fn common(&self) -> &Value {
        &self.common
    }

    pub(super) fn native_value(&self, identity: &str) -> Option<&Value> {
        self.native.get(identity)
    }
}

pub(super) fn direct_input(
    command: &NavigationCommand,
    selected_native_options: &[AdapterOptionSpec],
) -> DirectInput {
    let mut input = Map::new();
    input.insert("path".to_owned(), json!(command.document_path));
    insert_optional_string(&mut input, "adapter", command.adapter.as_deref());
    insert_optional_string(&mut input, "ref", command.ref_id.as_deref());
    insert_optional_string(&mut input, "query", command.query.as_deref());
    if let Some(page) = command.page {
        input.insert("page".to_owned(), json!(page.get()));
    }
    if let Some(enabled) = command.pagination_enabled {
        input.insert("pagination".to_owned(), json!(enabled));
    }
    if let Some(limit) = command.limit {
        input.insert("limit".to_owned(), json!(limit.get()));
    }
    if let Some(output) = command.output {
        input.insert("output".to_owned(), json!(output.as_str()));
    }
    DirectInput {
        common: Value::Object(input),
        native: native_option_input(command, selected_native_options),
    }
}

pub(super) fn native_option_cli_value(value: &str) -> Value {
    serde_json::from_str(value).unwrap_or_else(|_| json!(value))
}

fn insert_optional_string(input: &mut Map<String, Value>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        input.insert(key.to_owned(), json!(value));
    }
}

fn native_option_input(
    command: &NavigationCommand,
    selected_native_options: &[AdapterOptionSpec],
) -> BTreeMap<String, Value> {
    let mut input = BTreeMap::new();
    for option in &command.native_options {
        let Some(spec) = selected_native_options
            .iter()
            .find(|spec| spec.cli_flag() == Some(option.flag.as_str()))
        else {
            continue;
        };
        input.insert(
            spec.identity.clone(),
            native_option_cli_value(&option.value),
        );
    }
    input
}
