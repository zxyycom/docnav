#![forbid(unsafe_code)]
//! `clap` projection for canonical `cli-config-resolution` fields.

use std::collections::BTreeSet;
use std::fmt;

use clap::parser::ValueSource;
use clap::{Arg, ArgAction, ArgMatches, Command};
use cli_config_resolution::{
    CliBooleanEncoding, CliProcessingMetadata, DefaultMetadata, FieldDefSet, FieldIdentity,
    JsonValue, ProcessingId, ProcessingLocator, Source, SourceCandidate, SourceError, SourceId,
    SourceKind, SourceLocator, ValueKind,
};

/// Adds the CLI arguments declared for `processing_id` to `command`.
pub fn augment_command(
    mut command: Command,
    fields: &FieldDefSet,
    processing_id: &ProcessingId,
) -> Result<Command, ClapProjectionError> {
    let projections = cli_projections(fields, processing_id)?;
    let mut collision_view = command.clone();
    collision_view.build();

    for projection in projections {
        if command_conflicts(&collision_view, &projection) {
            return Err(ClapProjectionError::ArgumentConflict {
                field: projection.field,
                flag: projection.flag.raw,
            });
        }
        command = command.arg(argument_from_projection(&projection)?);
    }

    Ok(command)
}

/// Extracts explicitly supplied CLI arguments into one core source.
pub fn extract_cli(
    matches: &ArgMatches,
    fields: &FieldDefSet,
    processing_id: &ProcessingId,
    source_id: SourceId,
    priority: i32,
) -> Result<Source, ClapProjectionError> {
    let projections = cli_projections(fields, processing_id)?;
    let mut candidates = Vec::new();
    for projection in &projections {
        if let Some(candidate) = candidate_from_matches(matches, projection)? {
            candidates.push(candidate);
        }
    }

    Source::new(source_id, SourceKind::Cli, priority, candidates).map_err(Into::into)
}

/// A deterministic failure while mapping canonical metadata to `clap`.
#[derive(Clone, Debug, PartialEq)]
pub enum ClapProjectionError {
    UnsupportedLocator {
        field: FieldIdentity,
        locator: ProcessingLocator,
    },
    InvalidFlag {
        field: FieldIdentity,
        flag: String,
    },
    UnsupportedValueKind {
        field: FieldIdentity,
        value_kind: ValueKind,
    },
    ArgumentConflict {
        field: FieldIdentity,
        flag: String,
    },
    MatchRead {
        field: FieldIdentity,
        flag: String,
        reason: String,
    },
    Source(SourceError),
}

impl fmt::Display for ClapProjectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedLocator { field, locator } => write!(
                formatter,
                "field {} uses non-CLI processing locator {locator:?}",
                field.as_str()
            ),
            Self::InvalidFlag { field, flag } => write!(
                formatter,
                "field {} has invalid CLI flag {flag}",
                field.as_str()
            ),
            Self::UnsupportedValueKind { field, value_kind } => write!(
                formatter,
                "field {} has unsupported CLI value kind {value_kind:?}",
                field.as_str()
            ),
            Self::ArgumentConflict { field, flag } => write!(
                formatter,
                "CLI flag {flag} for field {} conflicts with another clap argument",
                field.as_str()
            ),
            Self::MatchRead {
                field,
                flag,
                reason,
            } => write!(
                formatter,
                "could not read CLI flag {flag} for field {}: {reason}",
                field.as_str()
            ),
            Self::Source(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ClapProjectionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Source(error) => Some(error),
            _ => None,
        }
    }
}

impl From<SourceError> for ClapProjectionError {
    fn from(error: SourceError) -> Self {
        Self::Source(error)
    }
}

