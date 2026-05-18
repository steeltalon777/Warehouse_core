use std::ffi::CString;
use std::os::raw::c_char;
use warehouse_core::CoreError;

/// FFI-safe error envelope returned by all exported functions.
///
/// When `code != 0`, the caller must eventually free `message` and `details`
/// by calling `warehouse_free_string` on each pointer.
#[repr(C)]
pub struct CoreErrorDto {
    pub code: i32,
    pub message: *mut c_char,
    pub details: *mut c_char,
}

impl CoreErrorDto {
    pub fn success() -> Self {
        Self {
            code: 0,
            message: std::ptr::null_mut(),
            details: std::ptr::null_mut(),
        }
    }

    pub fn from_error(err: &CoreError) -> Self {
        let (code, msg, details): (i32, String, String) = match err {
            CoreError::Config(s) => (1, "Configuration error".into(), s.clone()),
            CoreError::Io(e) => (2, "I/O error".into(), e.to_string()),
            CoreError::Database(s) => (3, "Database error".into(), s.clone()),
            CoreError::Network(s) => (4, "Network error".into(), s.clone()),
            CoreError::Auth(s) => (5, "Authentication error".into(), s.clone()),
            CoreError::Serialization(e) => (6, "Serialization error".into(), e.to_string()),
            CoreError::Validation(s) => (7, "Validation error".into(), s.clone()),
            CoreError::NotFound(s) => (8, "Not found".into(), s.clone()),
            CoreError::Conflict(s) => (9, "Conflict".into(), s.clone()),
            CoreError::Sync(s) => (10, "Sync error".into(), s.clone()),
            CoreError::Internal(s) => (11, "Internal error".into(), s.clone()),
            CoreError::Forbidden(s) => (12, "Forbidden".into(), s.clone()),
            CoreError::Timeout(s) => (13, "Timeout".into(), s.clone()),
            CoreError::Unavailable(s) => (14, "Unavailable".into(), s.clone()),
        };

        Self {
            code,
            message: CString::new(msg).unwrap_or_default().into_raw(),
            details: CString::new(details).unwrap_or_default().into_raw(),
        }
    }
}
