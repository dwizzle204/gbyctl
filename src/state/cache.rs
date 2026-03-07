//! Small local state cache.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::state::secure;

const EPHEMERAL_ENV: &str = "GBYCTL_EPHEMERAL";

/// Cached local state for planning hints.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocalState {
    /// Last detected Ubuntu release.
    pub ubuntu_version: Option<String>,
    /// Last known firewall backend.
    pub firewall_backend: Option<String>,
    /// Last package manager facts timestamp.
    pub package_facts_at: Option<DateTime<Utc>>,
    /// Operator preference for plan-first mode.
    pub plan_first: bool,
}

/// Load state from disk or return defaults if not present.
pub fn load(path: &Path) -> Result<LocalState> {
    if ephemeral_mode_enabled() {
        return Ok(LocalState::default());
    }
    if !path.exists() {
        return Ok(LocalState::default());
    }
    let raw =
        fs::read(path).with_context(|| format!("failed to read state file: {}", path.display()))?;

    let decrypted = secure::decrypt(&raw).context("state file is not valid encrypted content")?;

    let state: LocalState =
        serde_json::from_slice(&decrypted).context("failed to parse state json")?;
    Ok(state)
}

/// Persist state to disk.
pub fn store(path: &Path, state: &LocalState) -> Result<()> {
    if ephemeral_mode_enabled() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create state dir: {}", parent.display()))?;
    }
    let json = serde_json::to_vec_pretty(state).context("failed to serialize local state")?;
    let encrypted = secure::encrypt(&json).context("failed to encrypt local state")?;
    let temp_path = path.with_extension("json.tmp");
    fs::write(&temp_path, encrypted)
        .with_context(|| format!("failed to write temp state file: {}", temp_path.display()))?;
    fs::rename(&temp_path, path).with_context(|| {
        format!(
            "failed to atomically replace state file: {}",
            path.display()
        )
    })?;
    Ok(())
}

fn ephemeral_mode_enabled() -> bool {
    matches!(
        std::env::var(EPHEMERAL_ENV).ok().as_deref(),
        Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("YES")
    )
}
