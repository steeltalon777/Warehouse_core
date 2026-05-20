use crate::domain::auth::AuthSiteInfo;
use crate::error::{CoreError, CoreResult};
use crate::storage::repos::AuthContextRepo;
use crate::time::Timestamp;

/// Non-secret profile identity cached in local SQLite.
#[derive(Debug, Clone)]
pub struct Profile {
    pub user_id: uuid::Uuid,
    pub user_name: String,
    pub user_email: String,
    pub role: String,
    pub is_root: bool,
    pub available_sites: Vec<AuthSiteInfo>,
    pub device_id: i32,
    pub device_registered: bool,
    pub active_site_id: Option<i32>,
    pub protocol_version: String,
    pub refreshed_at: String,
}

impl Profile {
    const KEY_USER_ID: &str = "profile_user_id";
    const KEY_USER_NAME: &str = "profile_user_name";
    const KEY_USER_EMAIL: &str = "profile_user_email";
    const KEY_ROLE: &str = "profile_role";
    const KEY_IS_ROOT: &str = "profile_is_root";
    const KEY_DEVICE_ID: &str = "profile_device_id";
    const KEY_DEVICE_REGISTERED: &str = "profile_device_registered";
    const KEY_ACTIVE_SITE_ID: &str = "profile_active_site_id";
    const KEY_PROTOCOL_VERSION: &str = "profile_protocol_version";
    const KEY_REFRESHED_AT: &str = "profile_refreshed_at";

    /// Load profile from SQLite key-value store.
    pub async fn load(repo: &impl AuthContextRepo) -> CoreResult<Option<Self>> {
        let user_id = repo.get(Self::KEY_USER_ID).await?;
        let user_id = match user_id {
            Some(v) => uuid::Uuid::parse_str(&v).map_err(|e| CoreError::Config(e.to_string()))?,
            None => return Ok(None),
        };

        let available_sites_json = repo
            .get("profile_available_sites")
            .await?
            .unwrap_or_else(|| "[]".to_string());
        let available_sites: Vec<AuthSiteInfo> =
            serde_json::from_str(&available_sites_json).map_err(CoreError::Serialization)?;

        let active_site = repo.get(Self::KEY_ACTIVE_SITE_ID).await?;
        let active_site_id = active_site.and_then(|s| s.parse::<i32>().ok());

        Ok(Some(Self {
            user_id,
            user_name: repo.get(Self::KEY_USER_NAME).await?.unwrap_or_default(),
            user_email: repo.get(Self::KEY_USER_EMAIL).await?.unwrap_or_default(),
            role: repo.get(Self::KEY_ROLE).await?.unwrap_or_default(),
            is_root: repo
                .get(Self::KEY_IS_ROOT)
                .await?
                .map(|v| v == "true")
                .unwrap_or(false),
            available_sites,
            device_id: repo
                .get(Self::KEY_DEVICE_ID)
                .await?
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0),
            device_registered: repo
                .get(Self::KEY_DEVICE_REGISTERED)
                .await?
                .map(|v| v == "true")
                .unwrap_or(false),
            active_site_id,
            protocol_version: repo
                .get(Self::KEY_PROTOCOL_VERSION)
                .await?
                .unwrap_or_default(),
            refreshed_at: repo.get(Self::KEY_REFRESHED_AT).await?.unwrap_or_default(),
        }))
    }

    /// Persist profile fields to SQLite key-value store.
    pub async fn save(&self, repo: &impl AuthContextRepo) -> CoreResult<()> {
        repo.set(Self::KEY_USER_ID, &self.user_id.to_string())
            .await?;
        repo.set(Self::KEY_USER_NAME, &self.user_name).await?;
        repo.set(Self::KEY_USER_EMAIL, &self.user_email).await?;
        repo.set(Self::KEY_ROLE, &self.role).await?;
        repo.set(
            Self::KEY_IS_ROOT,
            if self.is_root { "true" } else { "false" },
        )
        .await?;
        repo.set(Self::KEY_DEVICE_ID, &self.device_id.to_string())
            .await?;
        repo.set(
            Self::KEY_DEVICE_REGISTERED,
            if self.device_registered {
                "true"
            } else {
                "false"
            },
        )
        .await?;
        repo.set(Self::KEY_PROTOCOL_VERSION, &self.protocol_version)
            .await?;
        repo.set(Self::KEY_REFRESHED_AT, &self.refreshed_at).await?;

        let sites_json =
            serde_json::to_string(&self.available_sites).map_err(CoreError::Serialization)?;
        repo.set("profile_available_sites", &sites_json).await?;

        if let Some(id) = self.active_site_id {
            repo.set(Self::KEY_ACTIVE_SITE_ID, &id.to_string()).await?;
        } else {
            repo.delete(Self::KEY_ACTIVE_SITE_ID).await?;
        }

        Ok(())
    }

    /// Update active site and persist.
    pub async fn set_active_site(
        &mut self,
        site_id: i32,
        repo: &impl AuthContextRepo,
    ) -> CoreResult<()> {
        if !self.available_sites.iter().any(|s| s.site_id == site_id) {
            return Err(CoreError::Validation(format!(
                "Site {site_id} is not in the available sites list"
            )));
        }
        self.active_site_id = Some(site_id);
        repo.set(Self::KEY_ACTIVE_SITE_ID, &site_id.to_string())
            .await
    }

    /// Remove active site without clearing the entire profile.
    pub async fn clear_active_site(&mut self, repo: &impl AuthContextRepo) -> CoreResult<()> {
        self.active_site_id = None;
        repo.delete(Self::KEY_ACTIVE_SITE_ID).await
    }

    /// Delete all profile keys from storage (logout).
    pub async fn clear(repo: &impl AuthContextRepo) -> CoreResult<()> {
        for key in &[
            Self::KEY_USER_ID,
            Self::KEY_USER_NAME,
            Self::KEY_USER_EMAIL,
            Self::KEY_ROLE,
            Self::KEY_IS_ROOT,
            Self::KEY_DEVICE_ID,
            Self::KEY_DEVICE_REGISTERED,
            Self::KEY_ACTIVE_SITE_ID,
            Self::KEY_PROTOCOL_VERSION,
            Self::KEY_REFRESHED_AT,
        ] {
            repo.delete(key).await?;
        }
        repo.delete("profile_available_sites").await?;
        Ok(())
    }
}

