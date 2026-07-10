#![forbid(unsafe_code)]
//! `clap` companion helpers for `cli-config-resolution`.

use std::collections::BTreeSet;
use std::fmt;

use clap::parser::ValueSource;
use clap::{Arg, ArgAction, ArgMatches, Command};
use cli_config_resolution::{
    CandidateState, FieldContract, FieldSet, SourceCandidate, SourceKind, SourceLocator,
    SourceSpec, Value, ValueKind,
};

pub fn augment_command(
    command: Command,
    fields: &FieldSet,
) -> Result<Command, ClapProjectionError> {
    let mut command = command;
    for arg in args_from_fields(fields)? {
        command = command.arg(arg);
    }
    Ok(command)
}

pub fn args_from_fields(fields: &FieldSet) -> Result<Vec<Arg>, ClapProjectionError> {
    let mut seen_ids = BTreeSet::new();
    let mut args = Vec::new();
    for field in fields.fields() {
        for projection in field
            .projections()
            .iter()
            .filter(|projection| projection.source_kind() == &SourceKind::Cli)
        {
            let spec = FlagSpec::parse(projection.locator())?;
            if !seen_ids.insert(spec.id.clone()) {
                return Err(ClapProjectionError::DuplicateArgId {
                    id: spec.id,
                    locator: projection.locator().clone(),
                });
            }
            args.push(arg_from_projection(field, spec));
        }
    }
    Ok(args)
}

pub fn candidates_from_matches(
    matches: &ArgMatches,
    source: &SourceSpec,
    fields: &FieldSet,
) -> Vec<SourceCandidate> {
    if source.kind() != &SourceKind::Cli {
        return Vec::new();
    }
    fields
        .fields()
        .iter()
        .flat_map(|field| {
            field
                .projections()
                .iter()
                .filter(|projection| projection.source_kind() == &SourceKind::Cli)
                .map(|projection| {
                    let state = match FlagSpec::parse(projection.locator()) {
                        Ok(spec) => candidate_state_from_matches(matches, &spec.id, field),
                        Err(error) => CandidateState::Invalid {
                            received: None,
                            reason: error.to_string(),
                        },
                    };
                    SourceCandidate::new(
                        projection.field().clone(),
                        source,
                        projection.locator().clone(),
                        state,
                    )
                })
        })
        .collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClapProjectionError {
    UnsupportedLocator(SourceLocator),
    InvalidFlag(String),
    DuplicateArgId { id: String, locator: SourceLocator },
}

impl fmt::Display for ClapProjectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedLocator(locator) => {
                write!(formatter, "unsupported clap locator {:?}", locator)
            }
            Self::InvalidFlag(flag) => write!(formatter, "invalid clap flag {flag}"),
            Self::DuplicateArgId { id, locator } => write!(
                formatter,
                "clap argument id {id} is generated more than once; duplicate locator {:?}",
                locator
            ),
        }
    }
}

impl std::error::Error for ClapProjectionError {}

