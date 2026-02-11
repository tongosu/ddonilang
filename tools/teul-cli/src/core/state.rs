use std::collections::BTreeMap;

use crate::core::value::Value;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key(pub String);

impl Key {
    pub fn new(name: impl Into<String>) -> Self {
        Key(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Default)]
pub struct State {
    pub resources: BTreeMap<Key, Value>,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &Key) -> Option<&Value> {
        self.resources.get(key)
    }

    pub fn set(&mut self, key: Key, value: Value) {
        self.resources.insert(key, value);
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }
}
