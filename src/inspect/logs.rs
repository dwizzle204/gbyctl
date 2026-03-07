//! Logs inspection helpers.

use crate::exec::runner;

/// Basic log facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogFacts {
    /// Recent kernel errors.
    pub kernel_errors: String,
}

/// Collect recent kernel errors.
pub fn collect() -> anyhow::Result<LogFacts> {
    let kernel_errors = runner::capture("journalctl -k -p err -n 50 --no-pager")?;
    Ok(LogFacts { kernel_errors })
}
