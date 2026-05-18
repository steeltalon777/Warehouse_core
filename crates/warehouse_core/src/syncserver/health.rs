use serde::Deserialize;

use super::client::{AuthKind, SyncServerClient};
use crate::error::CoreResult;

#[derive(Debug, Clone, Deserialize)]
pub struct ServerInfo {
    pub message: String,
    pub status: String,
    pub env: String,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HealthStatus {
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DetailedHealth {
    pub status: String,
    pub database: String,
    pub cache: Option<String>,
    pub version: String,
}

impl SyncServerClient {
    /// GET / — server info
    pub async fn server_info(&self) -> CoreResult<ServerInfo> {
        let req = self.get("/", AuthKind::None);
        self.send(req).await
    }

    /// GET /health — simple OK
    pub async fn health(&self) -> CoreResult<HealthStatus> {
        let req = self.get("/api/v1/health", AuthKind::None);
        self.send(req).await
    }

    /// GET /ready — readiness check
    pub async fn ready(&self) -> CoreResult<HealthStatus> {
        let req = self.get("/api/v1/ready", AuthKind::None);
        self.send(req).await
    }

    /// GET /health/detailed — comprehensive check
    pub async fn health_detailed(&self) -> CoreResult<DetailedHealth> {
        let req = self.get("/api/v1/health/detailed", AuthKind::None);
        self.send(req).await
    }

    /// GET /health/liveness — K8s liveness
    pub async fn health_liveness(&self) -> CoreResult<HealthStatus> {
        let req = self.get("/api/v1/health/liveness", AuthKind::None);
        self.send(req).await
    }
}
