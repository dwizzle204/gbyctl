//! Policy classification for bounded commands.

use crate::policy::deny;
use crate::policy::rules;
use crate::skills::types::PolicyClass;

/// Classify a shell command under policy classes.
#[must_use]
pub fn classify(command: &str) -> (PolicyClass, String) {
    if crate::exec::command::parse(command).is_err() {
        return (
            PolicyClass::Forbidden,
            "Command contains unsafe shell syntax or cannot be parsed safely".to_owned(),
        );
    }

    if deny::is_forbidden(command) {
        return (
            PolicyClass::Forbidden,
            "Command matches forbidden destructive or evasive pattern".to_owned(),
        );
    }

    if deny::is_manual_only(command) {
        return (
            PolicyClass::ManualOnly,
            "Command touches protected manual-only operation".to_owned(),
        );
    }

    if command.starts_with("sudo ") {
        return (
            PolicyClass::ApprovalRequired,
            "Elevated command requires explicit operator approval".to_owned(),
        );
    }

    if rules::touches_protected_path(command) && !rules::is_read_only(command) {
        return (
            PolicyClass::ManualOnly,
            "Protected targets cannot be modified automatically".to_owned(),
        );
    }

    (
        PolicyClass::SafeExecute,
        "Read-only or low-risk command".to_owned(),
    )
}

#[cfg(test)]
mod tests {
    use super::classify;
    use crate::skills::types::PolicyClass;

    #[test]
    fn classifies_safe() {
        let (class, _) = classify("df -h");
        assert_eq!(class, PolicyClass::SafeExecute);
    }

    #[test]
    fn classifies_approval() {
        let (class, _) = classify("sudo apt-get install -y nginx");
        assert_eq!(class, PolicyClass::ApprovalRequired);
    }

    #[test]
    fn classifies_manual_only() {
        let (class, _) = classify("sudo growpart /dev/sda 3");
        assert_eq!(class, PolicyClass::ManualOnly);
    }

    #[test]
    fn classifies_forbidden() {
        let (class, _) = classify("rm -rf /");
        assert_eq!(class, PolicyClass::Forbidden);
    }

    #[test]
    fn rejects_shell_chaining_syntax() {
        let (class, _) = classify("sudo apt-get update; rm -rf /");
        assert_eq!(class, PolicyClass::Forbidden);
    }
}
