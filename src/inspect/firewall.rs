//! Firewall inspection helpers.

use crate::exec::runner;

/// Firewall facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirewallFacts {
    /// UFW status output.
    pub ufw_status: String,
}

/// Collect firewall facts.
pub fn collect() -> anyhow::Result<FirewallFacts> {
    let ufw_status = runner::capture("ufw status verbose")?;
    Ok(FirewallFacts { ufw_status })
}
