use std::collections::BTreeMap;

use super::*;

fn source_id(value: &str) -> SourceId {
    SourceId::new(value).expect("source id")
}

fn source(value: &str, kind: SourceKind, priority: i32) -> SourceSpec {
    SourceSpec::new(source_id(value), kind, priority)
}

fn field(identity: &str, kind: ValueKind) -> FieldContractBuilder {
    FieldContract::builder(identity, kind)
}

fn built_field(identity: &str, kind: ValueKind) -> FieldContract {
    field(identity, kind).build().expect("field")
}

fn fields(builders: Vec<FieldContractBuilder>) -> FieldSet {
    builders
        .into_iter()
        .fold(FieldSet::builder(), |builder, field| {
            builder.add_declaration(field)
        })
        .build()
        .expect("field set")
}

fn sources(specs: Vec<SourceSpec>) -> SourceCollection {
    SourceCollection::new(specs).expect("sources")
}

fn map(entries: Vec<(&str, Value)>) -> Value {
    Value::Map(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect(),
    )
}

fn candidate(
    field: &FieldContract,
    source: &SourceSpec,
    locator: SourceLocator,
    state: CandidateState,
) -> SourceCandidate {
    SourceCandidate::new(field.identity().clone(), source, locator, state)
}

fn raw_present(value: impl Into<Value>) -> RawSourceValue {
    RawSourceValue::Present(value.into())
}

// @case WB-PARAM-FIELD-CONTRACT-001
#[test]
fn config_path_rejects_an_empty_path() {
    let error = ConfigPath::new(Vec::<String>::new()).expect_err("empty path rejected");

    assert_eq!(error, ConfigPathError::EmptyPath);
    assert_eq!(error.to_string(), "config path is empty");
}

#[test]
fn config_path_rejects_an_empty_segment() {
    let error = ConfigPath::new(["read", ""]).expect_err("empty segment rejected");

    assert_eq!(error, ConfigPathError::EmptySegment { index: 1 });
    assert_eq!(
        error.to_string(),
        "config path contains an empty segment at index 1"
    );
}

#[test]
fn field_set_builder_rejects_duplicate_identity_and_projection_locator() {
    let first = field("limit", ValueKind::Integer)
        .projection(FieldProjectionDeclaration::cli_flag("--limit"))
        .build()
        .expect("first field");
    let duplicate_identity = field("limit", ValueKind::Integer)
        .projection(FieldProjectionDeclaration::env_var("APP_LIMIT"))
        .build()
        .expect("duplicate field");
    let duplicate_identity_error = FieldSet::builder()
        .add_field(first.clone())
        .add_field(duplicate_identity)
        .build()
        .expect_err("duplicate identity rejected");
    assert!(matches!(
        duplicate_identity_error,
        FieldSetBuildError::DuplicateIdentity { .. }
    ));

    let second = field("other_limit", ValueKind::Integer)
        .projection(FieldProjectionDeclaration::cli_flag("--limit"))
        .build()
        .expect("second field");
    let duplicate_projection_error = FieldSet::builder()
        .add_field(first)
        .add_field(second)
        .build()
        .expect_err("duplicate projection rejected");
    assert!(matches!(
        duplicate_projection_error,
        FieldSetBuildError::DuplicateProjectionLocator { .. }
    ));
}

#[test]
fn field_set_builder_rejects_invalid_and_incompatible_declarations() {
    let invalid_identity = FieldSet::builder()
        .add_declaration(field("", ValueKind::String))
        .build()
        .expect_err("invalid field rejected");
    assert!(matches!(
        invalid_identity,
        FieldSetBuildError::InvalidFieldDeclaration(FieldBuildError::EmptyIdentity)
    ));

    let incompatible_projection = field("limit", ValueKind::Integer).projection(
        FieldProjectionDeclaration::new(SourceKind::Cli, SourceLocator::EnvVar("APP_LIMIT".into())),
    );
    let incompatible_error = FieldSet::builder()
        .add_declaration(incompatible_projection)
        .build()
        .expect_err("incompatible projection rejected");
    assert!(matches!(
        incompatible_error,
        FieldSetBuildError::InvalidFieldDeclaration(
            FieldBuildError::IncompatibleProjectionLocator { .. }
        )
    ));

    let invalid_path = FieldProjectionDeclaration::config_path(["read", ""]);
    assert!(matches!(
        invalid_path,
        Err(FieldBuildError::InvalidProjectionPath)
    ));
}

#[test]
fn field_contract_builder_enforces_value_kind_merge_strategy_compatibility() {
    let value_kinds = [
        ValueKind::String,
        ValueKind::Integer,
        ValueKind::Number,
        ValueKind::Boolean,
        ValueKind::List,
        ValueKind::Map,
        ValueKind::Any,
    ];
    let merge_strategies = [
        MergeStrategy::ScalarReplace,
        MergeStrategy::ListAppend,
        MergeStrategy::ListReplace,
        MergeStrategy::MapMerge,
        MergeStrategy::MapReplace,
        MergeStrategy::DenyConflict,
    ];

    for value_kind in value_kinds {
        for merge_strategy in merge_strategies {
            let compatible = matches!(
                (value_kind, merge_strategy),
                (
                    _,
                    MergeStrategy::ScalarReplace | MergeStrategy::DenyConflict
                ) | (
                    ValueKind::List,
                    MergeStrategy::ListAppend | MergeStrategy::ListReplace
                ) | (
                    ValueKind::Map,
                    MergeStrategy::MapMerge | MergeStrategy::MapReplace
                )
            );
            let result = field("value", value_kind)
                .merge_strategy(merge_strategy)
                .build();

            if compatible {
                result.expect("compatible kind and merge strategy accepted");
            } else {
                assert_eq!(
                    result.expect_err("incompatible kind and merge strategy rejected"),
                    FieldBuildError::IncompatibleMergeStrategy {
                        value_kind,
                        merge_strategy,
                    }
                );
            }
        }
    }
}

