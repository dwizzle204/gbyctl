//! Input sanitization for user-derived values inserted into commands.

use anyhow::Context;
use anyhow::Result;

/// Validate and normalize a service name.
pub fn service_name(input: &str) -> Result<String> {
    let valid = input
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '@'));
    if !valid {
        return Err(anyhow::anyhow!("invalid service name format"));
    }
    Ok(input.to_owned())
}

/// Validate firewall/listener port.
pub fn port(input: &str) -> Result<String> {
    if input.is_empty() || !input.chars().all(|c| c.is_ascii_digit()) {
        return Err(anyhow::anyhow!("invalid port format"));
    }
    let parsed: u16 = input.parse().context("port must fit u16")?;
    if parsed == 0 {
        return Err(anyhow::anyhow!("port 0 is not valid"));
    }
    Ok(parsed.to_string())
}

/// Validate apt package identifiers inserted into install commands.
pub fn package_name(input: &str) -> Result<String> {
    let valid = !input.is_empty()
        && input
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '+' | '-'));
    if !valid {
        return Err(anyhow::anyhow!("invalid package name format"));
    }
    Ok(input.to_owned())
}
