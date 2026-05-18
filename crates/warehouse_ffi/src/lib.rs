#![allow(
    unsafe_op_in_unsafe_fn,
    clippy::missing_safety_doc,
    clippy::not_unsafe_ptr_arg_deref
)]

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use tokio::runtime::Builder as RuntimeBuilder;
use warehouse_core::auth::token_provider::FfiTokenProvider;
use warehouse_core::config::CoreConfig;
use warehouse_core::facade::CoreHandle;
use warehouse_core::sync::SyncMode;

mod error;
pub use error::CoreErrorDto;

pub struct CoreHandleWrapper {
    runtime: tokio::runtime::Runtime,
    handle: CoreHandle,
    ffi_provider: FfiTokenProvider,
}

impl CoreHandleWrapper {
    fn new(config: CoreConfig) -> Result<Self, warehouse_core::CoreError> {
        let runtime = RuntimeBuilder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| warehouse_core::CoreError::Config(e.to_string()))?;
        let mut handle = runtime.block_on(CoreHandle::open(config))?;
        let ffi_provider = FfiTokenProvider::new();
        handle.set_token_provider(Box::new(ffi_provider.clone()));
        Ok(Self {
            runtime,
            handle,
            ffi_provider,
        })
    }
}

fn c_str_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let cstr = unsafe { CStr::from_ptr(ptr) };
    cstr.to_str().ok().map(|s| s.to_owned())
}

fn string_to_c_str(s: String) -> *mut c_char {
    CString::new(s).unwrap_or_default().into_raw()
}

