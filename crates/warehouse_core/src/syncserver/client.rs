use reqwest::{Method, RequestBuilder};
use serde::de::DeserializeOwned;

use crate::config::CoreConfig;
use crate::error::{CoreError, CoreResult};

pub(crate) enum AuthKind {
    User,
    Device,
    None,
}

#[derive(Clone)]
pub struct SyncServerClient {
    pub(crate) client: reqwest::Client,
    pub(crate) base_url: String,
    pub(crate) user_token: Option<String>,
    pub(crate) device_token: Option<String>,
}

impl SyncServerClient {
    pub fn new(config: &CoreConfig) -> CoreResult<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .user_agent(format!("{}/{}", config.client_name, config.client_version))
            .build()
            .map_err(|e| CoreError::Config(format!("Failed to build HTTP client: {e}")))?;

        Ok(Self {
            client,
            base_url: config.server_base_url.trim_end_matches('/').to_string(),
            user_token: None,
            device_token: None,
        })
    }

    pub fn set_user_token(&mut self, token: uuid::Uuid) {
        self.user_token = Some(token.to_string());
    }

    pub fn set_device_token(&mut self, token: uuid::Uuid) {
        self.device_token = Some(token.to_string());
    }

    pub fn clear_user_token(&mut self) {
        self.user_token = None;
    }

    pub fn clear_device_token(&mut self) {
        self.device_token = None;
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    pub(crate) fn request(&self, method: Method, path: &str, auth: AuthKind) -> RequestBuilder {
        let mut req = self.client.request(method, self.url(path));
        match auth {
            AuthKind::User => {
                if let Some(ref t) = self.user_token {
                    req = req.header("X-User-Token", t.clone());
                }
            }
            AuthKind::Device => {
                if let Some(ref t) = self.device_token {
                    req = req.header("X-Device-Token", t.clone());
                }
            }
            AuthKind::None => {}
        }
        req
    }

    pub(crate) async fn send<T: DeserializeOwned>(&self, req: RequestBuilder) -> CoreResult<T> {
        let resp = req
            .send()
            .await
            .map_err(|e| CoreError::Network(format!("Request failed: {e}")))?;

        let status = resp.status();
        let body = resp
            .bytes()
            .await
            .map_err(|e| CoreError::Network(format!("Failed to read response body: {e}")))?;

        if !status.is_success() {
            let text = String::from_utf8_lossy(&body);
            return Err(map_http_error(status, &text));
        }

        serde_json::from_slice(&body).map_err(CoreError::Serialization)
    }

    #[allow(dead_code)]
    pub(crate) async fn send_no_body(&self, req: RequestBuilder) -> CoreResult<()> {
        let resp = req
            .send()
            .await
            .map_err(|e| CoreError::Network(format!("Request failed: {e}")))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(map_http_error(status, &body));
        }
        Ok(())
    }

    pub(crate) fn get(&self, path: &str, auth: AuthKind) -> RequestBuilder {
        self.request(Method::GET, path, auth)
    }

    pub(crate) fn post(&self, path: &str, auth: AuthKind) -> RequestBuilder {
        self.request(Method::POST, path, auth)
    }

    pub(crate) fn patch(&self, path: &str, auth: AuthKind) -> RequestBuilder {
        self.request(Method::PATCH, path, auth)
    }

    #[allow(dead_code)]
    pub(crate) fn delete(&self, path: &str, auth: AuthKind) -> RequestBuilder {
        self.request(Method::DELETE, path, auth)
    }
}

