use std::collections::BTreeSet;
use std::fmt::Write as _;

use docnav_typed_fields::{FieldDefSet, ProcessingId};
use serde_json::{Map, Value};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum ConfigKeyIssue {
    UnregisteredField { path: ConfigValuePath },
    ExpectedObject { path: ConfigValuePath },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct ConfigKeyRegistry {
    leaf_paths: BTreeSet<Vec<RegisteredPathSegment>>,
    container_paths: BTreeSet<Vec<RegisteredPathSegment>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ConfigValuePath(Vec<ConfigValuePathSegment>);

#[derive(Clone, Debug, Eq, PartialEq)]
enum ConfigValuePathSegment {
    Key(String),
    Index(usize),
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum RegisteredPathSegment {
    Key(String),
    ArrayItem,
}

impl ConfigKeyRegistry {
    pub(super) fn from_field_set(fields: &FieldDefSet, processing_id: &ProcessingId) -> Self {
        let mut registry = Self::default();
        registry.register_field_set(fields, processing_id);
        registry
    }

    pub(super) fn field_set(mut self, fields: &FieldDefSet, processing_id: &ProcessingId) -> Self {
        self.register_field_set(fields, processing_id);
        self
    }

    pub(super) fn leaf_path<I, S>(mut self, path: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.register_leaf_path(registered_key_path(path));
        self
    }

    pub(super) fn container_path<I, S>(mut self, path: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.register_container_path(registered_key_path(path));
        self
    }

    pub(super) fn array_item_leaf_path<I, S>(
        mut self,
        array_path: I,
        item_key: impl Into<String>,
    ) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut path = registered_key_path(array_path);
        path.push(RegisteredPathSegment::ArrayItem);
        path.push(RegisteredPathSegment::Key(item_key.into()));
        self.register_leaf_path(path);
        self
    }

    pub(super) fn first_issue(&self, root: &Value) -> Option<ConfigKeyIssue> {
        root.as_object()?;
        self.first_issue_at(root, &ConfigValuePath::root())
    }

    fn register_leaf_path(&mut self, path: Vec<RegisteredPathSegment>) {
        for end in 0..path.len() {
            self.container_paths.insert(path[..end].to_vec());
        }
        self.leaf_paths.insert(path);
    }

    fn register_field_set(&mut self, fields: &FieldDefSet, processing_id: &ProcessingId) {
        for metadata in fields.processing_metadata(processing_id) {
            self.register_leaf_path(
                metadata
                    .path
                    .segments()
                    .into_iter()
                    .map(|segment| RegisteredPathSegment::Key(segment.to_owned()))
                    .collect(),
            );
        }
    }

    fn register_container_path(&mut self, path: Vec<RegisteredPathSegment>) {
        for end in 0..=path.len() {
            self.container_paths.insert(path[..end].to_vec());
        }
    }

    fn first_issue_at(&self, value: &Value, path: &ConfigValuePath) -> Option<ConfigKeyIssue> {
        let registered_path = path.registered_path();
        if self.leaf_paths.contains(&registered_path) {
            return None;
        }
        if self.expects_array_at(&registered_path) {
            return self.first_array_item_issue(value, path);
        }

        let Value::Object(object) = value else {
            return self.expected_object_issue(path, &registered_path);
        };
        self.first_object_key_issue(object, path)
    }

    fn expects_array_at(&self, path: &[RegisteredPathSegment]) -> bool {
        let mut item_path = path.to_vec();
        item_path.push(RegisteredPathSegment::ArrayItem);
        self.container_paths.contains(&item_path)
    }

    fn first_array_item_issue(
        &self,
        value: &Value,
        path: &ConfigValuePath,
    ) -> Option<ConfigKeyIssue> {
        let Value::Array(items) = value else {
            return None;
        };
        for (index, item) in items.iter().enumerate() {
            if let Some(issue) = self.first_issue_at(item, &path.child_index(index)) {
                return Some(issue);
            }
        }
        None
    }

    fn expected_object_issue(
        &self,
        path: &ConfigValuePath,
        registered_path: &[RegisteredPathSegment],
    ) -> Option<ConfigKeyIssue> {
        (!path.is_root() && self.container_paths.contains(registered_path))
            .then(|| ConfigKeyIssue::ExpectedObject { path: path.clone() })
    }

    fn first_object_key_issue(
        &self,
        object: &Map<String, Value>,
        path: &ConfigValuePath,
    ) -> Option<ConfigKeyIssue> {
        let mut keys = object.keys().collect::<Vec<_>>();
        keys.sort();

        for key in keys {
            if let Some(issue) = self.issue_for_child_key(object, path, key) {
                return Some(issue);
            }
        }
        None
    }

    fn issue_for_child_key(
        &self,
        object: &Map<String, Value>,
        path: &ConfigValuePath,
        key: &str,
    ) -> Option<ConfigKeyIssue> {
        let child_path = path.child_key(key);
        let child_registered_path = child_path.registered_path();
        if self.leaf_paths.contains(&child_registered_path) {
            return None;
        }
        if self.container_paths.contains(&child_registered_path)
            || self.expects_array_at(&child_registered_path)
        {
            return self.first_issue_at(&object[key], &child_path);
        }
        Some(ConfigKeyIssue::UnregisteredField { path: child_path })
    }
}

impl ConfigValuePath {
    fn root() -> Self {
        Self(Vec::new())
    }

    fn is_root(&self) -> bool {
        self.0.is_empty()
    }

    fn child_key(&self, key: &str) -> Self {
        let mut path = self.0.clone();
        path.push(ConfigValuePathSegment::Key(key.to_owned()));
        Self(path)
    }

    fn child_index(&self, index: usize) -> Self {
        let mut path = self.0.clone();
        path.push(ConfigValuePathSegment::Index(index));
        Self(path)
    }

    pub(super) fn field(&self) -> String {
        let mut field = String::new();
        for segment in &self.0 {
            match segment {
                ConfigValuePathSegment::Key(key) => {
                    if !field.is_empty() {
                        field.push('.');
                    }
                    field.push_str(key);
                }
                ConfigValuePathSegment::Index(index) => {
                    let _ = write!(field, "[{index}]");
                }
            }
        }
        field
    }

    pub(super) fn option_adapter_id(&self) -> Option<&str> {
        if let [ConfigValuePathSegment::Key(namespace), ConfigValuePathSegment::Key(adapter_id), ..] =
            self.0.as_slice()
        {
            if namespace == "options" {
                return Some(adapter_id.as_str());
            }
        }
        None
    }

    pub(super) fn option_key(&self) -> Option<&str> {
        if let [ConfigValuePathSegment::Key(namespace), ConfigValuePathSegment::Key(_adapter_id), ConfigValuePathSegment::Key(key), ..] =
            self.0.as_slice()
        {
            if namespace == "options" {
                return Some(key.as_str());
            }
        }
        None
    }

    pub(super) fn value<'a>(&self, root: &'a Value) -> Option<&'a Value> {
        let mut current = root;
        for segment in &self.0 {
            match segment {
                ConfigValuePathSegment::Key(key) => current = current.get(key)?,
                ConfigValuePathSegment::Index(index) => current = current.get(*index)?,
            }
        }
        Some(current)
    }

    fn registered_path(&self) -> Vec<RegisteredPathSegment> {
        self.0
            .iter()
            .map(|segment| match segment {
                ConfigValuePathSegment::Key(key) => RegisteredPathSegment::Key(key.clone()),
                ConfigValuePathSegment::Index(_) => RegisteredPathSegment::ArrayItem,
            })
            .collect()
    }
}

fn registered_key_path<I, S>(path: I) -> Vec<RegisteredPathSegment>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    path.into_iter()
        .map(|segment| RegisteredPathSegment::Key(segment.into()))
        .collect()
}
