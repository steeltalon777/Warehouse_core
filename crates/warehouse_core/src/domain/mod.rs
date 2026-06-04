//! Domain models, DTOs, and mapping layer.
//!
//! This module contains:
//! - Server DTOs (mirroring SyncServer Pydantic schemas)
//! - Local domain entities
//! - Public UI DTOs (stable API for UI clients)
//! - FFI-safe DTOs
//! - Mapping functions between layers

pub mod admin;
pub mod assets;
pub mod auth;
pub mod balance;
pub mod catalog;
pub mod documents;
pub mod issue_objects;
pub mod operation;
pub mod pagination;
pub mod recipient;
pub mod reports;
pub mod site;
pub mod sync_types;
pub mod temporary_items;
pub mod tools;

/// Serde helpers for handling flexible deserialization.
pub mod serde_helpers {
    use serde::de;

    /// Deserialize a field that can be either a string or a number into String.
    /// Used for operation IDs that SyncServer returns as UUID strings
    /// (backward compatible with numeric IDs).
    pub fn string_or_number<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct StringOrNumber;
        impl<'de> de::Visitor<'de> for StringOrNumber {
            type Value = String;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a string or number")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<String, E> {
                Ok(value.to_string())
            }
            fn visit_i64<E: de::Error>(self, value: i64) -> Result<String, E> {
                Ok(value.to_string())
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<String, E> {
                Ok(value.to_string())
            }
        }
        deserializer.deserialize_any(StringOrNumber)
    }
}
