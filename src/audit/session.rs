//! Local session logging.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::skills::types::{Plan, PolicyClass};
use crate::state::secure;

const EPHEMERAL_ENV: &str = "GBYCTL_EPHEMERAL";

/// Per-session execution log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLog {
    /// Session id.
    pub id: String,
    /// Start timestamp.
    pub started_at: DateTime<Utc>,
    /// Original request.
    pub request: String,
    /// Selected skill.
    pub skill: String,
    /// Overall outcome.
    pub outcome: String,
    /// Highest policy encountered.
    pub highest_policy: PolicyClass,
}

/// Writes a session log to local JSON storage.
pub fn persist(
    base_dir: &Path,
    plan: &Plan,
    outcome: &str,
    highest: PolicyClass,
) -> Result<PathBuf> {
    if ephemeral_mode_enabled() {
        return Ok(base_dir.join("ephemeral-session.json"));
    }
    fs::create_dir_all(base_dir)
        .with_context(|| format!("failed to create audit dir: {}", base_dir.display()))?;

    let now = Utc::now();
    let id = now.format("%Y%m%dT%H%M%SZ").to_string();
    let log = SessionLog {
        id: id.clone(),
        started_at: now,
        request: plan.request.clone(),
        skill: plan.skill_id.as_str().to_owned(),
        outcome: outcome.to_owned(),
        highest_policy: highest,
    };

    let path = base_dir.join(format!("{id}.json"));
    let raw = serde_json::to_vec_pretty(&log).context("failed to serialize session log")?;
    let encrypted = secure::encrypt(&raw).context("failed to encrypt session log")?;
    let temp_path = base_dir.join(format!("{id}.json.tmp"));
    fs::write(&temp_path, encrypted)
        .with_context(|| format!("failed to write temp session log: {}", temp_path.display()))?;
    fs::rename(&temp_path, &path)
        .with_context(|| format!("failed to atomically write session log: {}", path.display()))?;
    Ok(path)
}

fn ephemeral_mode_enabled() -> bool {
    matches!(
        std::env::var(EPHEMERAL_ENV).ok().as_deref(),
        Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("YES")
    )
}
