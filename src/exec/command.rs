//! Shell-free command parsing and validation.

use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result};

const FORBIDDEN_TOKENS: [&str; 10] = [";", "&&", "||", "|", "`", "$(", ")", "<", ">", "#"];

/// Parsed command with program and argv.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCommand {
    /// Executable name.
    pub program: OsString,
    /// Argument vector excluding program.
    pub args: Vec<OsString>,
}

/// Parse a bounded command string into argv form.
pub fn parse(command: &str) -> Result<ParsedCommand> {
    for token in FORBIDDEN_TOKENS {
        if command.contains(token) {
            return Err(anyhow::anyhow!(
                "unsafe shell token detected in command: {token}"
            ));
        }
    }

    let parts = shlex::split(command).context("invalid command syntax")?;
    if parts.is_empty() {
        return Err(anyhow::anyhow!("empty command is not allowed"));
    }

    let mut iter = parts.into_iter();
    let program = OsString::from(
        iter.next()
            .ok_or_else(|| anyhow::anyhow!("missing command program"))?,
    );
    let args = iter.map(OsString::from).collect();
    Ok(ParsedCommand { program, args })
}

/// Normalize path-like arguments for protected path checks.
#[must_use]
pub fn extract_path_args(command: &str) -> Vec<PathBuf> {
    let Ok(parsed) = parse(command) else {
        return Vec::new();
    };

    let mut paths = Vec::new();
    for arg in parsed.args {
        let text = arg.to_string_lossy();
        if text.starts_with('-') || !looks_like_path(&text) {
            continue;
        }
        paths.push(normalize_path(Path::new(text.as_ref())));
    }
    paths
}

fn looks_like_path(text: &str) -> bool {
    text.starts_with('/') || text.starts_with("./") || text.starts_with("../") || text.contains('/')
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                let _ = normalized.pop();
            }
            Component::RootDir | Component::Prefix(_) | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }
    normalized
}