#[derive(Clone, Debug)]
struct CliProjection {
    field: FieldIdentity,
    flag: FlagSpec,
    value_kind: ValueKind,
    accepted_values: Option<Vec<JsonValue>>,
    default: DefaultMetadata,
    cli: CliProcessingMetadata,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum FlagKind {
    Long(String),
    Short(char),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FlagSpec {
    raw: String,
    kind: FlagKind,
}

impl FlagSpec {
    fn parse(field: &FieldIdentity, flag: String) -> Result<Self, ClapProjectionError> {
        if let Some(long) = flag.strip_prefix("--") {
            if !long.is_empty()
                && !long.starts_with('-')
                && long
                    .chars()
                    .all(|character| !character.is_whitespace() && character != '=')
            {
                let long = long.to_owned();
                return Ok(Self {
                    raw: flag,
                    kind: FlagKind::Long(long),
                });
            }
        } else if let Some(short) = flag.strip_prefix('-') {
            let mut characters = short.chars();
            if let Some(value) = characters.next() {
                if characters.next().is_none()
                    && !value.is_whitespace()
                    && value != '='
                    && value != '-'
                {
                    return Ok(Self {
                        raw: flag,
                        kind: FlagKind::Short(value),
                    });
                }
            }
        }

        Err(ClapProjectionError::InvalidFlag {
            field: field.clone(),
            flag,
        })
    }
}

fn cli_projections(
    fields: &FieldDefSet,
    processing_id: &ProcessingId,
) -> Result<Vec<CliProjection>, ClapProjectionError> {
    let mut locators = BTreeSet::new();
    fields
        .processing_metadata(processing_id)
        .into_iter()
        .map(|metadata| {
            let field = metadata.identity;
            let flag = match metadata.locator {
                ProcessingLocator::CliFlag(flag) => FlagSpec::parse(&field, flag)?,
                locator => {
                    return Err(ClapProjectionError::UnsupportedLocator { field, locator });
                }
            };
            if metadata.value_kind == ValueKind::Json {
                return Err(ClapProjectionError::UnsupportedValueKind {
                    field,
                    value_kind: metadata.value_kind,
                });
            }
            if !locators.insert(flag.raw.clone()) {
                return Err(ClapProjectionError::ArgumentConflict {
                    field,
                    flag: flag.raw,
                });
            }
            Ok(CliProjection {
                field,
                flag,
                value_kind: metadata.value_kind,
                accepted_values: metadata.constraints.enum_values,
                default: metadata.default,
                cli: metadata.cli.unwrap_or_default(),
            })
        })
        .collect()
}

fn argument_from_projection(projection: &CliProjection) -> Result<Arg, ClapProjectionError> {
    let mut argument = Arg::new(projection.field.as_str().to_owned());
    argument = match &projection.flag.kind {
        FlagKind::Long(flag) => argument.long(flag.clone()),
        FlagKind::Short(flag) => argument.short(*flag),
    };
    if let Some(help) = projection.help() {
        argument = argument.help(help);
    }

    let takes_value = !matches!(
        (&projection.value_kind, &projection.cli.boolean_encoding),
        (
            ValueKind::Boolean,
            None | Some(CliBooleanEncoding::PresenceMeansTrue)
        )
    );
    if takes_value {
        if let Some(value_name) = &projection.cli.value_name {
            argument = argument.value_name(value_name.clone());
        }
    }

    match (&projection.value_kind, &projection.cli.boolean_encoding) {
        (ValueKind::Boolean, None | Some(CliBooleanEncoding::PresenceMeansTrue)) => {
            Ok(argument.action(ArgAction::SetTrue))
        }
        (ValueKind::Boolean, Some(CliBooleanEncoding::Explicit { .. })) => {
            Ok(argument.action(ArgAction::Set).num_args(1))
        }
        (ValueKind::Array | ValueKind::Object, _) => {
            Ok(argument.action(ArgAction::Append).num_args(1))
        }
        (ValueKind::String, _) => Ok(argument.action(ArgAction::Set).num_args(1)),
        (ValueKind::Integer | ValueKind::Number, _) => Ok(argument
            .action(ArgAction::Set)
            .num_args(1)
            .allow_negative_numbers(true)),
        (ValueKind::Json, _) => Err(ClapProjectionError::UnsupportedValueKind {
            field: projection.field.clone(),
            value_kind: projection.value_kind,
        }),
    }
}

fn candidate_from_matches(
    matches: &ArgMatches,
    projection: &CliProjection,
) -> Result<Option<SourceCandidate>, ClapProjectionError> {
    let argument_id = projection.field.as_str().to_owned();
    matches
        .try_contains_id(&argument_id)
        .map_err(|error| match_read_error(projection, error.to_string()))?;
    if matches.value_source(&argument_id) != Some(ValueSource::CommandLine) {
        return Ok(None);
    }

    let locator = SourceLocator::CliFlag(projection.flag.raw.clone());
    let candidate = match projection.value_kind {
        ValueKind::Boolean => match &projection.cli.boolean_encoding {
            None | Some(CliBooleanEncoding::PresenceMeansTrue) => {
                let value = matches
                    .try_get_one::<bool>(&argument_id)
                    .map_err(|error| match_read_error(projection, error.to_string()))?
                    .copied()
                    .ok_or_else(|| match_read_error(projection, "explicit flag has no value"))?;
                SourceCandidate::value(projection.field.clone(), locator, JsonValue::Bool(value))
            }
            Some(CliBooleanEncoding::Explicit {
                true_token: Some(true_token),
                false_token: Some(false_token),
            }) => {
                let raw = read_one_string(matches, projection)?;
                if raw == *true_token {
                    SourceCandidate::value(projection.field.clone(), locator, JsonValue::Bool(true))
                } else if raw == *false_token {
                    SourceCandidate::value(
                        projection.field.clone(),
                        locator,
                        JsonValue::Bool(false),
                    )
                } else {
                    SourceCandidate::invalid(
                        projection.field.clone(),
                        locator,
                        JsonValue::String(raw),
                        format!("expected Boolean CLI token {true_token:?} or {false_token:?}"),
                    )
                }
            }
            Some(CliBooleanEncoding::Explicit { .. }) => {
                return Err(match_read_error(
                    projection,
                    "canonical Boolean CLI token mapping is incomplete",
                ));
            }
        },
        ValueKind::String => {
            let raw = read_one_string(matches, projection)?;
            SourceCandidate::value(projection.field.clone(), locator, JsonValue::String(raw))
        }
        ValueKind::Integer => {
            let raw = read_one_string(matches, projection)?;
            match raw.parse::<i64>() {
                Ok(value) => SourceCandidate::value(
                    projection.field.clone(),
                    locator,
                    JsonValue::from(value),
                ),
                Err(_) => SourceCandidate::invalid(
                    projection.field.clone(),
                    locator,
                    JsonValue::String(raw),
                    "expected integer CLI value",
                ),
            }
        }
        ValueKind::Number => {
            let raw = read_one_string(matches, projection)?;
            match raw.parse::<f64>() {
                Ok(value) if value.is_finite() => SourceCandidate::value(
                    projection.field.clone(),
                    locator,
                    JsonValue::from(value),
                ),
                _ => SourceCandidate::invalid(
                    projection.field.clone(),
                    locator,
                    JsonValue::String(raw),
                    "expected finite number CLI value",
                ),
            }
        }
        ValueKind::Array => {
            let raw = read_strings(matches, projection)?;
            let value = JsonValue::Array(raw.into_iter().map(JsonValue::String).collect());
            SourceCandidate::value(projection.field.clone(), locator, value)
        }
        ValueKind::Object => {
            let raw = read_strings(matches, projection)?;
            match decode_object_entries(&raw) {
                Ok(value) => SourceCandidate::value(projection.field.clone(), locator, value),
                Err(reason) => SourceCandidate::invalid(
                    projection.field.clone(),
                    locator,
                    JsonValue::Array(raw.into_iter().map(JsonValue::String).collect()),
                    reason,
                ),
            }
        }
        ValueKind::Json => {
            return Err(ClapProjectionError::UnsupportedValueKind {
                field: projection.field.clone(),
                value_kind: projection.value_kind,
            });
        }
    };
    Ok(Some(candidate))
}

fn read_one_string(
    matches: &ArgMatches,
    projection: &CliProjection,
) -> Result<String, ClapProjectionError> {
    let values = read_strings(matches, projection)?;
    match values.as_slice() {
        [value] => Ok(value.clone()),
        _ => Err(match_read_error(
            projection,
            "expected exactly one command-line value",
        )),
    }
}

fn read_strings(
    matches: &ArgMatches,
    projection: &CliProjection,
) -> Result<Vec<String>, ClapProjectionError> {
    let values = matches
        .try_get_many::<String>(projection.field.as_str())
        .map_err(|error| match_read_error(projection, error.to_string()))?
        .ok_or_else(|| match_read_error(projection, "explicit argument has no value"))?;
    let values = values.cloned().collect::<Vec<_>>();
    if values.is_empty() {
        Err(match_read_error(
            projection,
            "explicit argument has no value",
        ))
    } else {
        Ok(values)
    }
}

fn decode_object_entries(entries: &[String]) -> Result<JsonValue, String> {
    entries
        .iter()
        .map(|entry| {
            let Some((key, value)) = entry.split_once('=') else {
                return Err(format!("expected key=value CLI object entry, got {entry}"));
            };
            if key.trim().is_empty() {
                return Err("expected non-empty CLI object key".to_owned());
            }
            Ok((key.to_owned(), JsonValue::String(value.to_owned())))
        })
        .collect()
}

fn match_read_error(projection: &CliProjection, reason: impl Into<String>) -> ClapProjectionError {
    ClapProjectionError::MatchRead {
        field: projection.field.clone(),
        flag: projection.flag.raw.clone(),
        reason: reason.into(),
    }
}

fn command_conflicts(command: &Command, projection: &CliProjection) -> bool {
    let flag = &projection.flag;
    let argument_id = projection.field.as_str();
    let automatic_flag_conflicts = match &flag.kind {
        FlagKind::Long(value) => {
            (!command.is_disable_help_flag_set() && value == "help")
                || (!command.is_disable_version_flag_set() && value == "version")
        }
        FlagKind::Short(value) => {
            (!command.is_disable_help_flag_set() && *value == 'h')
                || (!command.is_disable_version_flag_set() && *value == 'V')
        }
    };
    automatic_flag_conflicts
        || command
            .get_groups()
            .any(|group| group.get_id().as_str() == argument_id)
        || command
            .get_subcommands()
            .any(|subcommand| match &flag.kind {
                FlagKind::Long(value) => {
                    subcommand.get_long_flag() == Some(value)
                        || subcommand
                            .get_all_long_flag_aliases()
                            .any(|alias| alias == value)
                }
                FlagKind::Short(value) => {
                    subcommand.get_short_flag() == Some(*value)
                        || subcommand
                            .get_all_short_flag_aliases()
                            .any(|alias| alias == *value)
                }
            })
        || command.get_arguments().any(|argument| {
            argument.get_id().as_str() == argument_id
                || match &flag.kind {
                    FlagKind::Long(value) => {
                        argument.get_long() == Some(value)
                            || argument
                                .get_all_aliases()
                                .is_some_and(|aliases| aliases.contains(&value.as_str()))
                    }
                    FlagKind::Short(value) => {
                        argument.get_short() == Some(*value)
                            || argument
                                .get_all_short_aliases()
                                .is_some_and(|aliases| aliases.contains(value))
                    }
                }
        })
}

impl CliProjection {
    fn help(&self) -> Option<String> {
        let mut canonical_facts = Vec::new();
        if let Some(accepted_values) = &self.accepted_values {
            canonical_facts.push(format!(
                "possible values: {}",
                accepted_values
                    .iter()
                    .map(display_json_value)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if let DefaultMetadata::Static(value) = &self.default {
            canonical_facts.push(format!("default: {}", display_json_value(value)));
        }

        match (&self.cli.help, canonical_facts.is_empty()) {
            (Some(help), true) => Some(help.clone()),
            (Some(help), false) => Some(format!("{help} [{}]", canonical_facts.join("; "))),
            (None, false) => Some(format!("[{}]", canonical_facts.join("; "))),
            (None, true) => None,
        }
    }
}

fn display_json_value(value: &JsonValue) -> String {
    match value {
        JsonValue::String(value) => value.clone(),
        value => value.to_string(),
    }
}

#[cfg(test)]
mod tests;