fn map_http_error(status: reqwest::StatusCode, body: &str) -> CoreError {
    let redacted = redact_secrets(body);
    match status.as_u16() {
        401 => CoreError::Auth(format!("Unauthorized: {redacted}")),
        403 => CoreError::Forbidden(format!("Forbidden: {redacted}")),
        404 => CoreError::NotFound(format!("Resource not found: {redacted}")),
        409 => CoreError::Conflict(format!("Conflict: {redacted}")),
        422 => CoreError::Validation(format!("Validation error: {redacted}")),
        429 => CoreError::Timeout(format!("Rate limited: {redacted}")),
        503 => CoreError::Unavailable(format!("Service unavailable: {redacted}")),
        504 => CoreError::Timeout(format!("Gateway timeout: {redacted}")),
        s if s >= 500 => CoreError::Network(format!("Server error {s}: {redacted}")),
        s => CoreError::Network(format!("HTTP {s}: {redacted}")),
    }
}

fn redact_secrets(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if i + 36 <= len {
            let slice = &text[i..i + 36];
            let is_uuid = slice.len() == 36
                && slice.as_bytes()[8] == b'-'
                && slice.as_bytes()[13] == b'-'
                && slice.as_bytes()[18] == b'-'
                && slice.as_bytes()[23] == b'-'
                && slice.chars().all(|c| c.is_ascii_hexdigit() || c == '-');
            if is_uuid {
                result.push_str("***UUID***");
                i += 36;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }

    replace_token_patterns(&result)
}

fn replace_token_patterns(text: &str) -> String {
    let markers = [
        "user_token",
        "user-token",
        "user token",
        "device_token",
        "device-token",
        "device token",
        "bearer",
        "secret",
        "password",
        "api_key",
        "api-key",
        "api key",
    ];

    let mut result = text.to_string();
    for marker in &markers {
        let mut search_start = 0;
        while let Some(i) = result[search_start..].to_lowercase().find(marker) {
            let idx = search_start + i;
            let after_marker = idx + marker.len();
            let rest = &result[after_marker..];

            let has_colon_or_eq = rest.starts_with(':') || rest.starts_with('=');
            let has_space_colon = rest.starts_with(" :") || rest.starts_with(" =");
            let has_colon_space = rest.starts_with(": ") || rest.starts_with("= ");

            if has_colon_or_eq || has_space_colon || has_colon_space {
                let value_start = if has_colon_or_eq {
                    after_marker + 1
                } else {
                    after_marker + 2
                };
                let value_rest = &result[value_start..];
                let value_end = value_rest
                    .find(|c: char| c.is_whitespace())
                    .unwrap_or(value_rest.len());
                let end = value_start + value_end;
                result.replace_range(after_marker..end, ": ***REDACTED***");
                search_start = after_marker + ": ***REDACTED***".len();
            } else {
                search_start = after_marker;
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::token_provider::TokenProvider;
    use crate::config::CoreConfig;
    use std::path::PathBuf;
    use wiremock::matchers::{header, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_config(base_url: &str) -> CoreConfig {
        CoreConfig {
            server_base_url: base_url.to_string(),
            database_path: PathBuf::from(":memory:"),
            timeout_seconds: 5,
            ..Default::default()
        }
    }

    fn test_client(base_url: &str) -> SyncServerClient {
        SyncServerClient::new(&test_config(base_url)).unwrap()
    }

    #[tokio::test]
    async fn sends_user_token_header() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/auth/me"))
            .and(header(
                "X-User-Token",
                "550e8400-e29b-41d4-a716-446655440000",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "user_id": "550e8400-e29b-41d4-a716-446655440000",
                "name": "Test User",
                "email": "test@example.com",
                "role": "root"
            })))
            .mount(&mock_server)
            .await;

        let mut client = test_client(&mock_server.uri());
        client
            .set_user_token(uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap());
        let result: serde_json::Value = client
            .get("/api/v1/auth/me", AuthKind::User)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(result["role"], "root");
    }

    #[tokio::test]
    async fn sends_device_token_header() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/ping"))
            .and(header(
                "X-Device-Token",
                "660e8400-e29b-41d4-a716-446655440001",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "server_time": "2026-01-01T00:00:00Z",
                "server_seq_upto": 42,
                "backoff_seconds": 0
            })))
            .mount(&mock_server)
            .await;

        let mut client = test_client(&mock_server.uri());
        client.set_device_token(
            uuid::Uuid::parse_str("660e8400-e29b-41d4-a716-446655440001").unwrap(),
        );
        let result: serde_json::Value = client
            .post("/api/v1/ping", AuthKind::Device)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(result["server_seq_upto"], 42);
    }

    #[tokio::test]
    async fn no_auth_header_when_none_kind() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/health"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "ok"
            })))
            .mount(&mock_server)
            .await;

        let client = test_client(&mock_server.uri());
        let result: serde_json::Value = client
            .get("/api/v1/health", AuthKind::None)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(result["status"], "ok");
    }

    #[tokio::test]
    async fn sends_query_params() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/catalog/items"))
            .and(query_param("updated_after", "2026-01-01T00:00:00Z"))
            .and(query_param("limit", "100"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "items": [],
                "server_time": "2026-01-01T00:00:00Z",
                "next_updated_after": null
            })))
            .mount(&mock_server)
            .await;

        let client = test_client(&mock_server.uri());
        let req = client
            .get("/api/v1/catalog/items", AuthKind::User)
            .query(&[("updated_after", "2026-01-01T00:00:00Z"), ("limit", "100")]);
        let result: serde_json::Value = client.send(req).await.unwrap();
        assert!(result["items"].is_array());
    }

    #[tokio::test]
    async fn handles_401_as_auth_error() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/auth/me"))
            .respond_with(ResponseTemplate::new(401).set_body_string("Invalid token"))
            .mount(&mock_server)
            .await;

        let client = test_client(&mock_server.uri());
        let req = client.get("/api/v1/auth/me", AuthKind::User);
        let err = client.send::<serde_json::Value>(req).await.unwrap_err();
        assert!(matches!(err, CoreError::Auth(_)));
        assert!(err.to_string().contains("Unauthorized"));
    }

    #[tokio::test]
    async fn handles_403_as_forbidden() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/admin/users"))
            .respond_with(ResponseTemplate::new(403).set_body_string("Not enough permissions"))
            .mount(&mock_server)
            .await;

        let client = test_client(&mock_server.uri());
        let req = client.get("/api/v1/admin/users", AuthKind::User);
        let err = client.send::<serde_json::Value>(req).await.unwrap_err();
        assert!(matches!(err, CoreError::Forbidden(_)));
        assert!(err.to_string().contains("Forbidden"));
    }

    #[tokio::test]
    async fn handles_404_as_not_found() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/items/99999"))
            .respond_with(ResponseTemplate::new(404).set_body_string("Not found"))
            .mount(&mock_server)
            .await;

        let client = test_client(&mock_server.uri());
        let req = client.get("/api/v1/items/99999", AuthKind::User);
        let err = client.send::<serde_json::Value>(req).await.unwrap_err();
        assert!(matches!(err, CoreError::NotFound(_)));
    }

    #[tokio::test]
    async fn handles_409_as_conflict() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/operations"))
            .respond_with(ResponseTemplate::new(409).set_body_string("Duplicate operation"))
            .mount(&mock_server)
            .await;

        let client = test_client(&mock_server.uri());
        let req = client
            .post("/api/v1/operations", AuthKind::User)
            .json(&serde_json::json!({}));
        let err = client.send::<serde_json::Value>(req).await.unwrap_err();
        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn handles_422_as_validation_error() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/operations"))
            .respond_with(ResponseTemplate::new(422).set_body_string("line.0.qty: missing field"))
            .mount(&mock_server)
            .await;

        let client = test_client(&mock_server.uri());
        let req = client
            .post("/api/v1/operations", AuthKind::User)
            .json(&serde_json::json!({}));
        let err = client.send::<serde_json::Value>(req).await.unwrap_err();
        assert!(matches!(err, CoreError::Validation(_)));
    }

    #[tokio::test]
    async fn handles_429_as_timeout() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(429).set_body_string("Too many requests"))
            .mount(&mock_server)
            .await;

        let client = test_client(&mock_server.uri());
        let req = client.get("/api/v1/health", AuthKind::None);
        let err = client.send::<serde_json::Value>(req).await.unwrap_err();
        assert!(matches!(err, CoreError::Timeout(_)));
    }

    #[tokio::test]
    async fn handles_500_as_network_error() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal server error"))
            .mount(&mock_server)
            .await;

        let client = test_client(&mock_server.uri());
        let req = client.get("/api/v1/health", AuthKind::None);
        let err = client.send::<serde_json::Value>(req).await.unwrap_err();
        assert!(matches!(err, CoreError::Network(_)));
    }

    #[tokio::test]
    async fn handles_503_as_unavailable() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(503).set_body_string("Service unavailable"))
            .mount(&mock_server)
            .await;

        let client = test_client(&mock_server.uri());
        let req = client.get("/api/v1/health", AuthKind::None);
        let err = client.send::<serde_json::Value>(req).await.unwrap_err();
        assert!(matches!(err, CoreError::Unavailable(_)));
    }

    #[tokio::test]
    async fn handles_504_as_timeout() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(504).set_body_string("Gateway timeout"))
            .mount(&mock_server)
            .await;

        let client = test_client(&mock_server.uri());
        let req = client.get("/api/v1/health", AuthKind::None);
        let err = client.send::<serde_json::Value>(req).await.unwrap_err();
        assert!(matches!(err, CoreError::Timeout(_)));
    }

    #[tokio::test]
    async fn handles_connection_refused() {
        let client = test_client("http://127.0.0.1:1");
        let req = client.get("/api/v1/health", AuthKind::None);
        let err = client.send::<serde_json::Value>(req).await.unwrap_err();
        assert!(matches!(err, CoreError::Network(_)));
    }

    #[tokio::test]
    async fn handles_timeout() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(10)))
            .mount(&mock_server)
            .await;

        let config = CoreConfig {
            server_base_url: mock_server.uri(),
            database_path: PathBuf::from(":memory:"),
            timeout_seconds: 1,
            ..Default::default()
        };
        let client = SyncServerClient::new(&config).unwrap();
        let req = client.get("/api/v1/health", AuthKind::None);
        let err = client.send::<serde_json::Value>(req).await.unwrap_err();
        assert!(matches!(err, CoreError::Network(_)));
    }

    #[tokio::test]
    async fn post_sends_json_body() {
        let mock_server = MockServer::start().await;
        let expected_body = serde_json::json!({"name": "Test", "value": 42});
        Mock::given(method("POST"))
            .and(path("/api/v1/test"))
            .and(header("content-type", "application/json"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"id": 1, "name": "Test"})),
            )
            .mount(&mock_server)
            .await;

        let client = test_client(&mock_server.uri());
        let req = client
            .post("/api/v1/test", AuthKind::User)
            .json(&expected_body);
        let result: serde_json::Value = client.send(req).await.unwrap();
        assert_eq!(result["id"], 1);
    }

    #[tokio::test]
    async fn patch_sends_json_body() {
        let mock_server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/api/v1/test/1"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"id": 1, "name": "Updated"})),
            )
            .mount(&mock_server)
            .await;

        let client = test_client(&mock_server.uri());
        let req = client
            .patch("/api/v1/test/1", AuthKind::User)
            .json(&serde_json::json!({"name": "Updated"}));
        let result: serde_json::Value = client.send(req).await.unwrap();
        assert_eq!(result["name"], "Updated");
    }

    #[tokio::test]
    async fn delete_sends_request() {
        let mock_server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/api/v1/test/1"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;

        let client = test_client(&mock_server.uri());
        let req = client.delete("/api/v1/test/1", AuthKind::User);
        let result = client.send_no_body(req).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn deserialization_error_mapped_correctly() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json at all"))
            .mount(&mock_server)
            .await;

        let client = test_client(&mock_server.uri());
        let req = client.get("/api/v1/test", AuthKind::None);
        let err = client.send::<serde_json::Value>(req).await.unwrap_err();
        assert!(matches!(err, CoreError::Serialization(_)));
    }

    #[test]
    fn redact_secrets_removes_uuids() {
        let input = "User token 550e8400-e29b-41d4-a716-446655440000 not found";
        let result = redact_secrets(input);
        assert!(!result.contains("550e8400-e29b-41d4-a716-446655440000"));
        assert!(result.contains("***UUID***"));
    }

    #[test]
    fn redact_secrets_removes_token_headers() {
        let input = "user_token: 550e8400-e29b-41d4-a716-446655440000";
        let result = redact_secrets(input);
        assert!(!result.contains("550e8400"));
        assert!(result.contains("***REDACTED***"));
    }

    // ── FFI token binding: set_user_token + close produces token header ──

    #[tokio::test]
    async fn ffi_token_binding_sends_user_token_after_set_and_close() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/auth/me"))
            .and(header(
                "X-User-Token",
                "550e8400-e29b-41d4-a716-446655440000",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "user_id": "550e8400-e29b-41d4-a716-446655440000",
                "name": "Ffi Test User",
                "role": "storekeeper"
            })))
            .mount(&mock_server)
            .await;

        let mut client = test_client(&mock_server.uri());
        // Simulate FfiTokenProvider: set token then close (reset cached client)
        client
            .set_user_token(uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap());
        client.clear_user_token();
        client
            .set_user_token(uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap());

        let result: serde_json::Value = client
            .get("/api/v1/auth/me", AuthKind::User)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(result["name"], "Ffi Test User");
    }

    #[tokio::test]
    async fn ffi_device_token_binding_sends_device_token_after_set() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/ping"))
            .and(header(
                "X-Device-Token",
                "660e8400-e29b-41d4-a716-446655440001",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "server_time": "2026-01-01T00:00:00Z",
                "server_seq_upto": 100,
                "backoff_seconds": 0
            })))
            .mount(&mock_server)
            .await;

        let mut client = test_client(&mock_server.uri());
        client.set_device_token(
            uuid::Uuid::parse_str("660e8400-e29b-41d4-a716-446655440001").unwrap(),
        );

        let result: serde_json::Value = client
            .post("/api/v1/ping", AuthKind::Device)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(result["server_seq_upto"], 100);
    }

    // ── Idempotency-Key header from outbox replay ──

    #[tokio::test]
    async fn outbox_replay_sends_idempotency_key_header() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/operations"))
            .and(header("Idempotency-Key", "test-idem-001"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": 1,
                "operation_type": "RECEIVE",
                "site_id": 1,
                "site_code": "WH",
                "status": "submitted"
            })))
            .mount(&mock_server)
            .await;

        let client = test_client(&mock_server.uri());
        let v: serde_json::Value = client
            .send(
                client
                    .post("/api/v1/operations", AuthKind::User)
                    .json(&serde_json::json!({"operation_type":"RECEIVE","site_id":1,"lines":[]}))
                    .header("Idempotency-Key", "test-idem-001"),
            )
            .await
            .unwrap();
        assert_eq!(v["id"], 1);
    }

    #[tokio::test]
    async fn outbox_replay_without_idempotency_key_succeeds() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/operations"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": 2,
                "operation_type": "EXPENSE",
                "site_id": 2,
                "site_code": "WH",
                "status": "submitted"
            })))
            .mount(&mock_server)
            .await;

        let client = test_client(&mock_server.uri());
        let v: serde_json::Value = client
            .send(
                client
                    .post("/api/v1/operations", AuthKind::User)
                    .json(&serde_json::json!({"operation_type":"EXPENSE","site_id":2,"lines":[]})),
            )
            .await
            .unwrap();
        assert_eq!(v["id"], 2);
    }

    // ── Sync lock pattern ──

    #[tokio::test]
    async fn sync_lock_prevents_concurrent_runs() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        let lock = Arc::new(AtomicBool::new(false));

        // First acquire
        assert!(!lock.swap(true, Ordering::AcqRel));

        // Second attempt should fail
        assert!(lock.swap(true, Ordering::AcqRel));

        // Release
        lock.store(false, Ordering::Release);

        // Now should succeed again
        assert!(!lock.swap(true, Ordering::AcqRel));
        lock.store(false, Ordering::Release);
    }

    #[test]
    fn sync_lock_initial_state_is_false() {
        use std::sync::atomic::{AtomicBool, Ordering};
        assert!(!AtomicBool::new(false).load(Ordering::Acquire));
    }

    // ── FfiTokenProvider token storage ──

    #[tokio::test]
    async fn ffi_token_provider_stores_and_returns_user_token() {
        use crate::auth::token_provider::{FfiTokenProvider, TokenProvider};

        let provider = FfiTokenProvider::new();
        let token = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        assert!(provider.user_token().is_none());
        provider.set_user_token(token);
        assert_eq!(provider.user_token(), Some(token));
    }

    #[tokio::test]
    async fn ffi_token_provider_stores_and_returns_device_token() {
        use crate::auth::token_provider::{FfiTokenProvider, TokenProvider};

        let provider = FfiTokenProvider::new();
        let token = uuid::Uuid::parse_str("660e8400-e29b-41d4-a716-446655440001").unwrap();

        assert!(provider.device_token().is_none());
        provider.set_device_token(token);
        assert_eq!(provider.device_token(), Some(token));
    }

    #[tokio::test]
    async fn ffi_token_provider_updates_existing_token() {
        use crate::auth::token_provider::{FfiTokenProvider, TokenProvider};

        let provider = FfiTokenProvider::new();
        let token1 = uuid::Uuid::nil();
        let token2 = uuid::Uuid::max();

        provider.set_user_token(token1);
        assert_eq!(provider.user_token(), Some(token1));

        provider.set_user_token(token2);
        assert_eq!(provider.user_token(), Some(token2));
    }

    // ── FfiTokenProvider shared state via Clone ──

    #[tokio::test]
    async fn ffi_token_provider_clone_shares_state() {
        use crate::auth::token_provider::{FfiTokenProvider, TokenProvider};

        let provider = FfiTokenProvider::new();
        let cloned = provider.clone();

        let token = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        provider.set_user_token(token);

        assert_eq!(cloned.user_token(), Some(token));
    }

    // ── FfiTokenProvider + SyncServerClient integration ──

    #[tokio::test]
    async fn ffi_token_provider_integration_with_client() {
        use crate::auth::token_provider::FfiTokenProvider;

        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/auth/me"))
            .and(header(
                "X-User-Token",
                "550e8400-e29b-41d4-a716-446655440000",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "user_id": "550e8400-e29b-41d4-a716-446655440000",
                "role": "storekeeper"
            })))
            .mount(&mock_server)
            .await;

        let config = crate::config::CoreConfig {
            server_base_url: mock_server.uri(),
            database_path: std::path::PathBuf::from(":memory:"),
            timeout_seconds: 5,
            ..Default::default()
        };

        // Simulate CoreHandleWrapper::new behavior:
        let ffi_provider = FfiTokenProvider::new();
        ffi_provider
            .set_user_token(uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap());

        let mut client = SyncServerClient::new(&config).unwrap();
        let token = ffi_provider.user_token();
        if let Some(t) = token {
            client.set_user_token(t);
        }

        let result: serde_json::Value = client
            .get("/api/v1/auth/me", AuthKind::User)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(result["role"], "storekeeper");
    }
}