/// Service for refreshing and managing the identity profile.
#[derive(Debug)]
pub struct ProfileService {
    profile: Option<Profile>,
}

impl ProfileService {
    pub fn new() -> Self {
        Self { profile: None }
    }
}

impl Default for ProfileService {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfileService {
    /// Load existing profile from DB.
    pub async fn load(&mut self, repo: &impl AuthContextRepo) -> CoreResult<()> {
        self.profile = Profile::load(repo).await?;
        Ok(())
    }

    /// Get a reference to the current profile, or error if not loaded.
    pub fn current(&self) -> CoreResult<&Profile> {
        self.profile.as_ref().ok_or_else(|| {
            CoreError::Auth("No active profile. Call refresh_identity first.".into())
        })
    }

    /// Refresh identity from SyncServer `/auth/context` result and persist.
    pub async fn refresh_from_auth_context(
        &mut self,
        ctx: crate::domain::auth::AuthContext,
        repo: &impl AuthContextRepo,
    ) -> CoreResult<()> {
        let now = Timestamp::now_utc().to_string();

        let profile = Profile {
            user_id: ctx.user.id,
            user_name: ctx.user.username,
            user_email: ctx.user.email,
            role: ctx.role,
            is_root: ctx.is_root,
            available_sites: ctx.available_sites,
            device_id: ctx.device.as_ref().map(|d| d.id).unwrap_or(0),
            device_registered: ctx.device.is_some(),
            active_site_id: ctx.user.default_site_id,
            protocol_version: String::new(),
            refreshed_at: now,
        };

        profile.save(repo).await?;
        self.profile = Some(profile);
        Ok(())
    }

    /// Set active site (validates against available sites).
    pub async fn set_active_site(
        &mut self,
        site_id: i32,
        repo: &impl AuthContextRepo,
    ) -> CoreResult<()> {
        match self.profile {
            Some(ref mut p) => p.set_active_site(site_id, repo).await,
            None => Err(CoreError::Auth("No active profile".into())),
        }
    }

    /// Clear active site.
    pub async fn clear_active_site(&mut self, repo: &impl AuthContextRepo) -> CoreResult<()> {
        match self.profile {
            Some(ref mut p) => p.clear_active_site(repo).await,
            None => Err(CoreError::Auth("No active profile".into())),
        }
    }

    /// Full logout: clear profile and remove from storage.
    pub async fn logout(&mut self, repo: &impl AuthContextRepo) -> CoreResult<()> {
        Profile::clear(repo).await?;
        self.profile = None;
        Ok(())
    }

    /// Validate that the given site_id is in the user's available sites.
    pub fn validate_site_access(&self, site_id: i32) -> CoreResult<()> {
        match self.profile {
            Some(ref p) => {
                if p.available_sites.iter().any(|s| s.site_id == site_id) {
                    Ok(())
                } else {
                    Err(CoreError::Auth(format!(
                        "User does not have access to site {site_id}"
                    )))
                }
            }
            None => Err(CoreError::Auth("No active profile".into())),
        }
    }

    /// List available sites from the current profile.
    pub fn sites(&self) -> CoreResult<&[AuthSiteInfo]> {
        match self.profile {
            Some(ref p) => Ok(&p.available_sites),
            None => Err(CoreError::Auth("No active profile".into())),
        }
    }

    /// Available site IDs for quick permission checks.
    pub fn available_site_ids(&self) -> CoreResult<Vec<i32>> {
        match self.profile {
            Some(ref p) => Ok(p.available_sites.iter().map(|s| s.site_id).collect()),
            None => Err(CoreError::Auth("No active profile".into())),
        }
    }

    /// Check if the user has a specific permission on the active site.
    pub fn has_permission(&self, site_id: i32, permission: &str) -> CoreResult<bool> {
        match self.profile {
            Some(ref p) => Ok(p
                .available_sites
                .iter()
                .find(|s| s.site_id == site_id)
                .and_then(|s| s.permissions.get(permission))
                .copied()
                .unwrap_or(false)),
            None => Err(CoreError::Auth("No active profile".into())),
        }
    }

    /// Whether a profile has been loaded.
    pub fn is_authenticated(&self) -> bool {
        self.profile.is_some()
    }
}
