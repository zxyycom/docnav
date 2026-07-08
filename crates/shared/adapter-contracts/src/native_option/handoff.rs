use docnav_protocol::{OptionEntry, Options};
use serde_json::Value;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NativeOptionHandoff {
    entries: Vec<NativeOptionValue>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NativeOptionValue {
    pub identity: String,
    pub owner: String,
    pub namespace: String,
    pub key: String,
    pub source: String,
    pub type_variant: String,
    pub value: Value,
}

impl NativeOptionHandoff {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn from_options(options: Option<&Options>) -> Self {
        let entries = options
            .map(|options| {
                options
                    .entries()
                    .iter()
                    .map(NativeOptionValue::from)
                    .collect()
            })
            .unwrap_or_default();
        Self { entries }
    }

    pub fn entries(&self) -> &[NativeOptionValue] {
        &self.entries
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn get(&self, owner: &str, namespace: &str, key: &str) -> Option<&NativeOptionValue> {
        self.entries
            .iter()
            .find(|entry| entry.owner == owner && entry.namespace == namespace && entry.key == key)
    }

    pub fn get_key(&self, key: &str) -> Option<&NativeOptionValue> {
        self.entries.iter().find(|entry| entry.key == key)
    }
}

impl From<&OptionEntry> for NativeOptionValue {
    fn from(entry: &OptionEntry) -> Self {
        Self {
            identity: entry.identity.clone(),
            owner: entry.owner.clone(),
            namespace: entry.namespace.clone(),
            key: entry.key.clone(),
            source: entry.source.clone(),
            type_variant: entry.type_variant.clone(),
            value: entry.value.clone(),
        }
    }
}