#[test]
fn field_contract_records_reusable_metadata_without_application_slots() {
    let constraints = FieldConstraints {
        required: true,
        nullable: false,
        min_number: Some(1.0),
        max_number: Some(100.0),
        ..FieldConstraints::default()
    };
    let contract = field("limit", ValueKind::Integer)
        .constraints(constraints.clone())
        .default(DefaultMetadata::Dynamic(
            DynamicDefaultMetadata::new("runtime-limit").expect("dynamic default"),
        ))
        .projection(FieldProjectionDeclaration::cli_flag("--limit"))
        .projection(FieldProjectionDeclaration::env_var("APP_LIMIT"))
        .projection(
            FieldProjectionDeclaration::config_path(["read", "limit"]).expect("config projection"),
        )
        .projection(FieldProjectionDeclaration::custom("profile", "limit"))
        .merge_strategy(MergeStrategy::ScalarReplace)
        .build()
        .expect("contract");

    assert_eq!(contract.identity().as_str(), "limit");
    assert_eq!(contract.value_kind(), ValueKind::Integer);
    assert_eq!(contract.constraints(), &constraints);
    assert!(matches!(contract.default(), DefaultMetadata::Dynamic(_)));
    assert_eq!(
        contract
            .projections()
            .iter()
            .map(|projection| projection.source_kind().clone())
            .collect::<Vec<_>>(),
        vec![
            SourceKind::Cli,
            SourceKind::Env,
            SourceKind::Config,
            SourceKind::Custom("profile".to_owned())
        ]
    );
}