#[derive(Clone, Debug, Eq, PartialEq)]
enum FlagKind {
    Long(String),
    Short(char),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FlagSpec {
    id: String,
    kind: FlagKind,
}

impl FlagSpec {
    fn parse(locator: &SourceLocator) -> Result<Self, ClapProjectionError> {
        let SourceLocator::CliFlag(flag) = locator else {
            return Err(ClapProjectionError::UnsupportedLocator(locator.clone()));
        };
        let trimmed = flag.trim();
        if trimmed.is_empty() {
            return Err(ClapProjectionError::InvalidFlag(flag.clone()));
        }
        if let Some(long) = trimmed.strip_prefix("--") {
            if long.is_empty() || long.chars().any(char::is_whitespace) {
                return Err(ClapProjectionError::InvalidFlag(flag.clone()));
            }
            return Ok(Self {
                id: long.to_owned(),
                kind: FlagKind::Long(long.to_owned()),
            });
        }
        if let Some(short) = trimmed.strip_prefix('-') {
            let mut chars = short.chars();
            let Some(value) = chars.next() else {
                return Err(ClapProjectionError::InvalidFlag(flag.clone()));
            };
            if chars.next().is_some() || value.is_whitespace() {
                return Err(ClapProjectionError::InvalidFlag(flag.clone()));
            }
            return Ok(Self {
                id: value.to_string(),
                kind: FlagKind::Short(value),
            });
        }
        if trimmed.chars().any(char::is_whitespace) {
            return Err(ClapProjectionError::InvalidFlag(flag.clone()));
        }
        Ok(Self {
            id: trimmed.to_owned(),
            kind: FlagKind::Long(trimmed.to_owned()),
        })
    }
}

fn arg_from_projection(field: &FieldContract, spec: FlagSpec) -> Arg {
    let mut arg = Arg::new(spec.id.clone());
    arg = match spec.kind {
        FlagKind::Long(flag) => arg.long(flag),
        FlagKind::Short(flag) => arg.short(flag),
    };
    match field.value_kind() {
        ValueKind::Boolean => arg.action(ArgAction::SetTrue),
        ValueKind::List | ValueKind::Map => arg.action(ArgAction::Append).num_args(1),
        ValueKind::String | ValueKind::Integer | ValueKind::Number | ValueKind::Any => {
            arg.action(ArgAction::Set).num_args(1)
        }
    }
}

fn candidate_state_from_matches(
    matches: &ArgMatches,
    id: &str,
    field: &FieldContract,
) -> CandidateState {
    if matches.value_source(id) != Some(ValueSource::CommandLine) {
        return CandidateState::Missing;
    }
    match field.value_kind() {
        ValueKind::Boolean => read_bool(matches, id),
        ValueKind::List => read_strings(matches, id)
            .map(|values| {
                CandidateState::Present(Value::List(values.into_iter().map(Value::from).collect()))
            })
            .unwrap_or_else(invalid_reason),
        ValueKind::Map => read_strings(matches, id)
            .and_then(map_from_entries)
            .map(CandidateState::Present)
            .unwrap_or_else(invalid_reason),
        ValueKind::String => read_one_string(matches, id)
            .map(|value| CandidateState::Present(Value::from(value)))
            .unwrap_or_else(invalid_reason),
        ValueKind::Integer => read_one_string(matches, id)
            .and_then(|value| {
                value
                    .parse::<i64>()
                    .map(Value::Integer)
                    .map_err(|_| format!("expected integer value for {id}"))
            })
            .map(CandidateState::Present)
            .unwrap_or_else(invalid_reason),
        ValueKind::Number => read_one_string(matches, id)
            .and_then(|value| {
                value
                    .parse::<f64>()
                    .map(Value::Number)
                    .map_err(|_| format!("expected number value for {id}"))
            })
            .map(CandidateState::Present)
            .unwrap_or_else(invalid_reason),
        ValueKind::Any => read_strings(matches, id)
            .map(|values| match values.as_slice() {
                [value] => CandidateState::Present(Value::from(value.clone())),
                _ => CandidateState::Present(Value::List(
                    values.into_iter().map(Value::from).collect(),
                )),
            })
            .unwrap_or_else(invalid_reason),
    }
}

fn read_bool(matches: &ArgMatches, id: &str) -> CandidateState {
    if let Ok(Some(value)) = matches.try_get_one::<bool>(id) {
        return CandidateState::Present(Value::Boolean(*value));
    }
    read_one_string(matches, id)
        .and_then(|value| match value.as_str() {
            "true" => Ok(Value::Boolean(true)),
            "false" => Ok(Value::Boolean(false)),
            _ => Err(format!("expected boolean value for {id}")),
        })
        .map(CandidateState::Present)
        .unwrap_or_else(invalid_reason)
}

fn read_one_string(matches: &ArgMatches, id: &str) -> Result<String, String> {
    let values = read_strings(matches, id)?;
    match values.as_slice() {
        [value] => Ok(value.clone()),
        _ => Err(format!("expected one value for {id}")),
    }
}

fn read_strings(matches: &ArgMatches, id: &str) -> Result<Vec<String>, String> {
    match matches.try_get_many::<String>(id) {
        Ok(Some(values)) => Ok(values.cloned().collect()),
        Ok(None) => Ok(Vec::new()),
        Err(error) => Err(format!("could not read clap values for {id}: {error}")),
    }
}

fn map_from_entries(entries: Vec<String>) -> Result<Value, String> {
    let mut map = cli_config_resolution::ValueMap::new();
    for entry in entries {
        let Some((key, value)) = entry.split_once('=') else {
            return Err(format!("expected key=value map entry, got {entry}"));
        };
        if key.trim().is_empty() {
            return Err("expected non-empty map key".to_owned());
        }
        map.insert(key.to_owned(), Value::from(value.to_owned()));
    }
    Ok(Value::Map(map))
}

fn invalid_reason(reason: String) -> CandidateState {
    CandidateState::Invalid {
        received: None,
        reason,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use cli_config_resolution::{
        CandidateState, EnvVarSource, FieldContract, FieldProjectionDeclaration, FieldSet,
        RawSourceValue, Resolver, SourceCollection, SourceExtractor, SourceId, SourceKind,
        SourceLocator, SourceSpec, Value, ValueKind,
    };

    use super::{augment_command, candidates_from_matches};

    // @case WB-PARAM-CLAP-001
    fn source_id(value: &str) -> SourceId {
        SourceId::new(value).expect("source id")
    }

    fn source(value: &str, kind: SourceKind, priority: i32) -> SourceSpec {
        SourceSpec::new(source_id(value), kind, priority)
    }

    fn field(identity: &str, kind: ValueKind, flag: &str) -> FieldContract {
        FieldContract::builder(identity, kind)
            .projection(FieldProjectionDeclaration::cli_flag(flag))
            .build()
            .expect("field")
    }

    fn fields(values: Vec<FieldContract>) -> FieldSet {
        values
            .into_iter()
            .fold(FieldSet::builder(), |builder, field| {
                builder.add_field(field)
            })
            .build()
            .expect("fields")
    }

    #[test]
    fn generated_args_read_matches_as_source_candidates() {
        let field_set = fields(vec![
            field("limit", ValueKind::Integer, "--limit"),
            field("include", ValueKind::List, "--include"),
            field("label", ValueKind::Map, "--label"),
            field("verbose", ValueKind::Boolean, "--verbose"),
            field("missing", ValueKind::String, "--missing"),
        ]);
        let command = augment_command(clap::Command::new("demo"), &field_set).expect("command");
        let matches = command
            .try_get_matches_from([
                "demo",
                "--limit",
                "42",
                "--include",
                "src",
                "--include",
                "tests",
                "--label",
                "team=docs",
                "--verbose",
            ])
            .expect("matches");
        let cli = source("cli", SourceKind::Cli, 40);

        let candidates = candidates_from_matches(&matches, &cli, &field_set);

        assert_candidate_value(&candidates, "limit", Value::Integer(42));
        assert_candidate_value(
            &candidates,
            "include",
            Value::List(vec![Value::from("src"), Value::from("tests")]),
        );
        assert_candidate_value(
            &candidates,
            "label",
            Value::Map(cli_config_resolution::ValueMap::from([(
                "team".to_owned(),
                Value::from("docs"),
            )])),
        );
        assert_candidate_value(&candidates, "verbose", Value::Boolean(true));
        assert!(matches!(
            candidate_state(&candidates, "missing"),
            CandidateState::Missing
        ));
    }

    #[test]
    fn typed_read_errors_are_invalid_candidates() {
        let field_set = fields(vec![field("limit", ValueKind::Integer, "--limit")]);
        let command = augment_command(clap::Command::new("demo"), &field_set).expect("command");
        let matches = command
            .try_get_matches_from(["demo", "--limit", "not-an-integer"])
            .expect("matches");
        let cli = source("cli", SourceKind::Cli, 40);

        let candidates = candidates_from_matches(&matches, &cli, &field_set);

        assert!(matches!(
            candidate_state(&candidates, "limit"),
            CandidateState::Invalid { reason, .. } if reason == "expected integer value for limit"
        ));
    }

    #[test]
    fn omitted_boolean_is_a_missing_cli_candidate() {
        let field_set = fields(vec![field("verbose", ValueKind::Boolean, "--verbose")]);
        let command = augment_command(clap::Command::new("demo"), &field_set).expect("command");
        let matches = command.try_get_matches_from(["demo"]).expect("matches");
        let cli = source("cli", SourceKind::Cli, 40);

        assert_eq!(
            matches.value_source("verbose"),
            Some(clap::parser::ValueSource::DefaultValue)
        );
        let candidates = candidates_from_matches(&matches, &cli, &field_set);

        assert!(matches!(
            candidate_state(&candidates, "verbose"),
            CandidateState::Missing
        ));
    }

    #[test]
    fn omitted_boolean_does_not_override_lower_priority_true() {
        let verbose = FieldContract::builder("verbose", ValueKind::Boolean)
            .projection(FieldProjectionDeclaration::cli_flag("--verbose"))
            .projection(FieldProjectionDeclaration::env_var("APP_VERBOSE"))
            .build()
            .expect("field");
        let field_set = fields(vec![verbose]);
        let command = augment_command(clap::Command::new("demo"), &field_set).expect("command");
        let matches = command.try_get_matches_from(["demo"]).expect("matches");
        let cli = source("cli", SourceKind::Cli, 40);
        let env = source("env", SourceKind::Env, 30);
        let sources =
            SourceCollection::new(vec![cli.clone(), env.clone()]).expect("source collection");
        let mut candidates = candidates_from_matches(&matches, &cli, &field_set);
        candidates.extend(
            EnvVarSource::new(BTreeMap::from([(
                "APP_VERBOSE".to_owned(),
                RawSourceValue::Present(Value::Boolean(true)),
            )]))
            .extract(&env, &field_set),
        );

        let result = Resolver::resolve(&field_set, &sources, candidates);
        let resolution = result
            .fields()
            .get(field_set.fields()[0].identity())
            .expect("verbose resolution");

        assert_eq!(resolution.value(), Some(&Value::Boolean(true)));
        assert_eq!(
            resolution
                .trace()
                .selected
                .as_ref()
                .map(|candidate| candidate.source_kind.clone()),
            Some(SourceKind::Env)
        );
        assert!(result.diagnostics().is_empty());
    }

    #[test]
    fn non_cli_source_returns_no_candidates() {
        let field_set = fields(vec![field("limit", ValueKind::Integer, "--limit")]);
        let command = augment_command(clap::Command::new("demo"), &field_set).expect("command");
        let matches = command
            .try_get_matches_from(["demo", "--limit", "10"])
            .expect("matches");
        let config = source("config", SourceKind::Config, 20);

        assert!(candidates_from_matches(&matches, &config, &field_set).is_empty());
    }

    fn assert_candidate_value(
        candidates: &[cli_config_resolution::SourceCandidate],
        field: &str,
        expected: Value,
    ) {
        assert_eq!(
            candidate_state(candidates, field),
            &CandidateState::Present(expected)
        );
    }

    fn candidate_state<'a>(
        candidates: &'a [cli_config_resolution::SourceCandidate],
        field: &str,
    ) -> &'a CandidateState {
        candidates
            .iter()
            .find(|candidate| {
                candidate.field().as_str() == field
                    && matches!(candidate.locator(), SourceLocator::CliFlag(_))
            })
            .expect("candidate")
            .state()
    }
}
