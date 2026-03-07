//! Hard-deny and manual-only checks.

use crate::exec::command;

fn normalized(command: &str) -> String {
    command
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

/// Returns true if command falls into forbidden class.
#[must_use]
pub fn is_forbidden(command: &str) -> bool {
    let normalized = normalized(command);
    let parse_failed = command::parse(command).is_err();
    let patterns = [
        "rm -rf /",
        "mkfs",
        "backdoor",
        "wipe logs",
        "shred /",
        "passwd root",
    ];
    if parse_failed {
        return true;
    }
    for pattern in patterns {
        if normalized.contains(pattern) {
            return true;
        }
    }
    false
}

/// Returns true if command should be manual-only.
#[must_use]
pub fn is_manual_only(command: &str) -> bool {
    let normalized = normalized(command);
    let patterns = [
        "growpart",
        "parted",
        "fdisk",
        "passwd root",
        "visudo",
        "/etc/ssh/sshd_config",
        "grub",
    ];
    for pattern in patterns {
        if normalized.contains(pattern) {
            return true;
        }
    }
    false
}