// @case WB-PARAM-SOURCE-EXTRACTION-001
#[test]
fn source_extractors_preserve_states_and_locators() {
    let field_set = fields(vec![
        field("present", ValueKind::String)
            .projection(FieldProjectionDeclaration::cli_flag("--present"))
            .projection(FieldProjectionDeclaration::env_var("APP_PRESENT"))
            .projection(FieldProjectionDeclaration::config_path(["present"]).expect("present path"))
            .projection(FieldProjectionDeclaration::custom("profile", "present")),
        field("invalid", ValueKind::String)
            .projection(FieldProjectionDeclaration::cli_flag("--invalid"))
            .projection(FieldProjectionDeclaration::env_var("APP_INVALID"))
            .projection(FieldProjectionDeclaration::config_path(["invalid"]).expect("invalid path"))
            .projection(FieldProjectionDeclaration::custom("profile", "invalid"))
            .default(DefaultMetadata::Dynamic(
                DynamicDefaultMetadata::new("invalid-default").expect("dynamic default"),
            )),
        field("absent", ValueKind::String)
            .projection(FieldProjectionDeclaration::cli_flag("--absent"))
            .projection(FieldProjectionDeclaration::env_var("APP_ABSENT"))
            .projection(FieldProjectionDeclaration::config_path(["absent"]).expect("absent path"))
            .projection(FieldProjectionDeclaration::custom("profile", "absent"))
            .default(DefaultMetadata::Dynamic(
                DynamicDefaultMetadata::new("absent-default").expect("dynamic default"),
            )),
        field("missing", ValueKind::String)
            .projection(FieldProjectionDeclaration::cli_flag("--missing"))
            .projection(FieldProjectionDeclaration::env_var("APP_MISSING"))
            .projection(FieldProjectionDeclaration::config_path(["missing"]).expect("missing path"))
            .projection(FieldProjectionDeclaration::custom("profile", "missing")),
        field("static_default", ValueKind::String)
            .default(DefaultMetadata::Static(Value::from("fallback"))),
    ]);

    let cli_source = source("cli", SourceKind::Cli, 40);
    let env_source = source("env", SourceKind::Env, 30);
    let config_source = source("config", SourceKind::Config, 20);
    let default_source = source("default", SourceKind::Default, 0);
    let custom_source = source("profile", SourceKind::Custom("profile".to_owned()), 10);

    let cli_candidates = CliFlagSource::new(BTreeMap::from([
        ("--present".to_owned(), raw_present("cli")),
        (
            "--invalid".to_owned(),
            RawSourceValue::Invalid {
                received: Some(Value::from("bad")),
                reason: "parse failed".to_owned(),
            },
        ),
        ("--absent".to_owned(), RawSourceValue::ExplicitAbsent),
    ]))
    .extract(&cli_source, &field_set);
    assert_candidate(
        &cli_candidates,
        "present",
        SourceLocator::CliFlag("--present".to_owned()),
        CandidateStateName::Present,
    );
    assert_candidate(
        &cli_candidates,
        "invalid",
        SourceLocator::CliFlag("--invalid".to_owned()),
        CandidateStateName::Invalid,
    );
    assert_candidate(
        &cli_candidates,
        "absent",
        SourceLocator::CliFlag("--absent".to_owned()),
        CandidateStateName::ExplicitAbsent,
    );
    assert_candidate(
        &cli_candidates,
        "missing",
        SourceLocator::CliFlag("--missing".to_owned()),
        CandidateStateName::Missing,
    );

    let env_candidates = EnvVarSource::new(BTreeMap::from([
        ("APP_PRESENT".to_owned(), raw_present("env")),
        (
            "APP_INVALID".to_owned(),
            RawSourceValue::Invalid {
                received: Some(Value::from("bad")),
                reason: "parse failed".to_owned(),
            },
        ),
        ("APP_ABSENT".to_owned(), RawSourceValue::ExplicitAbsent),
    ]))
    .extract(&env_source, &field_set);
    assert_candidate(
        &env_candidates,
        "present",
        SourceLocator::EnvVar("APP_PRESENT".to_owned()),
        CandidateStateName::Present,
    );
    assert_candidate(
        &env_candidates,
        "invalid",
        SourceLocator::EnvVar("APP_INVALID".to_owned()),
        CandidateStateName::Invalid,
    );
    assert_candidate(
        &env_candidates,
        "absent",
        SourceLocator::EnvVar("APP_ABSENT".to_owned()),
        CandidateStateName::ExplicitAbsent,
    );
    assert_candidate(
        &env_candidates,
        "missing",
        SourceLocator::EnvVar("APP_MISSING".to_owned()),
        CandidateStateName::Missing,
    );

    let config_candidates =
        ConfigDocumentSource::new(map(vec![("present", Value::from("config"))]))
            .with_path_values(BTreeMap::from([
                (
                    "invalid".to_owned(),
                    RawSourceValue::Invalid {
                        received: Some(Value::from("bad")),
                        reason: "parse failed".to_owned(),
                    },
                ),
                ("absent".to_owned(), RawSourceValue::ExplicitAbsent),
            ]))
            .extract(&config_source, &field_set);
    assert_candidate(
        &config_candidates,
        "present",
        SourceLocator::ConfigPath(ConfigPath::new(["present"]).expect("path")),
        CandidateStateName::Present,
    );
    assert_candidate(
        &config_candidates,
        "invalid",
        SourceLocator::ConfigPath(ConfigPath::new(["invalid"]).expect("path")),
        CandidateStateName::Invalid,
    );
    assert_candidate(
        &config_candidates,
        "absent",
        SourceLocator::ConfigPath(ConfigPath::new(["absent"]).expect("path")),
        CandidateStateName::ExplicitAbsent,
    );
    assert_candidate(
        &config_candidates,
        "missing",
        SourceLocator::ConfigPath(ConfigPath::new(["missing"]).expect("path")),
        CandidateStateName::Missing,
    );

    let custom_candidates = CustomSource::new(BTreeMap::from([
        ("present".to_owned(), raw_present("custom")),
        (
            "invalid".to_owned(),
            RawSourceValue::Invalid {
                received: Some(Value::from("bad")),
                reason: "parse failed".to_owned(),
            },
        ),
        ("absent".to_owned(), RawSourceValue::ExplicitAbsent),
    ]))
    .extract(&custom_source, &field_set);
    assert_candidate(
        &custom_candidates,
        "present",
        SourceLocator::Custom("present".to_owned()),
        CandidateStateName::Present,
    );
    assert_candidate(
        &custom_candidates,
        "invalid",
        SourceLocator::Custom("invalid".to_owned()),
        CandidateStateName::Invalid,
    );
    assert_candidate(
        &custom_candidates,
        "absent",
        SourceLocator::Custom("absent".to_owned()),
        CandidateStateName::ExplicitAbsent,
    );
    assert_candidate(
        &custom_candidates,
        "missing",
        SourceLocator::Custom("missing".to_owned()),
        CandidateStateName::Missing,
    );

    let default_candidates = DefaultSource::new(BTreeMap::from([
        (
            "invalid-default".to_owned(),
            RawSourceValue::Invalid {
                received: Some(Value::from("bad")),
                reason: "default failed".to_owned(),
            },
        ),
        ("absent-default".to_owned(), RawSourceValue::ExplicitAbsent),
    ]))
    .extract(&default_source, &field_set);
    assert_candidate(
        &default_candidates,
        "static_default",
        SourceLocator::Default("static_default".to_owned()),
        CandidateStateName::DefaultFallback,
    );
    assert_candidate(
        &default_candidates,
        "invalid",
        SourceLocator::Default("invalid".to_owned()),
        CandidateStateName::Invalid,
    );
    assert_candidate(
        &default_candidates,
        "absent",
        SourceLocator::Default("absent".to_owned()),
        CandidateStateName::ExplicitAbsent,
    );
    assert_candidate(
        &default_candidates,
        "missing",
        SourceLocator::Default("missing".to_owned()),
        CandidateStateName::Missing,
    );
}

