use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StandardParameterPath(Vec<String>);

impl StandardParameterPath {
    pub fn new<I, S>(segments: I) -> Result<Self, InvalidStandardParameterPath>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let segments = segments.into_iter().map(Into::into).collect::<Vec<_>>();
        if segments.is_empty() {
            return Err(InvalidStandardParameterPath::Empty);
        }
        if segments.iter().any(|segment| segment.is_empty()) {
            return Err(InvalidStandardParameterPath::EmptySegment);
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
pub enum InvalidStandardParameterPath {
    Empty,
    EmptySegment,
}

impl fmt::Display for InvalidStandardParameterPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("standard parameter path is empty"),
            Self::EmptySegment => {
                formatter.write_str("standard parameter path contains an empty segment")
            }
        }
    }
}

impl std::error::Error for InvalidStandardParameterPath {}
