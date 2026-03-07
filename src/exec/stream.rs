//! Stream formatting for command execution output.

use std::io::{self, Write};

use anyhow::{Context, Result};

/// Emits one prefixed line to stdout.
pub fn emit(prefix: &str, line: &str) -> Result<()> {
    let mut stdout = io::stdout().lock();
    let payload = format!("[{prefix}] {line}\n");
    stdout
        .write_all(payload.as_bytes())
        .context("failed to write stream output")?;
    stdout.flush().context("failed to flush stream output")?;
    Ok(())
}