// @case WB-PARAM-RESOLVE-001
#[test]
fn ordered_resolution_selects_highest_applicable_source_and_records_overrides() {
    let limit = built_field("limit", ValueKind::Integer);
    let cli = source("cli", SourceKind::Cli, 30);
    let env = source("env", SourceKind::Env, 20).with_applicability(false);
    let config = source("config", SourceKind::Config, 10);
    let source_set = sources(vec![cli.clone(), env.clone(), config.clone()]);
    let field_set = FieldSet::builder()
        .add_field(limit.clone())
        .build()
        .expect("fields");

    let result = Resolver::resolve(
        &field_set,
        &source_set,
        vec![
            candidate(
                &limit,
                &config,
                SourceLocator::ConfigPath(ConfigPath::new(["limit"]).expect("path")),
                CandidateState::Present(Value::from(10_i64)),
            ),
            candidate(
                &limit,
                &env,
                SourceLocator::EnvVar("APP_LIMIT".to_owned()),
                CandidateState::Present(Value::from(20_i64)),
            ),
            candidate(
                &limit,
                &cli,
                SourceLocator::CliFlag("--limit".to_owned()),
                CandidateState::Present(Value::from(30_i64)),
            ),
        ],
    );

    let resolved = result.fields().get(limit.identity()).expect("resolution");
    assert_eq!(resolved.value(), Some(&Value::from(30_i64)));
    assert_eq!(
        resolved
            .trace()
            .selected
            .as_ref()
            .map(|trace| &trace.source_id),
        Some(cli.id())
    );
    assert_eq!(
        resolved
            .trace()
            .overridden
            .iter()
            .map(|trace| trace.source_id.as_str())
            .collect::<Vec<_>>(),
        vec!["config"]
    );
    assert!(result.diagnostics().is_empty());
    assert_eq!(
        result
            .materialize()
            .expect("materialized values")
            .get(limit.identity()),
        Some(&Value::from(30_i64))
    );
}

#[test]
fn explain_api_reports_stable_lines_from_trace() {
    let limit = field("limit", ValueKind::Integer)
        .projection(FieldProjectionDeclaration::cli_flag("--limit"))
        .projection(FieldProjectionDeclaration::config_path(["limit"]).expect("path"))
        .default(DefaultMetadata::Static(Value::from(5_i64)))
        .build()
        .expect("limit field");
    let include = field("include", ValueKind::List)
        .projection(FieldProjectionDeclaration::cli_flag("--include"))
        .projection(FieldProjectionDeclaration::config_path(["include"]).expect("path"))
        .merge_strategy(MergeStrategy::ListAppend)
        .build()
        .expect("include field");
    let cli = source("cli", SourceKind::Cli, 30);
    let config = source("config", SourceKind::Config, 20);
    let default = source("default", SourceKind::Default, 0);
    let field_set = FieldSet::builder()
        .add_field(limit.clone())
        .add_field(include.clone())
        .build()
        .expect("fields");
    let source_set = sources(vec![cli.clone(), config.clone(), default.clone()]);
    let mut candidates = DefaultSource::default().extract(&default, &field_set);
    candidates.extend([
        candidate(
            &limit,
            &config,
            SourceLocator::ConfigPath(ConfigPath::new(["limit"]).expect("path")),
            CandidateState::Present(Value::from(10_i64)),
        ),
        candidate(
            &limit,
            &cli,
            SourceLocator::CliFlag("--limit".to_owned()),
            CandidateState::Present(Value::from(30_i64)),
        ),
        candidate(
            &include,
            &config,
            SourceLocator::ConfigPath(ConfigPath::new(["include"]).expect("path")),
            CandidateState::Present(Value::List(vec![Value::from("config")])),
        ),
        candidate(
            &include,
            &cli,
            SourceLocator::CliFlag("--include".to_owned()),
            CandidateState::Present(Value::List(vec![Value::from("cli")])),
        ),
    ]);

    let explanation = Resolver::resolve(&field_set, &source_set, candidates).explain();

    assert_eq!(
        explanation
            .fields()
            .iter()
            .map(|field| field.field().as_str())
            .collect::<Vec<_>>(),
        vec!["include", "limit"]
    );
    let limit_explanation = explanation
        .fields()
        .iter()
        .find(|field| field.field().as_str() == "limit")
        .expect("limit explanation");
    assert_eq!(
        limit_explanation
            .selected()
            .map(|candidate| candidate.source_id().as_str()),
        Some("cli")
    );
    assert_eq!(
        limit_explanation
            .overridden()
            .iter()
            .map(|candidate| candidate.source_id().as_str())
            .collect::<Vec<_>>(),
        vec!["config"]
    );
    assert_eq!(
        limit_explanation
            .default_fallback()
            .map(|candidate| candidate.source_id().as_str()),
        Some("default")
    );

    let lines = explanation.lines();
    assert!(lines.contains(
        &"field=limit selected source=cli kind=cli locator=--limit state=present value=Integer(30)"
            .to_owned()
    ));
    assert!(lines.contains(&"field=limit default-fallback source=default kind=default locator=limit state=default-fallback value=Integer(5)".to_owned()));
    assert!(lines.contains(&"field=include merge-contributor source=cli kind=cli locator=--include state=present value=List([String(\"cli\")])".to_owned()));
}

