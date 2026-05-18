use super::client::{AuthKind, SyncServerClient};
use crate::domain::sync_types::{
    BootstrapResponse, PingRequest, PingResponse, PullRequest, PullResponse, PushRequest,
    PushResponse,
};
use crate::error::CoreResult;

impl SyncServerClient {
    /// POST /ping — device handshake
    pub async fn ping(&self, req: &PingRequest) -> CoreResult<PingResponse> {
        let req = self.post("/api/v1/ping", AuthKind::Device).json(req);
        self.send(req).await
    }

    /// POST /push — push device events to server
    pub async fn push(&self, req: &PushRequest) -> CoreResult<PushResponse> {
        let req = self.post("/api/v1/push", AuthKind::Device).json(req);
        self.send(req).await
    }

    /// POST /pull — pull new events from server
    pub async fn pull(&self, req: &PullRequest) -> CoreResult<PullResponse> {
        let req = self.post("/api/v1/pull", AuthKind::Device).json(req);
        self.send(req).await
    }

    /// POST /bootstrap/sync — full bootstrap data (root-only, user auth)
    pub async fn bootstrap_sync(&self) -> CoreResult<BootstrapResponse> {
        let req = self.post("/api/v1/bootstrap/sync", AuthKind::User);
        self.send(req).await
    }
}
