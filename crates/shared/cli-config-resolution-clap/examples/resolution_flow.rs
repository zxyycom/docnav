use std::collections::BTreeMap;

use clap::Command;
use cli_config_resolution::{
    DefaultMetadata, DefaultSource, EnvVarSource, FieldContract, FieldProjectionDeclaration,
    FieldSet, MergeStrategy, RawSourceValue, Resolver, SourceCollection, SourceExtractor, SourceId,
    SourceKind, SourceSpec, Value, ValueKind,
};
use cli_config_resolution_clap::{augment_command, candidates_from_matches};
use cli_config_resolution_serde::JsonConfigSource;
use serde_json::json;

fn main() {
    let fields = FieldSet::builder()
        .add_field(limit_field())
        .add_field(include_field())
        .add_field(labels_field())
        .add_field(mode_field())
        .add_field(format_field())
        .build()
        .expect("field set");

    let command = augment_command(Command::new("resolution-flow"), &fields).expect("command");
    let matches = command
        .try_get_matches_from([
            "resolution-flow",
            "--limit",
            "12",
            "--include",
            "cli-a",
            "--label",
            "team=cli",
            "--mode",
            "fast",
        ])
        .expect("matches");

    let cli = source("cli", SourceKind::Cli, 40);
    let env = source("env", SourceKind::Env, 30);
    let config = source("config", SourceKind::Config, 20);
    let default = source("default", SourceKind::Default, 0);
    let sources = SourceCollection::new(vec![
        cli.clone(),
        env.clone(),
        config.clone(),
        default.clone(),
    ])
    .expect("sources");

    let mut candidates = candidates_from_matches(&matches, &cli, &fields);
    candidates.extend(
        EnvVarSource::new(BTreeMap::from([(
            "APP_LIMIT".to_owned(),
            RawSourceValue::Present(Value::Integer(9)),
        )]))
        .extract(&env, &fields),
    );
    candidates.extend(
        JsonConfigSource::new(json!({
            "read": {
                "limit": 5,
                "include": ["config-a"],
                "labels": {
                    "region": "us",
                    "team": "config"
                },
                "mode": "safe"
            }
        }))
        .extract(&config, &fields),
    );
    candidates.extend(DefaultSource::default().extract(&default, &fields));

    let result = Resolver::resolve(&fields, &sources, candidates);
    for line in result.explain().lines() {
        println!("{line}");
    }
}

fn source(value: &str, kind: SourceKind, priority: i32) -> SourceSpec {
    SourceSpec::new(SourceId::new(value).expect("source id"), kind, priority)
}

fn limit_field() -> FieldContract {
    FieldContract::builder("limit", ValueKind::Integer)
        .projection(FieldProjectionDeclaration::cli_flag("--limit"))
        .projection(FieldProjectionDeclaration::env_var("APP_LIMIT"))
        .projection(
            FieldProjectionDeclaration::config_path(["read", "limit"]).expect("config path"),
        )
        .default(DefaultMetadata::Static(Value::Integer(20)))
        .build()
        .expect("limit field")
}

fn include_field() -> FieldContract {
    FieldContract::builder("include", ValueKind::List)
        .projection(FieldProjectionDeclaration::cli_flag("--include"))
        .projection(
            FieldProjectionDeclaration::config_path(["read", "include"]).expect("config path"),
        )
        .merge_strategy(MergeStrategy::ListAppend)
        .build()
        .expect("include field")
}

fn labels_field() -> FieldContract {
    FieldContract::builder("labels", ValueKind::Map)
        .projection(FieldProjectionDeclaration::cli_flag("--label"))
        .projection(
            FieldProjectionDeclaration::config_path(["read", "labels"]).expect("config path"),
        )
        .merge_strategy(MergeStrategy::MapMerge)
        .build()
        .expect("labels field")
}

fn mode_field() -> FieldContract {
    FieldContract::builder("mode", ValueKind::String)
        .projection(FieldProjectionDeclaration::cli_flag("--mode"))
        .projection(FieldProjectionDeclaration::config_path(["read", "mode"]).expect("config path"))
        .merge_strategy(MergeStrategy::DenyConflict)
        .build()
        .expect("mode field")
}

fn format_field() -> FieldContract {
    FieldContract::builder("format", ValueKind::String)
        .default(DefaultMetadata::Static(Value::from("readable")))
        .build()
        .expect("format field")
}
