//! Model provider configuration and secure API key storage.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use keyring::Entry;
use serde::{Deserialize, Serialize};

const SERVICE_NAME: &str = "gbyctl";

/// LLM provider family.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderKind {
    /// OpenAI or OpenAI-compatible endpoint.
    OpenAiCompatible,
    /// Anthropic Claude endpoint.
    Claude,
}

/// Persistent model configuration (non-secret fields).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Provider family.
    pub provider: ProviderKind,
    /// API base URL.
    pub base_url: String,
    /// Model identifier.
    pub model: String,
    /// Key identifier in secure store.
    pub api_key_id: String,
}

impl ModelConfig {
    /// Returns true when an API key exists in secure storage.
    pub fn has_api_key(&self) -> Result<bool> {
        let entry =
            Entry::new(SERVICE_NAME, &self.api_key_id).context("failed creating keyring entry")?;
        let secret = entry.get_password();
        if secret.is_ok() {
            return Ok(true);
        }
        Ok(false)
    }

    /// Reads API key from secure storage.
    pub fn read_api_key(&self) -> Result<String> {
        let entry =
            Entry::new(SERVICE_NAME, &self.api_key_id).context("failed creating keyring entry")?;
        entry
            .get_password()
            .context("failed reading API key from keyring")
    }

    /// Writes API key to secure storage.
    pub fn write_api_key(&self, api_key: &str) -> Result<()> {
        let entry =
            Entry::new(SERVICE_NAME, &self.api_key_id).context("failed creating keyring entry")?;
        entry
            .set_password(api_key)
            .context("failed writing API key to keyring")
    }
}

/// Load config if present.
pub fn load() -> Result<Option<ModelConfig>> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("failed reading config: {}", path.display()))?;
    let cfg: ModelConfig = serde_json::from_str(&raw).context("invalid config json")?;
    Ok(Some(cfg))
}

/// Persist non-secret config to disk.
pub fn store(cfg: &ModelConfig) -> Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed creating config dir: {}", parent.display()))?;
    }

    let raw = serde_json::to_vec_pretty(cfg).context("failed to serialize config")?;

    let mut file = fs::File::create(&path)
        .with_context(|| format!("failed writing config: {}", path.display()))?;
    file.write_all(&raw)
        .with_context(|| format!("failed writing config bytes: {}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o600);
        fs::set_permissions(&path, perms)
            .with_context(|| format!("failed setting config permissions: {}", path.display()))?;
    }

    Ok(())
}

/// Resolve default config file path.
pub fn config_path() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME env is not set")?;
    Ok(PathBuf::from(home).join(".config/gbyctl/config.json"))
}
