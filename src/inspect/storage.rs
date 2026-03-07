//! Storage inspection helpers.

use crate::exec::runner;

/// Snapshot of storage commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageFacts {
    /// `df -h` output.
    pub df: String,
    /// `lsblk -f` output.
    pub lsblk: String,
}

/// Collect storage facts.
pub fn collect() -> anyhow::Result<StorageFacts> {
    let df = runner::capture("df -h")?;
    let lsblk = runner::capture("lsblk -f")?;
    Ok(StorageFacts { df, lsblk })
}
