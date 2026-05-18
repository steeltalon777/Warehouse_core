use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Platform abstraction for injecting secrets into the core.
///
/// The core never stores raw token values in its own SQLite database.
/// Each platform (CLI, Android Keystore, .NET secure storage) implements
/// this trait to provide tokens at runtime.
///
/// Per ADR-0010: token ownership belongs to the platform secure storage.
pub trait TokenProvider: std::fmt::Debug + Send + Sync {
    fn user_token(&self) -> Option<Uuid>;
    fn device_token(&self) -> Option<Uuid>;
}

/// Null provider that always returns `None`.
#[derive(Debug, Clone, Copy)]
pub struct NullTokenProvider;

impl TokenProvider for NullTokenProvider {
    fn user_token(&self) -> Option<Uuid> {
        None
    }
    fn device_token(&self) -> Option<Uuid> {
        None
    }
}

/// CLI provider that reads tokens from environment variables.
///
/// Variables (checked in order):
/// - `WAREHOUSE_USER_TOKEN`, then `SYNC_USER_TOKEN`
/// - `WAREHOUSE_DEVICE_TOKEN`, then `SYNC_DEVICE_TOKEN`
#[derive(Debug, Clone)]
pub struct CliTokenProvider;

impl CliTokenProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CliTokenProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenProvider for CliTokenProvider {
    fn user_token(&self) -> Option<Uuid> {
        std::env::var("WAREHOUSE_USER_TOKEN")
            .ok()
            .or_else(|| std::env::var("SYNC_USER_TOKEN").ok())
            .and_then(|s| Uuid::parse_str(&s).ok())
    }

    fn device_token(&self) -> Option<Uuid> {
        std::env::var("WAREHOUSE_DEVICE_TOKEN")
            .ok()
            .or_else(|| std::env::var("SYNC_DEVICE_TOKEN").ok())
            .and_then(|s| Uuid::parse_str(&s).ok())
    }
}

/// FFI-compatible token provider that stores tokens in memory.
///
/// Used by `warehouse_ffi` when Android/WPF pass tokens from platform
/// secure storage. Tokens are held in `Arc<Mutex<>>` so the provider
/// can be shared between `CoreHandle` (which owns `Box<dyn TokenProvider>`)
/// and `CoreHandleWrapper` (which updates tokens via FFI calls).
///
/// Per ADR-0010: this holds tokens only in process memory, never in SQLite.
#[derive(Debug, Clone)]
pub struct FfiTokenProvider {
    inner: Arc<FfiTokenInner>,
}

#[derive(Debug, Default)]
struct FfiTokenInner {
    user_token: Mutex<Option<Uuid>>,
    device_token: Mutex<Option<Uuid>>,
}

impl FfiTokenProvider {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(FfiTokenInner::default()),
        }
    }

    pub fn set_user_token(&self, token: Uuid) {
        *self.inner.user_token.lock().unwrap() = Some(token);
    }

    pub fn clear_user_token(&self) {
        *self.inner.user_token.lock().unwrap() = None;
    }

    pub fn set_device_token(&self, token: Uuid) {
        *self.inner.device_token.lock().unwrap() = Some(token);
    }

    pub fn clear_device_token(&self) {
        *self.inner.device_token.lock().unwrap() = None;
    }
}

impl Default for FfiTokenProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenProvider for FfiTokenProvider {
    fn user_token(&self) -> Option<Uuid> {
        *self.inner.user_token.lock().unwrap()
    }

    fn device_token(&self) -> Option<Uuid> {
        *self.inner.device_token.lock().unwrap()
    }
}
