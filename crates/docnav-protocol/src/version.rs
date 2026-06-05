use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::{max, min};
use std::fmt;
use std::str::FromStr;

use crate::constants::fields;
use crate::StableError;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ProtocolVersion {
    major: u32,
    minor: u32,
}

impl ProtocolVersion {
    pub const fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }

    pub const fn major(self) -> u32 {
        self.major
    }

    pub const fn minor(self) -> u32 {
        self.minor
    }
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}", self.major, self.minor)
    }
}

impl FromStr for ProtocolVersion {
    type Err = VersionParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (major, minor) = value
            .split_once('.')
            .ok_or_else(|| VersionParseError(value.to_owned()))?;

        if major.is_empty() || minor.is_empty() || minor.contains('.') {
            return Err(VersionParseError(value.to_owned()));
        }

        let major = major
            .parse::<u32>()
            .map_err(|_| VersionParseError(value.to_owned()))?;
        let minor = minor
            .parse::<u32>()
            .map_err(|_| VersionParseError(value.to_owned()))?;

        Ok(Self::new(major, minor))
    }
}

impl Serialize for ProtocolVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ProtocolVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        raw.parse().map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionParseError(String);

impl fmt::Display for VersionParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid protocol version: {}", self.0)
    }
}

impl std::error::Error for VersionParseError {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProtocolRange {
    pub min: ProtocolVersion,
    pub max: ProtocolVersion,
}

impl ProtocolRange {
    pub fn new(min: ProtocolVersion, max: ProtocolVersion) -> Result<Self, ProtocolRangeError> {
        if min > max {
            return Err(ProtocolRangeError { min, max });
        }
        Ok(Self { min, max })
    }

    pub const fn v0_1() -> Self {
        Self {
            min: ProtocolVersion::new(0, 1),
            max: ProtocolVersion::new(0, 1),
        }
    }

    pub fn contains(&self, version: ProtocolVersion) -> bool {
        self.min <= version && version <= self.max
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolRangeError {
    pub min: ProtocolVersion,
    pub max: ProtocolVersion,
}

impl fmt::Display for ProtocolRangeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "protocol range min {} is greater than max {}",
            self.min, self.max
        )
    }
}

impl std::error::Error for ProtocolRangeError {}

pub fn select_highest_compatible(
    docnav: &ProtocolRange,
    adapter: &ProtocolRange,
) -> Result<ProtocolVersion, StableError> {
    let lower = max(docnav.min, adapter.min);
    let upper = min(docnav.max, adapter.max);

    if lower <= upper {
        Ok(upper)
    } else {
        Err(StableError::protocol_incompatible(
            format!("{}..{}", docnav.min, docnav.max),
            adapter.min.to_string(),
            adapter.max.to_string(),
        ))
    }
}

pub fn ensure_supported_protocol(
    requested: &str,
    supported: &ProtocolRange,
) -> Result<ProtocolVersion, StableError> {
    let requested_version = requested.parse::<ProtocolVersion>().map_err(|_| {
        StableError::invalid_request(
            fields::PROTOCOL_VERSION,
            format!("invalid protocol version {requested:?}"),
        )
    })?;

    if supported.contains(requested_version) {
        Ok(requested_version)
    } else {
        Err(StableError::protocol_incompatible(
            requested,
            supported.min.to_string(),
            supported.max.to_string(),
        ))
    }
}
