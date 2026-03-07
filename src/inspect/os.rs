//! Ubuntu detection and OS information.

use std::fs;

use anyhow::{Context, Result};

/// Small OS facts used by planners.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsFacts {
    /// True when Ubuntu detected.
    pub is_ubuntu: bool,
    /// PRETTY_NAME from os-release.
    pub pretty_name: String,
}

/// Reads `/etc/os-release` and extracts Ubuntu facts.
pub fn detect() -> Result<OsFacts> {
    let content =
        fs::read_to_string("/etc/os-release").context("failed to read /etc/os-release")?;
    let pretty_name = extract_key(&content, "PRETTY_NAME").unwrap_or_else(|| "unknown".to_owned());
    let id = extract_key(&content, "ID").unwrap_or_default();

    Ok(OsFacts {
        is_ubuntu: id.eq_ignore_ascii_case("ubuntu"),
        pretty_name,
    })
}

fn extract_key(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        if let Some((lhs, rhs)) = line.split_once('=')
            && lhs == key
        {
            return Some(rhs.trim_matches('"').to_owned());
        }
    }
    None
}
