//! Policy rule helpers.

use std::path::Path;

use crate::exec::command;

/// Sensitive path targets.
pub const PROTECTED_PATHS: [&str; 6] = ["/", "/boot", "/etc", "/usr", "/root", "/var/lib"];

/// Returns true when command references protected paths.
#[must_use]
pub fn touches_protected_path(command: &str) -> bool {
    let paths = command::extract_path_args(command);
    for path in paths {
        for protected in PROTECTED_PATHS {
            if path.starts_with(Path::new(protected)) {
                return true;
            }
        }
    }
    false
}

/// Returns true if command appears read-only.
#[must_use]
pub fn is_read_only(command: &str) -> bool {
    let Ok(parsed) = command::parse(command) else {
        return false;
    };

    let program = parsed.program.to_string_lossy();
    let args: Vec<String> = parsed
        .args
        .iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();

    match program.as_ref() {
        "cat" | "df" | "du" | "findmnt" | "lsblk" | "uptime" | "last" | "journalctl" | "ss"
        | "true" | "free" | "apt-cache" | "uname" | "dpkg-query" => true,
        "systemctl" => args
            .first()
            .is_some_and(|arg| arg == "status" || arg == "--failed" || arg == "list-units"),
        "apt" => args.first().is_some_and(|arg| arg == "list"),
        "ufw" => args.first().is_some_and(|arg| arg == "status"),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{is_read_only, touches_protected_path};

    #[test]
    fn detects_normalized_protected_paths() {
        assert!(touches_protected_path("cat /etc/../etc/passwd"));
    }

    #[test]
    fn plain_read_only_probe_stays_read_only() {
        assert!(is_read_only("journalctl -k -b -1 -p err --no-pager"));
    }
}
