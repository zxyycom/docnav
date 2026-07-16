use std::error::Error;

use clap::Command;
use cli_config_resolution::{
    extract_env, resolve, FieldBound, FieldDef, FieldDefSet, FieldIdentity, FieldLength,
    FieldValidation, MergeStrategy, ProcessStrategy, ProcessingId, ResolutionResult, SourceId,
    SourceKind,
};
use cli_config_resolution_clap::{augment_command, extract_cli};
use cli_config_resolution_serde::extract_config;
use docnav_typed_fields::FieldDefs;
use serde_json::{json, Map, Value};

#[derive(Debug, PartialEq, FieldDefs)]
struct Parameters {
    #[field(
        FieldDef::builder("limit")
            .process("cli", ProcessStrategy::cli_flag("--limit"))
            .process("env", ProcessStrategy::env_var("APP_LIMIT"))
            .process("config", ProcessStrategy::config_path(["settings", "limit"]))
            .validation(FieldValidation::int())
    )]
    limit: i64,

    #[field(
        FieldDef::builder("replace_list")
            .process("cli", ProcessStrategy::cli_flag("--replace-list"))
            .process(
                "config",
                ProcessStrategy::config_path(["settings", "replace_list"]),
            )
            .validation(FieldValidation::array())
    )]
    replace_list: Vec<Value>,

    #[field(
        FieldDef::builder("replace_map")
            .process("cli", ProcessStrategy::cli_flag("--replace-map"))
            .process(
                "config",
                ProcessStrategy::config_path(["settings", "replace_map"]),
            )
            .validation(FieldValidation::object())
    )]
    replace_map: Map<String, Value>,

    #[field(
        FieldDef::builder("append_items")
            .process("cli", ProcessStrategy::cli_flag("--append"))
            .process("env", ProcessStrategy::env_var("APP_APPEND"))
            .process(
                "config",
                ProcessStrategy::config_path(["settings", "append_items"]),
            )
            .validation(
                FieldValidation::array().length(FieldLength::max(FieldBound::closed(3))),
            )
            .merge(MergeStrategy::Append)
    )]
    append_items: Vec<Value>,

    #[field(
        FieldDef::builder("format")
            .process(
                "config",
                ProcessStrategy::config_path(["settings", "format"]),
            )
            .validation(FieldValidation::string())
            .default_static("readable")
    )]
    format: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let parameters = run()?;
    println!(
        "resolved limit={} replace_list={} replace_map={} append_items={} format={}",
        parameters.limit,
        serde_json::to_string(&parameters.replace_list)?,
        serde_json::to_string(&parameters.replace_map)?,
        serde_json::to_string(&parameters.append_items)?,
        parameters.format,
    );
    Ok(())
}

fn run() -> Result<Parameters, Box<dyn Error>> {
    let definitions = Parameters::field_defs()?;
    let fields = definitions.as_ref();

    for name in ["limit", "replace_list", "replace_map"] {
        let identity = FieldIdentity::new(name)?;
        assert_eq!(
            fields.field(&identity).map(|field| field.merge_strategy()),
            Some(MergeStrategy::Replace),
            "scalar, list, and map default to Replace",
        );
    }

    let result = resolve_flow(fields, &["cli-a"])?;
    assert!(result.diagnostics().is_empty());

    let limit = FieldIdentity::new("limit")?;
    let limit_trace = result.trace(&limit).expect("declared limit trace");
    assert_eq!(
        limit_trace
            .selected
            .as_ref()
            .map(|selected| selected.source_id.as_str()),
        Some("cli"),
    );

    let append_items = FieldIdentity::new("append_items")?;
    let append_trace = result.trace(&append_items).expect("declared append trace");
    assert_eq!(
        append_trace
            .contributors
            .iter()
            .map(|contributor| contributor.source_id.as_str())
            .collect::<Vec<_>>(),
        ["config", "env", "cli"],
    );
    assert_eq!(
        append_trace
            .selected
            .as_ref()
            .map(|selected| selected.source_id.as_str()),
        Some("cli"),
    );

    let format = FieldIdentity::new("format")?;
    let format_trace = result.trace(&format).expect("declared format trace");
    assert_eq!(
        format_trace
            .default_fallback
            .as_ref()
            .map(|fallback| &fallback.source_kind),
        Some(&SourceKind::Default),
    );
    assert_eq!(
        format_trace
            .selected
            .as_ref()
            .map(|selected| selected.source_id.as_str()),
        Some("static-default"),
    );

    let values = result.materialize()?;
    let parameters = definitions.materialize(&values)?;
    assert_eq!(parameters.limit, 12);
    assert_eq!(parameters.replace_list, vec![json!("cli-list")]);
    assert_eq!(
        parameters.replace_map,
        json!({"cli": "only"})
            .as_object()
            .expect("object literal")
            .clone(),
    );
    assert_eq!(
        parameters.append_items,
        vec![json!("config-a"), json!("env-a"), json!("cli-a")],
    );
    assert_eq!(parameters.format, "readable");
    Ok(parameters)
}

fn resolve_flow(
    fields: &FieldDefSet,
    cli_append_items: &[&str],
) -> Result<ResolutionResult, Box<dyn Error>> {
    let cli_id = ProcessingId::new("cli").expect("valid processing id");
    let env_id = ProcessingId::new("env").expect("valid processing id");
    let config_id = ProcessingId::new("config").expect("valid processing id");
    let command = augment_command(Command::new("resolution-flow"), fields, &cli_id)?;
    let mut arguments = vec![
        "resolution-flow",
        "--limit",
        "12",
        "--replace-list",
        "cli-list",
        "--replace-map",
        "cli=only",
    ];
    for item in cli_append_items {
        arguments.extend(["--append", *item]);
    }
    let matches = command.try_get_matches_from(arguments)?;

    let config = extract_config(
        &json!({
            "settings": {
                "limit": 5,
                "replace_list": ["config-list"],
                "replace_map": {"config": "only"},
                "append_items": ["config-a"]
            }
        }),
        fields,
        &config_id,
        SourceId::new("config")?,
        10,
    )?;
    let env = extract_env(
        fields,
        &env_id,
        SourceId::new("env")?,
        20,
        [("APP_LIMIT", "9"), ("APP_APPEND", r#"["env-a"]"#)],
    )?;
    let cli = extract_cli(&matches, fields, &cli_id, SourceId::new("cli")?, 30)?;
    let sources = [config, env, cli];
    assert_eq!(
        sources
            .iter()
            .map(cli_config_resolution::Source::priority)
            .collect::<Vec<_>>(),
        vec![10, 20, 30],
        "priority increases from config to env to CLI",
    );
    resolve(fields, &sources).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use cli_config_resolution::{DiagnosticReason, FieldIdentity};

    #[test]
    fn merged_value_is_rejected_by_canonical_final_validation() {
        let definitions = <super::Parameters as docnav_typed_fields::FieldDefs>::field_defs()
            .expect("field definitions");
        let result =
            super::resolve_flow(definitions.as_ref(), &["cli-a", "cli-b"]).expect("resolution");
        let identity = FieldIdentity::new("append_items").expect("identity");
        let diagnostic = result
            .diagnostics()
            .iter()
            .find(|diagnostic| diagnostic.field == identity)
            .expect("append final-validation diagnostic");

        assert!(matches!(
            &diagnostic.reason,
            DiagnosticReason::FinalValidation(_)
        ));
        assert!(result.materialize().is_err());
    }
}
