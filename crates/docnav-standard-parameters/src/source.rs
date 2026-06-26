use std::collections::BTreeMap;

use docnav_typed_fields::{FieldIdentity, JsonValue};

use crate::StandardParameterPath;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardParameterSourceKind {
    DirectInput,
    ProjectConfig,
    UserConfig,
    Default,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StandardParameterSourceInfo {
    pub kind: StandardParameterSourceKind,
}

impl StandardParameterSourceInfo {
    pub const fn new(kind: StandardParameterSourceKind) -> Self {
        Self { kind }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StandardParameterSource {
    values: BTreeMap<FieldIdentity, JsonValue>,
    passthrough: Vec<PassthroughInput>,
}

impl StandardParameterSource {
    pub fn insert_value(&mut self, identity: FieldIdentity, value: JsonValue) {
        self.values.insert(identity, value);
    }

    pub fn with_value(mut self, identity: FieldIdentity, value: JsonValue) -> Self {
        self.insert_value(identity, value);
        self
    }

    pub fn push_passthrough(&mut self, path: StandardParameterPath, value: JsonValue) {
        self.passthrough.push(PassthroughInput { path, value });
    }

    pub fn with_passthrough(mut self, path: StandardParameterPath, value: JsonValue) -> Self {
        self.push_passthrough(path, value);
        self
    }

    pub(crate) fn value(&self, identity: &FieldIdentity) -> Option<&JsonValue> {
        self.values.get(identity)
    }

    pub(crate) fn passthrough(&self) -> &[PassthroughInput] {
        &self.passthrough
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StandardParameterSources {
    pub direct_input: StandardParameterSource,
    pub project_config: StandardParameterSource,
    pub user_config: StandardParameterSource,
    pub default: StandardParameterSource,
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
pub struct PassthroughInput {
    pub path: StandardParameterPath,
    pub value: JsonValue,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PassthroughValue {
    pub source: StandardParameterSourceInfo,
    pub path: StandardParameterPath,
    pub value: JsonValue,
    pub disposition: PassthroughDisposition,
}
