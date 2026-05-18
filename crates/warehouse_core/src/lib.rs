#![allow(async_fn_in_trait)]

pub mod auth;
pub mod config;
pub mod domain;
pub mod error;
pub mod facade;
pub mod ids;
pub mod operations;
pub mod storage;
pub mod sync;
pub mod syncserver;
pub mod time;

pub use config::CoreConfig;
pub use error::{CoreError, CoreResult};
pub use facade::CoreHandle;
pub use ids::CoreIds;

pub const CLIENT_NAME: &str = "warehouse_client_core";
pub const CLIENT_VERSION: &str = env!("CARGO_PKG_VERSION");
