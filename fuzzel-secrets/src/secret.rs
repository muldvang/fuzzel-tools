use crate::field::Field;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Data structure for storing secret details, encapsulating fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secret {
    #[serde(flatten)]
    fields: HashMap<String, String>,
}

impl Secret {
    /// Create a new empty SecretData
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    /// Insert a field by providing key and value strings
    pub fn insert(&mut self, key: String, value: String) {
        self.fields.insert(key, value);
    }

    /// Get a field by key
    pub fn get(&self, key: &str) -> Option<Field> {
        self.fields
            .get(key)
            .map(|value| Field::new(key.to_string(), value.clone()))
    }

    /// Remove a field by key
    pub fn remove(&mut self, key: &str) -> Option<Field> {
        self.fields
            .remove(key)
            .map(|value| Field::new(key.to_string(), value.clone()))
    }

    /// Check if a key exists
    pub fn contains_key(&self, key: &str) -> bool {
        self.fields.contains_key(key)
    }

    /// Get the number of fields
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Iterate over all fields
    pub fn iter(&self) -> impl Iterator<Item = Field> + '_ {
        self.fields
            .iter()
            .map(|(k, v)| Field::new(k.clone(), v.clone()))
    }

    /// Get all field keys
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.fields.keys()
    }

    /// Get all fields as a vector
    pub fn fields(&self) -> Vec<Field> {
        self.iter().collect()
    }
}

impl Default for Secret {
    fn default() -> Self {
        Self::new()
    }
}

impl FromIterator<Field> for Secret {
    fn from_iter<I: IntoIterator<Item = Field>>(iter: I) -> Self {
        let mut data = Secret::new();
        for field in iter {
            data.insert(field.key, field.value);
        }
        data
    }
}

impl From<Vec<Field>> for Secret {
    fn from(fields: Vec<Field>) -> Self {
        fields.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut data = Secret::new();
        data.insert("username".to_string(), "john".to_string());

        let field = data.get("username").unwrap();
        assert_eq!(field.key, "username");
        assert_eq!(field.value, "john");
    }

    #[test]
    fn test_remove() {
        let mut data = Secret::new();
        data.insert("password".to_string(), "secret".to_string());

        let removed = data.remove("password").unwrap();
        assert_eq!(removed.value, "secret");
        assert!(data.is_empty());
    }

    #[test]
    fn test_from_vec() {
        let fields = vec![
            Field::new("username".to_string(), "john".to_string()),
            Field::new("password".to_string(), "secret".to_string()),
        ];

        let data = Secret::from(fields);
        assert_eq!(data.len(), 2);
        assert!(data.contains_key("username"));
        assert!(data.contains_key("password"));
    }

    #[test]
    fn test_iter() {
        let mut data = Secret::new();
        data.insert("key1".to_string(), "val1".to_string());
        data.insert("key2".to_string(), "val2".to_string());

        let keys: Vec<String> = data.iter().map(|f| f.key).collect();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
    }
}
