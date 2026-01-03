use anyhow::{anyhow, Result};

/// Represents a field with a key and value, with support for identifying sensitive data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub key: String,
    pub value: String,
}

impl Field {
    pub fn new(key: String, value: String) -> Self {
        Self { key, value }
    }

    /// Check if a key represents sensitive information
    pub fn is_key_sensitive(key: &str) -> bool {
        let key_lower = key.to_lowercase();
        key_lower.contains("password")
            || key_lower.contains("secret")
            || key_lower.contains("token")
    }

    /// Checks if this field contains sensitive information based on its key
    pub fn is_sensitive(&self) -> bool {
        Self::is_key_sensitive(&self.key)
    }

    /// Returns the display value, masking sensitive fields with asterisks
    pub fn display_value(&self) -> String {
        if self.is_sensitive() {
            "*".repeat(8)
        } else {
            self.value.clone()
        }
    }

    /// Returns the formatted display string "key: value"
    pub fn display(&self) -> String {
        format!("{}: {}", self.key, self.display_value())
    }

    /// Parse a Field from a display string (format: "key: value")
    /// Note: The value will be the displayed value (may be masked for sensitive fields)
    pub fn parse_from_display(display: &str) -> Result<Field> {
        display
            .find(": ")
            .map(|pos| {
                let key = display[..pos].to_string();
                let value = display[pos + 2..].to_string();
                Field::new(key, value)
            })
            .ok_or_else(|| anyhow!("Invalid display format: '{}'", display))
    }
}
