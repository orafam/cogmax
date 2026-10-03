use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MemoryScope(String);

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ScopeError {
    #[error("memory scope cannot be empty")]
    Empty,
    #[error("memory scope contains invalid characters")]
    InvalidCharacters,
}

impl MemoryScope {
    pub fn new(value: impl Into<String>) -> Result<Self, ScopeError> {
        let value = value.into();
        if value.is_empty() {
            return Err(ScopeError::Empty);
        }
        if value.chars().any(char::is_whitespace) {
            return Err(ScopeError::InvalidCharacters);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
