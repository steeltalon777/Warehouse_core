use crate::auth::profile::Profile;
use crate::error::CoreResult;
use crate::storage::cursor_store::CursorStore;
use crate::storage::cursor_store::keys;
use crate::storage::repos::SqliteAuthContextRepo;
use crate::storage::snapshot_writer::SnapshotWriter;
use crate::syncserver::SyncServerClient;

/// Result of a bootstrap run.
#[derive(Debug, Clone)]
pub struct BootstrapResult {
    pub success: bool,
    pub profile: Option<Profile>,
    pub protocol_version: String,
    pub families_synced: Vec<String>,
    pub errors: Vec<String>,
    pub server_info: String,
}

/// BootstrapService: first-run protocol check, identity, initial working set.
pub struct BootstrapService {
    client: SyncServerClient,
    auth_repo: SqliteAuthContextRepo,
    cursor_store: CursorStore,
    writer: SnapshotWriter,
}

impl BootstrapService {
    pub fn new(
        client: SyncServerClient,
        auth_repo: SqliteAuthContextRepo,
        cursor_store: CursorStore,
        writer: SnapshotWriter,
    ) -> Self {
        Self {
            client,
            auth_repo,
            cursor_store,
            writer,
        }
    }

    /// Full bootstrap: health check -> identity -> catalog -> sites.
    pub async fn run_bootstrap(
        &mut self,
        existing_profile: Option<&Profile>,
    ) -> CoreResult<BootstrapResult> {
        let mut result = BootstrapResult {
            success: false,
            profile: None,
            protocol_version: String::new(),
            families_synced: Vec::new(),
            errors: Vec::new(),
            server_info: String::new(),
        };

        // Step 1: Health check
        match self.health_check().await {
            Ok(info) => result.server_info = info,
            Err(e) => {
                result.errors.push(format!("health: {e}"));
                return Ok(result);
            }
        }

        // Step 2: Protocol check
        match self.ready_check().await {
            Ok(version) => result.protocol_version = version,
            Err(e) => {
                result.errors.push(format!("protocol: {e}"));
                return Ok(result);
            }
        }

        // Step 3: Refresh identity if no existing profile
        if existing_profile.is_none() {
            match self.refresh_identity().await {
                Ok(profile) => {
                    result.profile = Some(profile);
                    result.families_synced.push("identity".to_string());
                }
                Err(e) => {
                    result.errors.push(format!("identity: {e}"));
                    return Ok(result);
                }
            }
        }

        // Step 4: Catalog sync (items, categories, units)
        match self.sync_catalog_items().await {
            Ok(true) => result.families_synced.push("catalog_items".to_string()),
            Ok(false) => result.errors.push("catalog_items: empty response".into()),
            Err(e) => result.errors.push(format!("catalog_items: {e}")),
        }
        match self.sync_catalog_categories().await {
            Ok(true) => result
                .families_synced
                .push("catalog_categories".to_string()),
            Ok(false) => result
                .errors
                .push("catalog_categories: empty response".into()),
            Err(e) => result.errors.push(format!("catalog_categories: {e}")),
        }
        match self.sync_catalog_units().await {
            Ok(true) => result.families_synced.push("catalog_units".to_string()),
            Ok(false) => result.errors.push("catalog_units: empty response".into()),
            Err(e) => result.errors.push(format!("catalog_units: {e}")),
        }

        // Step 5: Sites
        match self.client.catalog_sites().await {
            Ok(sites) => {
                if let Err(e) = self.writer.write_sites(&sites).await {
                    result.errors.push(format!("sites: {e}"));
                } else {
                    result.families_synced.push("sites".to_string());
                    let _ = self
                        .cursor_store
                        .set_updated_after(
                            keys::SITES,
                            &crate::time::Timestamp::now_utc().to_string(),
                        )
                        .await;
                }
            }
            Err(e) => {
                result.errors.push(format!("sites: {e}"));
            }
        }

        result.success = !result.errors.iter().any(|e| {
            e.starts_with("health") || e.starts_with("protocol") || e.starts_with("identity")
        });
        Ok(result)
    }

    async fn health_check(&self) -> CoreResult<String> {
        let info = self.client.server_info().await?;
        Ok(format!("{} v{}", info.status, info.version))
    }

    async fn ready_check(&self) -> CoreResult<String> {
        let ready = self.client.ready().await?;
        if ready.status == "ok" || ready.status == "healthy" || ready.status == "ready" {
            Ok("ok".to_string())
        } else {
            Err(crate::error::CoreError::Network(format!(
                "Server not ready: {}",
                ready.status
            )))
        }
    }

    async fn refresh_identity(&mut self) -> CoreResult<Profile> {
        let ctx = self.client.auth_context().await?;
        let now = crate::time::Timestamp::now_utc().to_string();
        let profile = Profile {
            user_id: ctx.user.id,
            user_name: ctx.user.username,
            user_email: ctx.user.email,
            role: ctx.role,
            is_root: ctx.is_root,
            available_sites: ctx.available_sites,
            device_id: ctx.device.as_ref().map(|d| d.id).unwrap_or_default(),
            device_registered: ctx.device.is_some(),
            active_site_id: ctx.user.default_site_id,
            protocol_version: String::new(),
            refreshed_at: now,
        };
        profile.save(&self.auth_repo).await?;
        Ok(profile)
    }

    async fn sync_catalog_items(&self) -> CoreResult<bool> {
        let cursor = self
            .cursor_store
            .get_updated_after(keys::CATALOG_ITEMS)
            .await?;
        let resp = self
            .client
            .catalog_items(cursor.as_deref(), Some(500))
            .await?;
        self.writer.write_items(&resp.items).await?;
        if let Some(next) = &resp.next_updated_after {
            self.cursor_store
                .set_updated_after(keys::CATALOG_ITEMS, next)
                .await?;
        }
        Ok(!resp.items.is_empty())
    }

    async fn sync_catalog_categories(&self) -> CoreResult<bool> {
        let cursor = self
            .cursor_store
            .get_updated_after(keys::CATALOG_CATEGORIES)
            .await?;
        let resp = self
            .client
            .catalog_categories(cursor.as_deref(), Some(500))
            .await?;
        self.writer.write_categories(&resp.items).await?;
        if let Some(next) = &resp.next_updated_after {
            self.cursor_store
                .set_updated_after(keys::CATALOG_CATEGORIES, next)
                .await?;
        }
        Ok(!resp.items.is_empty())
    }

    async fn sync_catalog_units(&self) -> CoreResult<bool> {
        let cursor = self
            .cursor_store
            .get_updated_after(keys::CATALOG_UNITS)
            .await?;
        let resp = self
            .client
            .catalog_units(cursor.as_deref(), Some(500))
            .await?;
        self.writer.write_units(&resp.items).await?;
        if let Some(next) = &resp.next_updated_after {
            self.cursor_store
                .set_updated_after(keys::CATALOG_UNITS, next)
                .await?;
        }
        Ok(!resp.items.is_empty())
    }
}