#[test]
fn invalid_default_fallback_does_not_block_valid_explicit_value() {
    let limit = field("limit", ValueKind::Integer)
        .default(DefaultMetadata::Static(Value::from("not-an-integer")))
        .build()
        .expect("field allows deferred default validation");
    let cli = source("cli", SourceKind::Cli, 10);
    let default = source("default", SourceKind::Default, 0);
    let field_set = FieldSet::builder()
        .add_field(limit.clone())
        .build()
        .expect("fields");
    let source_set = sources(vec![cli.clone(), default.clone()]);
    let mut candidates = DefaultSource::default().extract(&default, &field_set);
    candidates.push(candidate(
        &limit,
        &cli,
        SourceLocator::CliFlag("--limit".to_owned()),
        CandidateState::Present(Value::from(25_i64)),
    ));

    let result = Resolver::resolve(&field_set, &source_set, candidates);

    assert!(result.diagnostics().is_empty());
    assert_eq!(
        result
            .fields()
            .get(limit.identity())
            .and_then(FieldResolution::value),
        Some(&Value::from(25_i64))
    );
    assert_eq!(
        result
            .materialize()
            .expect("materialize")
            .get(limit.identity()),
        Some(&Value::from(25_i64))
    );
    assert!(result
        .fields()
        .get(limit.identity())
        .expect("trace")
        .trace()
        .invalid_candidates
        .iter()
        .any(|candidate| candidate.locator == SourceLocator::Default("limit".to_owned())));
}

#[test]
fn merge_strategies_cover_list_map_replace_and_conflict() {
    let list = field("include", ValueKind::List)
        .merge_strategy(MergeStrategy::ListAppend)
        .build()
        .expect("list field");
    let list_replace = field("ordered_include", ValueKind::List)
        .merge_strategy(MergeStrategy::ListReplace)
        .build()
        .expect("list replace field");
    let map_merge = field("labels", ValueKind::Map)
        .merge_strategy(MergeStrategy::MapMerge)
        .build()
        .expect("map merge field");
    let map_replace = field("env", ValueKind::Map)
        .merge_strategy(MergeStrategy::MapReplace)
        .build()
        .expect("map replace field");
    let exclusive = field("mode", ValueKind::String)
        .merge_strategy(MergeStrategy::DenyConflict)
        .build()
        .expect("exclusive field");
    let cli = source("cli", SourceKind::Cli, 20);
    let config = source("config", SourceKind::Config, 10);
    let field_set = FieldSet::builder()
        .add_field(list.clone())
        .add_field(list_replace.clone())
        .add_field(map_merge.clone())
        .add_field(map_replace.clone())
        .add_field(exclusive.clone())
        .build()
        .expect("fields");
    let source_set = sources(vec![cli.clone(), config.clone()]);

    let result = Resolver::resolve(
        &field_set,
        &source_set,
        vec![
            candidate(
                &list,
                &cli,
                SourceLocator::CliFlag("--include".to_owned()),
                CandidateState::Present(Value::List(vec![Value::from("cli")])),
            ),
            candidate(
                &list,
                &config,
                SourceLocator::ConfigPath(ConfigPath::new(["include"]).expect("path")),
                CandidateState::Present(Value::List(vec![Value::from("config")])),
            ),
            candidate(
                &list_replace,
                &cli,
                SourceLocator::CliFlag("--ordered-include".to_owned()),
                CandidateState::Present(Value::List(vec![Value::from("cli")])),
            ),
            candidate(
                &list_replace,
                &config,
                SourceLocator::ConfigPath(ConfigPath::new(["ordered_include"]).expect("path")),
                CandidateState::Present(Value::List(vec![Value::from("config")])),
            ),
            candidate(
                &map_merge,
                &cli,
                SourceLocator::CliFlag("--label".to_owned()),
                CandidateState::Present(map(vec![
                    ("shared", Value::from("cli")),
                    ("only_cli", Value::from("yes")),
                ])),
            ),
            candidate(
                &map_merge,
                &config,
                SourceLocator::ConfigPath(ConfigPath::new(["labels"]).expect("path")),
                CandidateState::Present(map(vec![
                    ("shared", Value::from("config")),
                    ("only_config", Value::from("yes")),
                ])),
            ),
            candidate(
                &map_replace,
                &cli,
                SourceLocator::CliFlag("--env".to_owned()),
                CandidateState::Present(map(vec![("cli", Value::from("yes"))])),
            ),
            candidate(
                &map_replace,
                &config,
                SourceLocator::ConfigPath(ConfigPath::new(["env"]).expect("path")),
                CandidateState::Present(map(vec![("config", Value::from("yes"))])),
            ),
            candidate(
                &exclusive,
                &cli,
                SourceLocator::CliFlag("--mode".to_owned()),
                CandidateState::Present(Value::from("fast")),
            ),
            candidate(
                &exclusive,
                &config,
                SourceLocator::ConfigPath(ConfigPath::new(["mode"]).expect("path")),
                CandidateState::Present(Value::from("safe")),
            ),
        ],
    );

    assert_eq!(
        result
            .fields()
            .get(list.identity())
            .and_then(FieldResolution::value),
        Some(&Value::List(vec![
            Value::from("cli"),
            Value::from("config")
        ]))
    );
    assert_eq!(
        result
            .fields()
            .get(list.identity())
            .expect("list trace")
            .trace()
            .merge_contributors
            .len(),
        2
    );
    assert_eq!(
        result
            .fields()
            .get(list_replace.identity())
            .and_then(FieldResolution::value),
        Some(&Value::List(vec![Value::from("cli")]))
    );
    assert_eq!(
        result
            .fields()
            .get(map_merge.identity())
            .and_then(FieldResolution::value),
        Some(&map(vec![
            ("shared", Value::from("cli")),
            ("only_cli", Value::from("yes")),
            ("only_config", Value::from("yes")),
        ]))
    );
    assert_eq!(
        result
            .fields()
            .get(map_replace.identity())
            .and_then(FieldResolution::value),
        Some(&map(vec![("cli", Value::from("yes"))]))
    );
    assert!(result.diagnostics().iter().any(|diagnostic| matches!(
        diagnostic.reason,
        DiagnosticReason::MergeConflict(MergeConflictReason::DenyConflict)
    )));
}

