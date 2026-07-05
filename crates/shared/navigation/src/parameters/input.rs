use docnav_adapter_contracts::AdapterOptionSpec;
use serde_json::{json, Map, Value};

use crate::NavigationCommand;

pub(super) fn direct_input(
    command: &NavigationCommand,
    selected_native_options: &[AdapterOptionSpec],
) -> Value {
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
    let options = native_option_input(command, selected_native_options);
    for (key, value) in options {
        input.insert(key, value);
    }
    Value::Object(input)
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
) -> Map<String, Value> {
    let mut input = Map::new();
    for option in &command.native_options {
        let Some(spec) = selected_native_options
            .iter()
            .find(|spec| spec.cli_flag() == Some(option.flag.as_str()))
        else {
            continue;
        };
        let Some(path) = spec.cli_input_path() else {
            continue;
        };
        insert_at_path(&mut input, &path, native_option_cli_value(&option.value));
    }
    input
}

fn insert_at_path(root: &mut Map<String, Value>, path: &[String], value: Value) {
    let Some((first, rest)) = path.split_first() else {
        return;
    };
    if rest.is_empty() {
        root.insert(first.clone(), value);
        return;
    }
    let child = root
        .entry(first.clone())
        .or_insert_with(|| Value::Object(Map::new()));
    if !child.is_object() {
        *child = Value::Object(Map::new());
    }
    if let Some(child) = child.as_object_mut() {
        insert_at_path(child, rest, value);
    }
}
