//! Package inspection helpers.

use crate::exec::runner;

/// Package management facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageFacts {
    /// Upgradable packages list.
    pub upgradable: String,
}

/// Collect apt package facts.
pub fn collect() -> anyhow::Result<PackageFacts> {
    let upgradable = runner::capture("apt list --upgradable")?;
    Ok(PackageFacts { upgradable })
}