unsafe fn verify_handle<'a>(
    handle: *mut CoreHandleWrapper,
) -> Result<&'a mut CoreHandleWrapper, CoreErrorDto> {
    if handle.is_null() {
        return Err(CoreErrorDto::from_error(
            &warehouse_core::CoreError::Config("handle is null".into()),
        ));
    }
    unsafe { Ok(&mut *handle) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_init() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_ansi(false)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_open(
    db_path: *const c_char,
    server_url: *const c_char,
    out_handle: *mut *mut CoreHandleWrapper,
) -> CoreErrorDto {
    if out_handle.is_null() {
        return CoreErrorDto::from_error(&warehouse_core::CoreError::Config(
            "out_handle is null".into(),
        ));
    }
    let path = match c_str_to_string(db_path) {
        Some(p) => p,
        None => {
            return CoreErrorDto::from_error(&warehouse_core::CoreError::Config(
                "db_path is null".into(),
            ));
        }
    };

    let server_url_str = c_str_to_string(server_url);
    let mut config = warehouse_core::CoreConfig::for_testing(std::path::PathBuf::from(&path));
    if let Some(url) = server_url_str {
        config.server_base_url = url;
    }

    match CoreHandleWrapper::new(config) {
        Ok(wrapper) => {
            let boxed = Box::new(wrapper);
            *out_handle = Box::into_raw(boxed);
            CoreErrorDto::success()
        }
        Err(e) => CoreErrorDto::from_error(&e),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_close(handle: *mut CoreHandleWrapper) -> CoreErrorDto {
    if handle.is_null() {
        return CoreErrorDto::from_error(&warehouse_core::CoreError::Config(
            "handle is null".into(),
        ));
    }
    let mut wrapper = Box::from_raw(handle);
    wrapper.handle.close();
    drop(wrapper);
    CoreErrorDto::success()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_set_user_token(
    handle: *mut CoreHandleWrapper,
    token: *const c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let token_str = c_str_to_string(token);
    let uuid = match token_str.and_then(|s| uuid::Uuid::parse_str(&s).ok()) {
        Some(u) => u,
        None => {
            return CoreErrorDto::from_error(&warehouse_core::CoreError::Validation(
                "Invalid user token UUID".into(),
            ));
        }
    };
    wrapper.ffi_provider.set_user_token(uuid);
    wrapper.handle.close();
    CoreErrorDto::success()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_set_device_token(
    handle: *mut CoreHandleWrapper,
    token: *const c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let token_str = c_str_to_string(token);
    let uuid = match token_str.and_then(|s| uuid::Uuid::parse_str(&s).ok()) {
        Some(u) => u,
        None => {
            return CoreErrorDto::from_error(&warehouse_core::CoreError::Validation(
                "Invalid device token UUID".into(),
            ));
        }
    };
    wrapper.ffi_provider.set_device_token(uuid);
    wrapper.handle.close();
    CoreErrorDto::success()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_refresh_identity(
    handle: *mut CoreHandleWrapper,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let CoreHandleWrapper {
        runtime: _,
        handle: _core,
        ..
    } = wrapper;
    match wrapper.runtime.block_on(wrapper.handle.refresh_identity()) {
        Ok(_) => CoreErrorDto::success(),
        Err(e) => CoreErrorDto::from_error(&e),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_set_active_site(
    handle: *mut CoreHandleWrapper,
    site_id: i32,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let CoreHandleWrapper {
        runtime: _,
        handle: _core,
        ..
    } = wrapper;
    match wrapper
        .runtime
        .block_on(wrapper.handle.set_active_site(site_id))
    {
        Ok(_) => CoreErrorDto::success(),
        Err(e) => CoreErrorDto::from_error(&e),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_bootstrap(
    handle: *mut CoreHandleWrapper,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let CoreHandleWrapper {
        runtime: _,
        handle: _core,
        ..
    } = wrapper;
    match wrapper.runtime.block_on(wrapper.handle.bootstrap()) {
        Ok(r) => {
            let json = serde_json::json!({
                "success": r.success,
                "families_synced": r.families_synced,
                "errors": r.errors,
            })
            .to_string();
            if !out_json.is_null() {
                *out_json = string_to_c_str(json);
            }
            CoreErrorDto::success()
        }
        Err(e) => CoreErrorDto::from_error(&e),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_pull_once(
    handle: *mut CoreHandleWrapper,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let CoreHandleWrapper {
        runtime: _,
        handle: _core,
        ..
    } = wrapper;
    let summary = wrapper.runtime.block_on(wrapper.handle.pull_once());
    let families_succeeded = summary.families.iter().filter(|f| f.success).count();
    let families_failed = summary.families.iter().filter(|f| !f.success).count();
    let errors_list: Vec<&str> = summary.errors();
    let json = serde_json::json!({
        "families_attempted": summary.families.len(),
        "families_succeeded": families_succeeded,
        "families_failed": families_failed,
        "errors": errors_list,
        "total_items": summary.total_items,
        "is_complete": summary.is_complete,
    })
    .to_string();
    if !out_json.is_null() {
        *out_json = string_to_c_str(json);
    }
    CoreErrorDto::success()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_get_sync_status(
    handle: *mut CoreHandleWrapper,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let CoreHandleWrapper {
        runtime: _,
        handle: _core,
        ..
    } = wrapper;
    match wrapper.runtime.block_on(wrapper.handle.get_sync_status()) {
        Ok(s) => {
            let json = serde_json::json!({
                "is_authenticated": s.is_authenticated,
                "active_site_id": s.active_site_id,
                "outbox_pending": s.outbox_pending,
                "cursor_count": s.cursor_count,
            })
            .to_string();
            if !out_json.is_null() {
                *out_json = string_to_c_str(json);
            }
            CoreErrorDto::success()
        }
        Err(e) => CoreErrorDto::from_error(&e),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_get_auth_context(
    handle: *mut CoreHandleWrapper,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    match wrapper.handle.get_auth_context() {
        Ok(ctx) => {
            let json = serde_json::json!({
                "user_id": ctx.user_id.to_string(),
                "user_name": ctx.user_name,
                "user_email": ctx.user_email,
                "role": ctx.role,
                "is_root": ctx.is_root,
                "available_sites": ctx.available_sites,
                "device_id": ctx.device_id.to_string(),
                "device_registered": ctx.device_registered,
            })
            .to_string();
            if !out_json.is_null() {
                *out_json = string_to_c_str(json);
            }
            CoreErrorDto::success()
        }
        Err(e) => CoreErrorDto::from_error(&e),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_search_catalog(
    handle: *mut CoreHandleWrapper,
    query: *const c_char,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let q = c_str_to_string(query).unwrap_or_default();
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let CoreHandleWrapper {
        runtime: _,
        handle: _core,
        ..
    } = wrapper;
    match wrapper.runtime.block_on(wrapper.handle.search_items(&q)) {
        Ok(items) => {
            let json = serde_json::to_string(&items).unwrap_or_else(|_| "[]".into());
            if !out_json.is_null() {
                *out_json = string_to_c_str(json);
            }
            CoreErrorDto::success()
        }
        Err(e) => CoreErrorDto::from_error(&e),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_list_balances(
    handle: *mut CoreHandleWrapper,
    site_id: i32,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let CoreHandleWrapper {
        runtime: _,
        handle: _core,
        ..
    } = wrapper;
    match wrapper
        .runtime
        .block_on(wrapper.handle.list_balances(site_id))
    {
        Ok(items) => {
            let json = serde_json::to_string(&items).unwrap_or_else(|_| "[]".into());
            if !out_json.is_null() {
                *out_json = string_to_c_str(json);
            }
            CoreErrorDto::success()
        }
        Err(e) => CoreErrorDto::from_error(&e),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_list_operations(
    handle: *mut CoreHandleWrapper,
    site_id: i32,
    page: u32,
    page_size: u32,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let CoreHandleWrapper {
        runtime: _,
        handle: _core,
        ..
    } = wrapper;
    match wrapper
        .runtime
        .block_on(wrapper.handle.list_operations(site_id, page, page_size))
    {
        Ok(ops) => {
            let json = serde_json::to_string(&ops).unwrap_or_else(|_| "{}".into());
            if !out_json.is_null() {
                *out_json = string_to_c_str(json);
            }
            CoreErrorDto::success()
        }
        Err(e) => CoreErrorDto::from_error(&e),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_list_temp_items(
    handle: *mut CoreHandleWrapper,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let CoreHandleWrapper {
        runtime: _,
        handle: _core,
        ..
    } = wrapper;
    match wrapper
        .runtime
        .block_on(wrapper.handle.list_temporary_items())
    {
        Ok(items) => {
            let json = serde_json::to_string(&items).unwrap_or_else(|_| "[]".into());
            if !out_json.is_null() {
                *out_json = string_to_c_str(json);
            }
            CoreErrorDto::success()
        }
        Err(e) => CoreErrorDto::from_error(&e),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_list_pending_acceptance(
    handle: *mut CoreHandleWrapper,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let CoreHandleWrapper {
        runtime: _,
        handle: _core,
        ..
    } = wrapper;
    match wrapper
        .runtime
        .block_on(wrapper.handle.list_pending_acceptance())
    {
        Ok(items) => {
            let json = serde_json::to_string(&items).unwrap_or_else(|_| "[]".into());
            if !out_json.is_null() {
                *out_json = string_to_c_str(json);
            }
            CoreErrorDto::success()
        }
        Err(e) => CoreErrorDto::from_error(&e),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_list_lost_assets(
    handle: *mut CoreHandleWrapper,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let CoreHandleWrapper {
        runtime: _,
        handle: _core,
        ..
    } = wrapper;
    match wrapper.runtime.block_on(wrapper.handle.list_lost_assets()) {
        Ok(items) => {
            let json = serde_json::to_string(&items).unwrap_or_else(|_| "[]".into());
            if !out_json.is_null() {
                *out_json = string_to_c_str(json);
            }
            CoreErrorDto::success()
        }
        Err(e) => CoreErrorDto::from_error(&e),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_list_issued_assets(
    handle: *mut CoreHandleWrapper,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let CoreHandleWrapper {
        runtime: _,
        handle: _core,
        ..
    } = wrapper;
    match wrapper
        .runtime
        .block_on(wrapper.handle.list_issued_assets())
    {
        Ok(items) => {
            let json = serde_json::to_string(&items).unwrap_or_else(|_| "[]".into());
            if !out_json.is_null() {
                *out_json = string_to_c_str(json);
            }
            CoreErrorDto::success()
        }
        Err(e) => CoreErrorDto::from_error(&e),
    }
}

// ── Drafts ──────────────────────────────────────────────────────

macro_rules! block_on_void {
    ($wrapper:expr, $h:ident.$method:ident($($args:tt)*)) => {{
        let w: &mut CoreHandleWrapper = $wrapper;
        let $h = &mut w.handle;
        match w.runtime.block_on($h.$method($($args)*)) {
            Ok(_) => CoreErrorDto::success(),
            Err(e) => CoreErrorDto::from_error(&e),
        }
    }};
    ($wrapper:expr, $call:expr) => {{
        let w: &mut CoreHandleWrapper = $wrapper;
        match w.runtime.block_on($call) {
            Ok(_) => CoreErrorDto::success(),
            Err(e) => CoreErrorDto::from_error(&e),
        }
    }};
}

macro_rules! block_on_json {
    ($wrapper:expr, $h:ident.$method:ident($($args:tt)*), $out_json:expr) => {{
        let w: &mut CoreHandleWrapper = $wrapper;
        let $h = &mut w.handle;
        match w.runtime.block_on($h.$method($($args)*)) {
            Ok(r) => {
                let json = serde_json::to_string(&r).unwrap_or_else(|_| "null".into());
                if !$out_json.is_null() {
                    *$out_json = string_to_c_str(json);
                }
                CoreErrorDto::success()
            }
            Err(e) => CoreErrorDto::from_error(&e),
        }
    }};
    ($wrapper:expr, $call:expr, $out_json:expr) => {{
        let w: &mut CoreHandleWrapper = $wrapper;
        match w.runtime.block_on($call) {
            Ok(r) => {
                let json = serde_json::to_string(&r).unwrap_or_else(|_| "null".into());
                if !$out_json.is_null() {
                    *$out_json = string_to_c_str(json);
                }
                CoreErrorDto::success()
            }
            Err(e) => CoreErrorDto::from_error(&e),
        }
    }};
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_create_draft(
    handle: *mut CoreHandleWrapper,
    operation_type: *const c_char,
    site_id: i32,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let op_type_str = c_str_to_string(operation_type).unwrap_or_default();
    use warehouse_core::domain::operation::OperationType;
    let op_type = serde_json::from_str::<OperationType>(&format!("\"{op_type_str}\""))
        .unwrap_or(OperationType::Receive);
    let site_id_opt = if site_id > 0 { Some(site_id) } else { None };
    block_on_json!(wrapper, h.create_draft(op_type, site_id_opt), out_json)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_get_draft(
    handle: *mut CoreHandleWrapper,
    draft_id: *const c_char,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let id = c_str_to_string(draft_id).unwrap_or_default();
    block_on_json!(wrapper, h.get_draft(&id), out_json)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_list_drafts(
    handle: *mut CoreHandleWrapper,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    block_on_json!(wrapper, h.list_drafts(), out_json)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_delete_draft(
    handle: *mut CoreHandleWrapper,
    draft_id: *const c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let id = c_str_to_string(draft_id).unwrap_or_default();
    block_on_void!(wrapper, h.delete_draft(&id))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_clone_draft(
    handle: *mut CoreHandleWrapper,
    draft_id: *const c_char,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let id = c_str_to_string(draft_id).unwrap_or_default();
    block_on_json!(wrapper, h.clone_draft(&id), out_json)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_validate_draft(
    handle: *mut CoreHandleWrapper,
    draft_id: *const c_char,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let id = c_str_to_string(draft_id).unwrap_or_default();
    let CoreHandleWrapper {
        runtime: _,
        handle: _core,
        ..
    } = wrapper;
    let result = wrapper.runtime.block_on(wrapper.handle.validate_draft(&id));
    match result {
        Ok(()) => {
            let json = serde_json::json!({"valid": true}).to_string();
            if !out_json.is_null() {
                *out_json = string_to_c_str(json);
            }
            CoreErrorDto::success()
        }
        Err(errors) => {
            let json = serde_json::json!({"valid": false, "errors": errors}).to_string();
            if !out_json.is_null() {
                *out_json = string_to_c_str(json);
            }
            CoreErrorDto::success()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_add_draft_line(
    handle: *mut CoreHandleWrapper,
    draft_id: *const c_char,
    item_id: i32,
    qty: *const c_char,
    batch: *const c_char,
    comment: *const c_char,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let id = c_str_to_string(draft_id).unwrap_or_default();
    let qty_str = c_str_to_string(qty).unwrap_or_else(|| "1".into());
    let qty_val: serde_json::Value =
        serde_json::from_str(&qty_str).unwrap_or(serde_json::Value::String(qty_str));
    let batch_opt = c_str_to_string(batch);
    let comment_opt = c_str_to_string(comment);
    block_on_json!(
        wrapper,
        core.add_draft_item_line(&id, item_id, qty_val, batch_opt, comment_opt),
        out_json
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_add_draft_temp_line(
    handle: *mut CoreHandleWrapper,
    draft_id: *const c_char,
    name: *const c_char,
    unit_id: i32,
    qty: *const c_char,
    batch: *const c_char,
    comment: *const c_char,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let id = c_str_to_string(draft_id).unwrap_or_default();
    let name_str = c_str_to_string(name).unwrap_or_default();
    let qty_str = c_str_to_string(qty).unwrap_or_else(|| "1".into());
    let qty_val: serde_json::Value =
        serde_json::from_str(&qty_str).unwrap_or(serde_json::Value::String(qty_str));
    let batch_opt = c_str_to_string(batch);
    let comment_opt = c_str_to_string(comment);
    let temp_item = warehouse_core::domain::operation::TemporaryItemInlineCreate {
        name: name_str,
        unit_id,
        sku: None,
        category_id: None,
        description: None,
    };
    block_on_json!(
        wrapper,
        core.add_draft_temp_item_line(&id, temp_item, qty_val, batch_opt, comment_opt),
        out_json
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_delete_draft_line(
    handle: *mut CoreHandleWrapper,
    draft_id: *const c_char,
    line_id: *const c_char,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let did = c_str_to_string(draft_id).unwrap_or_default();
    let lid = c_str_to_string(line_id).unwrap_or_default();
    block_on_json!(wrapper, h.delete_draft_line(&did, &lid), out_json)
}

// ── Outbox ──────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_queue_submit(
    handle: *mut CoreHandleWrapper,
    draft_id: *const c_char,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let id = c_str_to_string(draft_id).unwrap_or_default();
    let CoreHandleWrapper {
        runtime: _,
        handle: _core,
        ..
    } = wrapper;
    match wrapper
        .runtime
        .block_on(wrapper.handle.queue_draft_submit(&id))
    {
        Ok(event_uuid) => {
            let json = serde_json::json!({"event_uuid": event_uuid}).to_string();
            if !out_json.is_null() {
                *out_json = string_to_c_str(json);
            }
            CoreErrorDto::success()
        }
        Err(e) => CoreErrorDto::from_error(&e),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_list_outbox(
    handle: *mut CoreHandleWrapper,
    site_id: i32,
    status: *const c_char,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let s = c_str_to_string(status);
    let sid = if site_id > 0 { Some(site_id) } else { None };
    block_on_json!(wrapper, h.list_outbox_events(sid, s.as_deref()), out_json)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_get_outbox(
    handle: *mut CoreHandleWrapper,
    event_uuid: *const c_char,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let uid = c_str_to_string(event_uuid).unwrap_or_default();
    block_on_json!(wrapper, h.get_outbox_event(&uid), out_json)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_retry_outbox(
    handle: *mut CoreHandleWrapper,
    event_uuid: *const c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let uid = c_str_to_string(event_uuid).unwrap_or_default();
    block_on_void!(wrapper, h.retry_outbox_event(&uid))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_cancel_outbox(
    handle: *mut CoreHandleWrapper,
    event_uuid: *const c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let uid = c_str_to_string(event_uuid).unwrap_or_default();
    block_on_void!(wrapper, h.cancel_outbox_event(&uid))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_send_outbox(
    handle: *mut CoreHandleWrapper,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let CoreHandleWrapper {
        runtime: _,
        handle: _core,
        ..
    } = wrapper;
    match wrapper.runtime.block_on(wrapper.handle.send_outbox()) {
        Ok(sr) => {
            let json = serde_json::json!({
                "accepted": sr.accepted,
                "failed": sr.failed,
                "conflicts": sr.conflicts,
            })
            .to_string();
            if !out_json.is_null() {
                *out_json = string_to_c_str(json);
            }
            CoreErrorDto::success()
        }
        Err(e) => CoreErrorDto::from_error(&e),
    }
}

// ── Sync Engine ─────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_sync_once(
    handle: *mut CoreHandleWrapper,
    mode: i32,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let sync_mode = match mode {
        0 => SyncMode::Bootstrap,
        1 => SyncMode::PullOnly,
        2 => SyncMode::PushOnly,
        3 => SyncMode::PushThenPull,
        _ => SyncMode::Full,
    };
    let CoreHandleWrapper {
        runtime: _,
        handle: _core,
        ..
    } = wrapper;
    let result = wrapper
        .runtime
        .block_on(wrapper.handle.sync_once(sync_mode));
    let json = serde_json::json!({
        "success": result.success,
        "mode": format!("{:?}", result.mode),
        "push_accepted": result.push_accepted,
        "push_failed": result.push_failed,
        "push_conflicts": result.push_conflicts,
        "pull_items": result.pull_items,
        "pull_errors": result.pull_errors,
        "bootstrap_ok": result.bootstrap_ok,
        "error": result.error,
        "conflict_count": result.conflicts.total,
    })
    .to_string();
    if !out_json.is_null() {
        *out_json = string_to_c_str(json);
    }
    CoreErrorDto::success()
}

// ── Temporary Items ─────────────────────────────────────────────

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_approve_temp_item(
    handle: *mut CoreHandleWrapper,
    temp_id: i32,
    name: *const c_char,
    sku: *const c_char,
    category_id: i32,
    unit_id: i32,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let req = warehouse_core::domain::temporary_items::ApproveAsItemRequest {
        name: c_str_to_string(name).unwrap_or_default(),
        sku: c_str_to_string(sku),
        category_id: if category_id > 0 {
            Some(category_id)
        } else {
            None
        },
        unit_id,
    };
    let CoreHandleWrapper {
        runtime: _,
        handle: _core,
        ..
    } = wrapper;
    match wrapper
        .runtime
        .block_on(wrapper.handle.approve_temp_item(temp_id, &req))
    {
        Ok(r) => {
            let json = serde_json::to_string(&r).unwrap_or_else(|_| "null".into());
            if !out_json.is_null() {
                *out_json = string_to_c_str(json);
            }
            CoreErrorDto::success()
        }
        Err(e) => CoreErrorDto::from_error(&e),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_merge_temp_item(
    handle: *mut CoreHandleWrapper,
    temp_id: i32,
    target_item_id: i32,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let req = warehouse_core::domain::temporary_items::MergeToItemRequest { target_item_id };
    let CoreHandleWrapper {
        runtime: _,
        handle: _core,
        ..
    } = wrapper;
    match wrapper
        .runtime
        .block_on(wrapper.handle.merge_temp_item(temp_id, &req))
    {
        Ok(r) => {
            let json = serde_json::to_string(&r).unwrap_or_else(|_| "null".into());
            if !out_json.is_null() {
                *out_json = string_to_c_str(json);
            }
            CoreErrorDto::success()
        }
        Err(e) => CoreErrorDto::from_error(&e),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_delete_temp_item(
    handle: *mut CoreHandleWrapper,
    temp_id: i32,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    block_on_void!(wrapper, h.delete_temp_item(temp_id))
}

// ── Documents ───────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_generate_document(
    handle: *mut CoreHandleWrapper,
    operation_id: *const c_char,
    document_type: *const c_char,
    template_name: *const c_char,
    auto_finalize: bool,
    language: *const c_char,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let op_id_str = c_str_to_string(operation_id).unwrap_or_default();
    let op_id = uuid::Uuid::parse_str(&op_id_str).unwrap_or_default();
    use warehouse_core::domain::documents::DocumentType;
    let doc_type = c_str_to_string(document_type)
        .and_then(|s| serde_json::from_str::<DocumentType>(&format!("\"{s}\"")).ok())
        .unwrap_or(DocumentType::Waybill);
    let req = warehouse_core::domain::documents::DocumentGenerateRequest {
        operation_id: op_id,
        document_type: doc_type,
        template_name: c_str_to_string(template_name),
        auto_finalize,
        language: c_str_to_string(language).unwrap_or_else(|| "ru".into()),
    };
    let CoreHandleWrapper {
        runtime: _,
        handle: _core,
        ..
    } = wrapper;
    match wrapper
        .runtime
        .block_on(wrapper.handle.generate_document(&req))
    {
        Ok(doc) => {
            let json = serde_json::to_string(&doc).unwrap_or_else(|_| "null".into());
            if !out_json.is_null() {
                *out_json = string_to_c_str(json);
            }
            CoreErrorDto::success()
        }
        Err(e) => CoreErrorDto::from_error(&e),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_render_document(
    handle: *mut CoreHandleWrapper,
    document_id: *const c_char,
    out_bytes: *mut *mut u8,
    out_len: *mut usize,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let doc_id_str = c_str_to_string(document_id).unwrap_or_default();
    let doc_id = uuid::Uuid::parse_str(&doc_id_str).unwrap_or_default();
    let CoreHandleWrapper {
        runtime: _,
        handle: _core,
        ..
    } = wrapper;
    match wrapper
        .runtime
        .block_on(wrapper.handle.render_document(doc_id))
    {
        Ok(bytes) => {
            let len = bytes.len();
            let boxed = bytes.into_boxed_slice();
            let ptr = Box::into_raw(boxed) as *mut u8;
            if !out_bytes.is_null() {
                *out_bytes = ptr;
            }
            if !out_len.is_null() {
                *out_len = len;
            }
            CoreErrorDto::success()
        }
        Err(e) => CoreErrorDto::from_error(&e),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_free_bytes(ptr: *mut u8, len: usize) {
    if !ptr.is_null() {
        let _ = unsafe { Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, len)) };
    }
}

// ── Assets ──────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_resolve_lost_asset(
    handle: *mut CoreHandleWrapper,
    operation_line_id: i64,
    action: *const c_char,
    qty: *const c_char,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    use warehouse_core::domain::assets::LostAssetResolveAction;
    let action_str = c_str_to_string(action).unwrap_or_default();
    let resolve_action = match action_str.as_str() {
        "found_to_destination" => LostAssetResolveAction::FoundToDestination,
        "return_to_source" => LostAssetResolveAction::ReturnToSource,
        _ => LostAssetResolveAction::WriteOff,
    };
    let qty_str = c_str_to_string(qty).unwrap_or_else(|| "1".into());
    let resolved_qty: serde_json::Value =
        serde_json::from_str(&qty_str).unwrap_or(serde_json::Value::String(qty_str));
    let req = warehouse_core::domain::assets::LostAssetResolveRequest {
        action: resolve_action,
        resolved_qty,
    };
    let CoreHandleWrapper {
        runtime: _,
        handle: _core,
        ..
    } = wrapper;
    match wrapper
        .runtime
        .block_on(wrapper.handle.resolve_lost_asset(operation_line_id, &req))
    {
        Ok(r) => {
            let json = serde_json::to_string(&r).unwrap_or_else(|_| "null".into());
            if !out_json.is_null() {
                *out_json = string_to_c_str(json);
            }
            CoreErrorDto::success()
        }
        Err(e) => CoreErrorDto::from_error(&e),
    }
}

// ── Conflicts / Extra Read ──────────────────────────────────────

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_list_conflicts(
    handle: *mut CoreHandleWrapper,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let summary = wrapper.handle.get_last_conflicts();
    let json = serde_json::to_string(&summary).unwrap_or_else(|_| "{}".into());
    if !out_json.is_null() {
        *out_json = string_to_c_str(json);
    }
    CoreErrorDto::success()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_list_available_sites(
    handle: *mut CoreHandleWrapper,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    match wrapper.handle.list_available_sites() {
        Ok(sites) => {
            let json = serde_json::to_string(&sites).unwrap_or_else(|_| "[]".into());
            if !out_json.is_null() {
                *out_json = string_to_c_str(json);
            }
            CoreErrorDto::success()
        }
        Err(e) => CoreErrorDto::from_error(&e),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_get_active_site(
    handle: *mut CoreHandleWrapper,
    out_site_id: *mut i32,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    match wrapper.handle.get_active_site() {
        Ok(site_id) => {
            if let Some(sid) = site_id {
                if !out_site_id.is_null() {
                    *out_site_id = sid;
                }
            }
            CoreErrorDto::success()
        }
        Err(e) => CoreErrorDto::from_error(&e),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_clear_active_site(
    handle: *mut CoreHandleWrapper,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    block_on_void!(wrapper, h.clear_active_site())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_ready_remote(
    handle: *mut CoreHandleWrapper,
    out_json: *mut *mut c_char,
) -> CoreErrorDto {
    let wrapper = match verify_handle(handle) {
        Ok(w) => w,
        Err(e) => return e,
    };
    let CoreHandleWrapper {
        runtime: _,
        handle: _core,
        ..
    } = wrapper;
    match wrapper.runtime.block_on(wrapper.handle.ready_remote()) {
        Ok(status) => {
            let json = serde_json::json!({"status": status}).to_string();
            if !out_json.is_null() {
                *out_json = string_to_c_str(json);
            }
            CoreErrorDto::success()
        }
        Err(e) => CoreErrorDto::from_error(&e),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warehouse_free_error(dto: *mut CoreErrorDto) {
    if !dto.is_null() {
        let boxed = Box::from_raw(dto);
        drop(boxed);
    }
}
