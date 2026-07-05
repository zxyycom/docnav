use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Options {
    values: Map<String, Value>,
    entries: Vec<OptionEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OptionEntry {
    pub identity: String,
    pub owner: String,
    pub namespace: String,
    pub key: String,
    pub source: String,
    pub type_variant: String,
    pub value: Value,
}

impl Options {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: String, value: Value) -> Option<Value> {
        self.entries.retain(|entry| entry.key != key);
        self.values.insert(key, value)
    }

    pub fn insert_entry(&mut self, entry: OptionEntry) -> Option<Value> {
        let previous = self.values.insert(entry.key.clone(), entry.value.clone());
        self.entries.push(entry);
        previous
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.values.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn entries(&self) -> &[OptionEntry] {
        &self.entries
    }

    pub fn entry_for(&self, owner: &str, namespace: &str, key: &str) -> Option<&OptionEntry> {
        self.entries
            .iter()
            .find(|entry| entry.owner == owner && entry.namespace == namespace && entry.key == key)
    }

    pub fn entry_for_key(&self, key: &str) -> Option<&OptionEntry> {
        self.entries.iter().find(|entry| entry.key == key)
    }
}

impl FromIterator<(String, Value)> for Options {
    fn from_iter<T: IntoIterator<Item = (String, Value)>>(iter: T) -> Self {
        let mut options = Self::new();
        for (key, value) in iter {
            options.insert(key, value);
        }
        options
    }
}

impl Serialize for Options {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.values.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Options {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let values = Map::<String, Value>::deserialize(deserializer)?;
        Ok(Self {
            values,
            entries: Vec::new(),
        })
    }
}
