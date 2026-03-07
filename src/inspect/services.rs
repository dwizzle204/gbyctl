//! Service inspection helpers.

use crate::exec::runner;

/// Service summary facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceFacts {
    /// Failed unit list.
    pub failed_units: String,
}

/// Collect failed services.
pub fn collect() -> anyhow::Result<ServiceFacts> {
    let failed_units = runner::capture("systemctl --failed --no-pager")?;
    Ok(ServiceFacts { failed_units })
}
