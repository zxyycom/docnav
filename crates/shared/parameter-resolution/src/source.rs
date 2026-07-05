use std::collections::BTreeMap;

use docnav_typed_fields::{FieldIdentity, JsonValue};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParameterSourceKind {
    DirectInput,
    ProjectConfig,
    UserConfig,
    Default,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParameterSourceInfo {
    pub kind: ParameterSourceKind,
}

impl ParameterSourceInfo {
    pub const fn new(kind: ParameterSourceKind) -> Self {
        Self { kind }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ParameterSource {
    values: BTreeMap<FieldIdentity, JsonValue>,
    processing_result: Option<JsonValue>,
}

impl ParameterSource {
    pub(crate) fn insert_value(&mut self, identity: FieldIdentity, value: JsonValue) {
        self.values.insert(identity, value);
    }

    #[cfg(test)]
    pub(crate) fn with_value(mut self, identity: FieldIdentity, value: JsonValue) -> Self {
        self.insert_value(identity, value);
        self
    }

    pub(crate) fn set_processing_result(&mut self, value: JsonValue) {
        self.processing_result = Some(value);
    }

    pub(crate) fn value(&self, identity: &FieldIdentity) -> Option<&JsonValue> {
        self.values.get(identity)
    }

    pub(crate) fn processing_result(&self) -> Option<&JsonValue> {
        self.processing_result.as_ref()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ParameterSources {
    pub(crate) direct_input: ParameterSource,
    pub(crate) project_config: ParameterSource,
    pub(crate) user_config: ParameterSource,
    pub(crate) default: ParameterSource,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntryPassthroughPolicy {
    Retain,
    Discard,
    Delegate,
}

impl EntryPassthroughPolicy {
    pub(crate) const fn disposition(self) -> PassthroughDisposition {
        match self {
            Self::Retain => PassthroughDisposition::Retained,
            Self::Discard => PassthroughDisposition::Discarded,
            Self::Delegate => PassthroughDisposition::Delegated,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PassthroughDisposition {
    Retained,
    Discarded,
    Delegated,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PassthroughValue {
    pub source: ParameterSourceInfo,
    pub value: JsonValue,
    pub disposition: PassthroughDisposition,
}
