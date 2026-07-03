use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParameterPath(Vec<String>);

impl ParameterPath {
    pub fn new<I, S>(segments: I) -> Result<Self, InvalidParameterPath>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let segments = segments.into_iter().map(Into::into).collect::<Vec<_>>();
        if segments.is_empty() {
            return Err(InvalidParameterPath::Empty);
        }
        if segments.iter().any(|segment| segment.is_empty()) {
            return Err(InvalidParameterPath::EmptySegment);
        }
        Ok(Self(segments))
    }

    pub fn segments(&self) -> Vec<&str> {
        self.0.iter().map(String::as_str).collect()
    }

    pub(crate) fn key(&self) -> Vec<String> {
        self.0.clone()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidParameterPath {
    Empty,
    EmptySegment,
}

impl fmt::Display for InvalidParameterPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("parameter resolution path is empty"),
            Self::EmptySegment => {
                formatter.write_str("parameter resolution path contains an empty segment")
            }
        }
    }
}

impl std::error::Error for InvalidParameterPath {}