#[test]
fn nullable_aggregate_merge_treats_null_as_priority_boundary() {
    let list = field("include", ValueKind::List)
        .nullable()
        .merge_strategy(MergeStrategy::ListAppend)
        .build()
        .expect("nullable list field");
    let map_field = field("labels", ValueKind::Map)
        .nullable()
        .merge_strategy(MergeStrategy::MapMerge)
        .build()
        .expect("nullable map field");
    let cli = source("cli", SourceKind::Cli, 30);
    let env = source("env", SourceKind::Env, 20);
    let config = source("config", SourceKind::Config, 10);
    let field_set = FieldSet::builder()
        .add_field(list.clone())
        .add_field(map_field.clone())
        .build()
        .expect("fields");
    let source_set = sources(vec![cli.clone(), env.clone(), config.clone()]);

    let result = Resolver::resolve(
        &field_set,
        &source_set,
        vec![
            candidate(
                &list,
                &cli,
                SourceLocator::CliFlag("--include".to_owned()),
                CandidateState::Present(Value::List(vec![Value::from("cli")])),
            ),
            candidate(
                &list,
                &env,
                SourceLocator::EnvVar("APP_INCLUDE".to_owned()),
                CandidateState::Present(Value::Null),
            ),
            candidate(
                &list,
                &config,
                SourceLocator::ConfigPath(ConfigPath::new(["include"]).expect("path")),
                CandidateState::Present(Value::List(vec![Value::from("config")])),
            ),
            candidate(
                &map_field,
                &cli,
                SourceLocator::CliFlag("--labels".to_owned()),
                CandidateState::Present(Value::Null),
            ),
            candidate(
                &map_field,
                &config,
                SourceLocator::ConfigPath(ConfigPath::new(["labels"]).expect("path")),
                CandidateState::Present(map(vec![("config", Value::from("yes"))])),
            ),
        ],
    );

    assert!(result.diagnostics().is_empty());
    assert_eq!(
        result
            .fields()
            .get(list.identity())
            .and_then(FieldResolution::value),
        Some(&Value::List(vec![Value::from("cli")]))
    );
    assert_eq!(
        result
            .fields()
            .get(map_field.identity())
            .and_then(FieldResolution::value),
        Some(&Value::Null)
    );
    let list_trace = result
        .fields()
        .get(list.identity())
        .expect("list resolution")
        .trace();
    assert_eq!(list_trace.merge_contributors.len(), 1);
    assert_eq!(list_trace.overridden.len(), 2);
    let map_trace = result
        .fields()
        .get(map_field.identity())
        .expect("map resolution")
        .trace();
    assert!(map_trace.merge_contributors.is_empty());
    assert_eq!(map_trace.overridden.len(), 1);
    result
        .materialize()
        .expect("nullable aggregates materialize");
}

#[test]
fn replace_and_deny_conflict_diagnostics_include_all_conflicting_locators() {
    let replace = built_field("replace", ValueKind::String);
    let deny = field("deny", ValueKind::String)
        .merge_strategy(MergeStrategy::DenyConflict)
        .build()
        .expect("deny field");
    let first = source("first", SourceKind::Custom("first".to_owned()), 10);
    let second = source("second", SourceKind::Custom("second".to_owned()), 10);
    let field_set = FieldSet::builder()
        .add_field(replace.clone())
        .add_field(deny.clone())
        .build()
        .expect("fields");
    let source_set = sources(vec![first.clone(), second.clone()]);

    let result = Resolver::resolve(
        &field_set,
        &source_set,
        vec![
            candidate(
                &replace,
                &first,
                SourceLocator::Custom("replace:first".to_owned()),
                CandidateState::Present(Value::from("first")),
            ),
            candidate(
                &replace,
                &second,
                SourceLocator::Custom("replace:second".to_owned()),
                CandidateState::Present(Value::from("second")),
            ),
            candidate(
                &deny,
                &first,
                SourceLocator::Custom("deny:first".to_owned()),
                CandidateState::Present(Value::from("first")),
            ),
            candidate(
                &deny,
                &second,
                SourceLocator::Custom("deny:second".to_owned()),
                CandidateState::Present(Value::from("second")),
            ),
        ],
    );

    assert_conflict_locators(
        result.diagnostics(),
        replace.identity(),
        MergeConflictReason::SamePriorityReplace,
        vec!["replace:first", "replace:second"],
    );
    assert_conflict_locators(
        result.diagnostics(),
        deny.identity(),
        MergeConflictReason::DenyConflict,
        vec!["deny:first", "deny:second"],
    );
    assert_eq!(
        result
            .fields()
            .get(replace.identity())
            .expect("replace trace")
            .trace()
            .invalid_candidates
            .iter()
            .map(|candidate| candidate.locator.as_key())
            .collect::<Vec<_>>(),
        vec!["replace:first".to_owned(), "replace:second".to_owned()]
    );
    assert_eq!(
        result
            .fields()
            .get(deny.identity())
            .expect("deny trace")
            .trace()
            .invalid_candidates
            .iter()
            .map(|candidate| candidate.locator.as_key())
            .collect::<Vec<_>>(),
        vec!["deny:first".to_owned(), "deny:second".to_owned()]
    );
}

