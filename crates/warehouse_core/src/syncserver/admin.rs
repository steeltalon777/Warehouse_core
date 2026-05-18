use super::client::{AuthKind, SyncServerClient};
use crate::domain::admin::{
    AdminSiteCreate, AdminSiteResponse, DeviceResponse, DeviceWithTokenResponse, UserCreate,
    UserResponse, UserWithTokenResponse,
};
use crate::domain::pagination::PaginatedResponse;
use crate::error::CoreResult;

impl SyncServerClient {
    // ── Sites ──────────────────────────────────────────

    /// GET /admin/sites — list sites (paginated)
    pub async fn admin_sites_list(
        &self,
        page: u32,
        page_size: u32,
    ) -> CoreResult<PaginatedResponse<AdminSiteResponse>> {
        let req = self.get("/api/v1/admin/sites", AuthKind::User).query(&[
            ("page", page.to_string()),
            ("page_size", page_size.to_string()),
        ]);
        self.send(req).await
    }

    /// POST /admin/sites — create a site
    pub async fn admin_sites_create(
        &self,
        site: &AdminSiteCreate,
    ) -> CoreResult<AdminSiteResponse> {
        let req = self.post("/api/v1/admin/sites", AuthKind::User).json(site);
        self.send(req).await
    }

    /// PATCH /admin/sites/{id} — update a site
    pub async fn admin_sites_update(
        &self,
        id: i32,
        update: &serde_json::Value,
    ) -> CoreResult<AdminSiteResponse> {
        let req = self
            .patch(&format!("/api/v1/admin/sites/{id}"), AuthKind::User)
            .json(update);
        self.send(req).await
    }

    // ── Users ──────────────────────────────────────────

    /// GET /admin/users — list users (root-only)
    pub async fn admin_users_list(
        &self,
        page: u32,
        page_size: u32,
    ) -> CoreResult<PaginatedResponse<UserResponse>> {
        let req = self.get("/api/v1/admin/users", AuthKind::User).query(&[
            ("page", page.to_string()),
            ("page_size", page_size.to_string()),
        ]);
        self.send(req).await
    }

    /// GET /admin/users/{id} — single user (root-only)
    pub async fn admin_users_get(&self, id: uuid::Uuid) -> CoreResult<UserResponse> {
        let req = self.get(&format!("/api/v1/admin/users/{id}"), AuthKind::User);
        self.send(req).await
    }

    /// POST /admin/users — create user (root-only)
    pub async fn admin_users_create(&self, user: &UserCreate) -> CoreResult<UserWithTokenResponse> {
        let req = self.post("/api/v1/admin/users", AuthKind::User).json(user);
        self.send(req).await
    }

    /// PATCH /admin/users/{id} — update user (root-only)
    pub async fn admin_users_update(
        &self,
        id: uuid::Uuid,
        update: &serde_json::Value,
    ) -> CoreResult<UserResponse> {
        let req = self
            .patch(&format!("/api/v1/admin/users/{id}"), AuthKind::User)
            .json(update);
        self.send(req).await
    }

    /// DELETE /admin/users/{id} — soft-deactivate user (root-only)
    pub async fn admin_users_delete(&self, id: uuid::Uuid) -> CoreResult<()> {
        let req = self.delete(&format!("/api/v1/admin/users/{id}"), AuthKind::User);
        self.send_no_body(req).await
    }

    /// POST /admin/users/{id}/rotate-token — rotate user token (root-only)
    pub async fn admin_users_rotate_token(
        &self,
        id: uuid::Uuid,
    ) -> CoreResult<UserWithTokenResponse> {
        let req = self.post(
            &format!("/api/v1/admin/users/{id}/rotate-token"),
            AuthKind::User,
        );
        self.send(req).await
    }

    // ── Devices ────────────────────────────────────────

    /// GET /admin/devices — list devices (paginated)
    pub async fn admin_devices_list(
        &self,
        page: u32,
        page_size: u32,
    ) -> CoreResult<PaginatedResponse<DeviceResponse>> {
        let req = self.get("/api/v1/admin/devices", AuthKind::User).query(&[
            ("page", page.to_string()),
            ("page_size", page_size.to_string()),
        ]);
        self.send(req).await
    }

    /// GET /admin/devices/{id} — single device
    pub async fn admin_devices_get(&self, id: i32) -> CoreResult<DeviceResponse> {
        let req = self.get(&format!("/api/v1/admin/devices/{id}"), AuthKind::User);
        self.send(req).await
    }

    /// POST /admin/devices — register a device
    pub async fn admin_devices_register(
        &self,
        device: &serde_json::Value,
    ) -> CoreResult<DeviceWithTokenResponse> {
        let req = self
            .post("/api/v1/admin/devices", AuthKind::User)
            .json(device);
        self.send(req).await
    }

    /// PATCH /admin/devices/{id} — update device
    pub async fn admin_devices_update(
        &self,
        id: i32,
        update: &serde_json::Value,
    ) -> CoreResult<DeviceResponse> {
        let req = self
            .patch(&format!("/api/v1/admin/devices/{id}"), AuthKind::User)
            .json(update);
        self.send(req).await
    }

    /// DELETE /admin/devices/{id} — soft-delete device
    pub async fn admin_devices_delete(&self, id: i32) -> CoreResult<()> {
        let req = self.delete(&format!("/api/v1/admin/devices/{id}"), AuthKind::User);
        self.send_no_body(req).await
    }

    /// POST /admin/devices/{id}/rotate-token — rotate device token
    pub async fn admin_devices_rotate_token(&self, id: i32) -> CoreResult<DeviceWithTokenResponse> {
        let req = self.post(
            &format!("/api/v1/admin/devices/{id}/rotate-token"),
            AuthKind::User,
        );
        self.send(req).await
    }
}
