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
pub mod operation;
pub mod pagination;
pub mod recipient;
pub mod reports;
pub mod site;
pub mod sync_types;
pub mod temporary_items;
pub mod tools;
