use super::client::{AuthKind, SyncServerClient};
use crate::domain::auth::{AuthContext, SyncUserResponse};
use crate::domain::site::SiteDto;
use crate::error::CoreResult;

impl SyncServerClient {
    /// POST /auth/sync-user — sync/create a user (root-only)
    pub async fn auth_sync_user(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> CoreResult<SyncUserResponse> {
        let body = serde_json::json!({
            "username": username,
            "email": email,
            "password": password,
        });
        let req = self
            .post("/api/v1/auth/sync-user", AuthKind::User)
            .json(&body);
        self.send(req).await
    }

    /// GET /auth/me — current user info
    pub async fn auth_me(&self) -> CoreResult<AuthContext> {
        let req = self.get("/api/v1/auth/me", AuthKind::User);
        self.send(req).await
    }

    /// GET /auth/sites — available sites
    pub async fn auth_sites(&self) -> CoreResult<Vec<SiteDto>> {
        let req = self.get("/api/v1/auth/sites", AuthKind::User);
        self.send(req).await
    }

    /// GET /auth/context — full auth context (user, role, sites, permissions, device)
    pub async fn auth_context(&self) -> CoreResult<AuthContext> {
        let req = self.get("/api/v1/auth/context", AuthKind::User);
        self.send(req).await
    }
}