#[test]
fn required_optional_default_and_value_presence_rules_are_distinct() {
    let required = field("required", ValueKind::String)
        .required()
        .build()
        .expect("required");
    let nullable = field("nullable", ValueKind::String)
        .required()
        .nullable()
        .build()
        .expect("nullable");
    let flag = built_field("flag", ValueKind::Boolean);
    let empty_list = built_field("empty_list", ValueKind::List);
    let empty_map = built_field("empty_map", ValueKind::Map);
    let with_default = field("with_default", ValueKind::String)
        .default(DefaultMetadata::Static(Value::from("fallback")))
        .build()
        .expect("default");
    let cli = source("cli", SourceKind::Cli, 10);
    let default = source("default", SourceKind::Default, 0);
    let field_set = FieldSet::builder()
        .add_field(required.clone())
        .add_field(nullable.clone())
        .add_field(flag.clone())
        .add_field(empty_list.clone())
        .add_field(empty_map.clone())
        .add_field(with_default.clone())
        .build()
        .expect("fields");
    let source_set = sources(vec![cli.clone(), default.clone()]);

    let mut candidates = DefaultSource::default().extract(&default, &field_set);
    candidates.extend([
        candidate(
            &nullable,
            &cli,
            SourceLocator::CliFlag("--nullable".to_owned()),
            CandidateState::Present(Value::Null),
        ),
        candidate(
            &flag,
            &cli,
            SourceLocator::CliFlag("--flag".to_owned()),
            CandidateState::Present(Value::from(false)),
        ),
        candidate(
            &empty_list,
            &cli,
            SourceLocator::CliFlag("--empty-list".to_owned()),
            CandidateState::Present(Value::List(Vec::new())),
        ),
        candidate(
            &empty_map,
            &cli,
            SourceLocator::CliFlag("--empty-map".to_owned()),
            CandidateState::Present(Value::Map(ValueMap::new())),
        ),
    ]);
    let result = Resolver::resolve(&field_set, &source_set, candidates);

    assert!(result.diagnostics().iter().any(|diagnostic| {
        diagnostic.field == *required.identity()
            && matches!(
                diagnostic.reason,
                DiagnosticReason::ValidationFailed(ValidationReason::MissingRequired)
            )
    }));
    assert!(
        result
            .fields()
            .get(required.identity())
            .expect("required trace")
            .trace()
            .missing_required
    );
    assert_eq!(
        result
            .fields()
            .get(nullable.identity())
            .and_then(FieldResolution::value),
        Some(&Value::Null)
    );
    assert_eq!(
        result
            .fields()
            .get(flag.identity())
            .and_then(FieldResolution::value),
        Some(&Value::from(false))
    );
    assert_eq!(
        result
            .fields()
            .get(empty_list.identity())
            .and_then(FieldResolution::value),
        Some(&Value::List(Vec::new()))
    );
    assert_eq!(
        result
            .fields()
            .get(empty_map.identity())
            .and_then(FieldResolution::value),
        Some(&Value::Map(ValueMap::new()))
    );
    assert_eq!(
        result
            .fields()
            .get(with_default.identity())
            .and_then(FieldResolution::value),
        Some(&Value::from("fallback"))
    );
    assert!(matches!(
        result
            .fields()
            .get(with_default.identity())
            .expect("default trace")
            .trace()
            .selected
            .as_ref()
            .map(|trace| &trace.state),
        Some(CandidateTraceState::DefaultFallback)
    ));
}

#[test]
fn default_source_preserves_declared_default_projection_locator() {
    let field_with_projection = field("runtime_threads", ValueKind::Integer)
        .default(DefaultMetadata::Dynamic(
            DynamicDefaultMetadata::new("threads").expect("dynamic default"),
        ))
        .projection(FieldProjectionDeclaration::default(
            "defaults.runtime.threads",
        ))
        .build()
        .expect("field");
    let field_without_projection = field("fallback_threads", ValueKind::Integer)
        .default(DefaultMetadata::Static(Value::from(4_i64)))
        .build()
        .expect("field");
    let field_set = FieldSet::builder()
        .add_field(field_with_projection.clone())
        .add_field(field_without_projection.clone())
        .build()
        .expect("fields");
    let default = source("default", SourceKind::Default, 0);
    let candidates =
        DefaultSource::new(BTreeMap::from([("threads".to_owned(), raw_present(8_i64))]))
            .extract(&default, &field_set);

    assert_candidate(
        &candidates,
        "runtime_threads",
        SourceLocator::Default("defaults.runtime.threads".to_owned()),
        CandidateStateName::DefaultFallback,
    );
    assert_candidate(
        &candidates,
        "fallback_threads",
        SourceLocator::Default("fallback_threads".to_owned()),
        CandidateStateName::DefaultFallback,
    );
}

