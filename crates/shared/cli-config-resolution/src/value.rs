use std::collections::BTreeMap;

pub type ValueMap = BTreeMap<String, Value>;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    String(String),
    Integer(i64),
    Number(f64),
    Boolean(bool),
    List(Vec<Value>),
    Map(ValueMap),
    Null,
}

impl Value {
    pub fn received_kind(&self) -> ReceivedValueKind {
        match self {
            Self::String(_) => ReceivedValueKind::String,
            Self::Integer(_) => ReceivedValueKind::Integer,
            Self::Number(_) => ReceivedValueKind::Number,
            Self::Boolean(_) => ReceivedValueKind::Boolean,
            Self::List(_) => ReceivedValueKind::List,
            Self::Map(_) => ReceivedValueKind::Map,
            Self::Null => ReceivedValueKind::Null,
        }
    }

    pub(crate) fn len(&self) -> Option<usize> {
        match self {
            Self::String(value) => Some(value.chars().count()),
            Self::List(value) => Some(value.len()),
            Self::Map(value) => Some(value.len()),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValueKind {
    String,
    Integer,
    Number,
    Boolean,
    List,
    Map,
    Any,
}

impl ValueKind {
    pub fn accepts(self, received: ReceivedValueKind) -> bool {
        match self {
            Self::String => received == ReceivedValueKind::String,
            Self::Integer => received == ReceivedValueKind::Integer,
            Self::Number => {
                received == ReceivedValueKind::Number || received == ReceivedValueKind::Integer
            }
            Self::Boolean => received == ReceivedValueKind::Boolean,
            Self::List => received == ReceivedValueKind::List,
            Self::Map => received == ReceivedValueKind::Map,
            Self::Any => true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceivedValueKind {
    String,
    Integer,
    Number,
    Boolean,
    List,
    Map,
    Null,
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}
