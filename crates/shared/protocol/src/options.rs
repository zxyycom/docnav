use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Options {
    values: Map<String, Value>,
}

impl Options {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: String, value: Value) -> Option<Value> {
        self.values.insert(key, value)
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
        Ok(Self { values })
    }
}
