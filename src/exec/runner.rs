//! Command execution with live output streaming.

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;

use anyhow::{Context, Result};

use crate::exec::command;
use crate::exec::stream;

/// Command execution result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandResult {
    /// Exit code.
    pub exit_code: i32,
    /// Combined stdout and stderr.
    pub output: String,
}

/// Execute command with live streaming and collect output.
pub fn run_streaming(command: &str) -> Result<CommandResult> {
    let parsed = command::parse(command).context("failed to parse command for execution")?;
    let mut child = Command::new(&parsed.program)
        .args(&parsed.args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to spawn command: {command}"))?;

    let stdout = child.stdout.take().context("stdout unavailable")?;
    let stderr = child.stderr.take().context("stderr unavailable")?;

    let out_handle = thread::spawn(move || -> Result<Vec<String>> {
        let reader = BufReader::new(stdout);
        let mut lines = Vec::new();
        for line in reader.lines() {
            let line = line.context("failed to read stdout line")?;
            stream::emit("stdout", &line)?;
            lines.push(line);
        }
        Ok(lines)
    });

    let err_handle = thread::spawn(move || -> Result<Vec<String>> {
        let reader = BufReader::new(stderr);
        let mut lines = Vec::new();
        for line in reader.lines() {
            let line = line.context("failed to read stderr line")?;
            stream::emit("stderr", &line)?;
            lines.push(line);
        }
        Ok(lines)
    });

    let status = child.wait().context("failed waiting for command")?;

    let out_lines = out_handle
        .join()
        .map_err(|_| anyhow::anyhow!("stdout thread panicked"))??;
    let err_lines = err_handle
        .join()
        .map_err(|_| anyhow::anyhow!("stderr thread panicked"))??;

    let mut combined = String::new();
    for line in out_lines {
        combined.push_str(&line);
        combined.push('\n');
    }
    for line in err_lines {
        combined.push_str(&line);
        combined.push('\n');
    }

    let code = status.code().unwrap_or(1);

    Ok(CommandResult {
        exit_code: code,
        output: combined,
    })
}

/// Capture command output without streaming context tags.
pub fn capture(command: &str) -> Result<String> {
    let parsed = command::parse(command).context("failed to parse command for capture")?;
    let output = Command::new(&parsed.program)
        .args(&parsed.args)
        .output()
        .with_context(|| format!("failed to run command: {command}"))?;

    let mut combined = String::new();
    combined.push_str(&String::from_utf8_lossy(&output.stdout));
    combined.push_str(&String::from_utf8_lossy(&output.stderr));
    Ok(combined)
}

/// Execute command without streaming any incremental output.
pub fn run_quiet(command: &str) -> Result<CommandResult> {
    let parsed = command::parse(command).context("failed to parse command for quiet execution")?;
    let output = Command::new(&parsed.program)
        .args(&parsed.args)
        .output()
        .with_context(|| format!("failed to run command: {command}"))?;

    let mut combined = String::new();
    combined.push_str(&String::from_utf8_lossy(&output.stdout));
    combined.push_str(&String::from_utf8_lossy(&output.stderr));

    Ok(CommandResult {
        exit_code: output.status.code().unwrap_or(1),
        output: combined,
    })
}

#[cfg(test)]
mod tests {
    use super::capture;

    #[test]
    fn rejects_shell_operator_in_capture() {
        let result = capture("echo test && whoami");
        assert!(result.is_err());
    }
}