#[test]
fn diagnostics_keep_source_facts_and_materialization_blocks_partial_success() {
    let limit = field("limit", ValueKind::Integer)
        .required()
        .build()
        .expect("limit");
    let cli = source("cli", SourceKind::Cli, 10);
    let field_set = FieldSet::builder()
        .add_field(limit.clone())
        .build()
        .expect("fields");
    let source_set = sources(vec![cli.clone()]);
    let result = Resolver::resolve(
        &field_set,
        &source_set,
        vec![candidate(
            &limit,
            &cli,
            SourceLocator::CliFlag("--limit".to_owned()),
            CandidateState::Present(Value::from("not-a-number")),
        )],
    );

    let diagnostic = result.diagnostics().first().expect("diagnostic");
    assert_eq!(diagnostic.field, *limit.identity());
    assert_eq!(diagnostic.source_id.as_ref(), Some(cli.id()));
    assert_eq!(
        diagnostic.locator.as_ref(),
        Some(&SourceLocator::CliFlag("--limit".to_owned()))
    );
    assert_eq!(diagnostic.received_kind, Some(ReceivedValueKind::String));
    assert!(matches!(
        diagnostic.reason,
        DiagnosticReason::ValidationFailed(ValidationReason::WrongType { .. })
    ));
    assert!(result.materialize().is_err());
    assert!(result
        .fields()
        .get(limit.identity())
        .expect("trace")
        .trace()
        .invalid_candidates
        .iter()
        .any(|candidate| candidate.locator == SourceLocator::CliFlag("--limit".to_owned())));
}

#[test]
fn application_materialization_hook_is_not_called_when_diagnostics_exist() {
    let limit = built_field("limit", ValueKind::Integer);
    let mode = built_field("mode", ValueKind::String);
    let cli = source("cli", SourceKind::Cli, 10);
    let field_set = FieldSet::builder()
        .add_field(limit.clone())
        .add_field(mode.clone())
        .build()
        .expect("fields");
    let source_set = sources(vec![cli.clone()]);
    let success = Resolver::resolve(
        &field_set,
        &source_set,
        vec![
            candidate(
                &limit,
                &cli,
                SourceLocator::CliFlag("--limit".to_owned()),
                CandidateState::Present(Value::from(10_i64)),
            ),
            candidate(
                &mode,
                &cli,
                SourceLocator::CliFlag("--mode".to_owned()),
                CandidateState::Present(Value::from("fast")),
            ),
        ],
    );
    let rendered = success
        .try_materialize_with(|values| {
            let limit = values.get(limit.identity()).expect("limit");
            let mode = values.get(mode.identity()).expect("mode");
            format!("{limit:?}:{mode:?}")
        })
        .expect("mapper success");
    assert!(rendered.contains("Integer(10)"));
    assert!(rendered.contains("String(\"fast\")"));

    let invalid = Resolver::resolve(
        &field_set,
        &source_set,
        vec![candidate(
            &limit,
            &cli,
            SourceLocator::CliFlag("--limit".to_owned()),
            CandidateState::Present(Value::from("bad")),
        )],
    );
    let mut called = false;
    let error = invalid
        .try_materialize_with(|_| {
            called = true;
            "partial"
        })
        .expect_err("diagnostics block mapper");
    assert!(!called);
    assert!(!error.diagnostics().is_empty());
}

#[test]
fn same_priority_replace_conflict_is_diagnostic_and_deterministic() {
    let mode = built_field("mode", ValueKind::String);
    let first = source("first", SourceKind::Custom("profile".to_owned()), 10);
    let second = source("second", SourceKind::Custom("workspace".to_owned()), 10);
    let field_set = FieldSet::builder()
        .add_field(mode.clone())
        .build()
        .expect("fields");
    let source_set = sources(vec![first.clone(), second.clone()]);
    let result = Resolver::resolve(
        &field_set,
        &source_set,
        vec![
            candidate(
                &mode,
                &first,
                SourceLocator::Custom("mode".to_owned()),
                CandidateState::Present(Value::from("profile")),
            ),
            candidate(
                &mode,
                &second,
                SourceLocator::Custom("mode".to_owned()),
                CandidateState::Present(Value::from("workspace")),
            ),
        ],
    );

    assert!(result
        .fields()
        .get(mode.identity())
        .and_then(FieldResolution::value)
        .is_none());
    assert!(result.diagnostics().iter().any(|diagnostic| matches!(
        diagnostic.reason,
        DiagnosticReason::AmbiguousPriority { priority: 10 }
    )));
    assert!(result.materialize().is_err());
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CandidateStateName {
    Missing,
    Present,
    Invalid,
    ExplicitAbsent,
    DefaultFallback,
}

fn assert_candidate(
    candidates: &[SourceCandidate],
    field: &str,
    locator: SourceLocator,
    expected_state: CandidateStateName,
) {
    let candidate = candidates
        .iter()
        .find(|candidate| candidate.field().as_str() == field && candidate.locator() == &locator)
        .expect("candidate");
    assert_eq!(state_name(candidate.state()), expected_state);
}

fn state_name(state: &CandidateState) -> CandidateStateName {
    match state {
        CandidateState::Missing => CandidateStateName::Missing,
        CandidateState::Present(_) => CandidateStateName::Present,
        CandidateState::Invalid { .. } => CandidateStateName::Invalid,
        CandidateState::ExplicitAbsent => CandidateStateName::ExplicitAbsent,
        CandidateState::DefaultFallback { .. } => CandidateStateName::DefaultFallback,
    }
}

fn assert_conflict_locators(
    diagnostics: &[ResolutionDiagnostic],
    field: &FieldIdentity,
    reason: MergeConflictReason,
    expected_locators: Vec<&str>,
) {
    let actual = diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.field == *field
                && matches!(
                    &diagnostic.reason,
                    DiagnosticReason::MergeConflict(actual_reason) if *actual_reason == reason
                )
        })
        .map(|diagnostic| diagnostic.locator.as_ref().expect("locator").as_key())
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        expected_locators
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>()
    );
}
