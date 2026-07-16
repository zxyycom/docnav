#![forbid(unsafe_code)]
//! `clap` projection for canonical `cli-config-resolution` fields.

use std::collections::BTreeSet;
use std::fmt;

use clap::parser::ValueSource;
use clap::{Arg, ArgAction, ArgMatches, Command};
use cli_config_resolution::{
    FieldDefSet, FieldIdentity, JsonValue, ProcessingId, ProcessingLocator, Source,
    SourceCandidate, SourceError, SourceId, SourceKind, SourceLocator, ValueKind,
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
        if command_conflicts(&collision_view, &projection.flag) {
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum FlagKind {
    Long(String),
    Short(char),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FlagSpec {
    raw: String,
    id: String,
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
                    id: long.clone(),
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
                        id: value.to_string(),
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
    let mut argument_ids = BTreeSet::new();
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
            if !argument_ids.insert(flag.id.clone()) {
                return Err(ClapProjectionError::ArgumentConflict {
                    field,
                    flag: flag.raw,
                });
            }
            Ok(CliProjection {
                field,
                flag,
                value_kind: metadata.value_kind,
            })
        })
        .collect()
}

fn argument_from_projection(projection: &CliProjection) -> Result<Arg, ClapProjectionError> {
    let mut argument = Arg::new(projection.flag.id.clone());
    argument = match &projection.flag.kind {
        FlagKind::Long(flag) => argument.long(flag.clone()),
        FlagKind::Short(flag) => argument.short(*flag),
    };
    match projection.value_kind {
        ValueKind::Boolean => Ok(argument.action(ArgAction::SetTrue)),
        ValueKind::Array | ValueKind::Object => Ok(argument.action(ArgAction::Append).num_args(1)),
        ValueKind::String => Ok(argument.action(ArgAction::Set).num_args(1)),
        ValueKind::Integer | ValueKind::Number => Ok(argument
            .action(ArgAction::Set)
            .num_args(1)
            .allow_negative_numbers(true)),
        ValueKind::Json => Err(ClapProjectionError::UnsupportedValueKind {
            field: projection.field.clone(),
            value_kind: projection.value_kind,
        }),
    }
}

fn candidate_from_matches(
    matches: &ArgMatches,
    projection: &CliProjection,
) -> Result<Option<SourceCandidate>, ClapProjectionError> {
    matches
        .try_contains_id(&projection.flag.id)
        .map_err(|error| match_read_error(projection, error.to_string()))?;
    if matches.value_source(&projection.flag.id) != Some(ValueSource::CommandLine) {
        return Ok(None);
    }

    let locator = SourceLocator::CliFlag(projection.flag.raw.clone());
    let candidate = match projection.value_kind {
        ValueKind::Boolean => {
            let value = matches
                .try_get_one::<bool>(&projection.flag.id)
                .map_err(|error| match_read_error(projection, error.to_string()))?
                .copied()
                .ok_or_else(|| match_read_error(projection, "explicit flag has no value"))?;
            SourceCandidate::value(projection.field.clone(), locator, JsonValue::Bool(value))
        }
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
        .try_get_many::<String>(&projection.flag.id)
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

fn command_conflicts(command: &Command, flag: &FlagSpec) -> bool {
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
            .any(|group| group.get_id().as_str() == flag.id)
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
            argument.get_id().as_str() == flag.id
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

#[cfg(test)]
mod tests;
