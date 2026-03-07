//! Compact pre-execution security checklist.

use crate::policy;
use crate::skills::types::{PlanStep, PolicyClass};

/// Result of checklist evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChecklistResult {
    /// Step may continue to normal policy handling.
    Allow,
    /// Step must stop immediately with refusal.
    Block { reason: String },
}

/// Evaluate a single plan step against required security checks.
#[must_use]
pub fn evaluate(step: &PlanStep) -> ChecklistResult {
    // Supported scope: only known policy classes may execute.
    let class_known = matches!(
        step.policy_class,
        PolicyClass::SafeExecute
            | PolicyClass::ApprovalRequired
            | PolicyClass::ManualOnly
            | PolicyClass::Forbidden
    );
    if !class_known {
        return ChecklistResult::Block {
            reason: "Unknown policy class for step".to_owned(),
        };
    }

    // Least privilege: read-only steps must not require sudo unless they are
    // explicitly approval-gated (for example, operator-confirmed recovery).
    let has_sudo = step.command.command.starts_with("sudo ");
    if !step.command.modifies_state
        && has_sudo
        && step.policy_class != PolicyClass::ApprovalRequired
    {
        return ChecklistResult::Block {
            reason: "Read-only step uses elevated command; violates least privilege".to_owned(),
        };
    }

    // Verify target sensitivity before modify.
    let touches_protected = policy::rules::touches_protected_path(&step.command.command);
    if step.command.modifies_state
        && touches_protected
        && step.policy_class == PolicyClass::SafeExecute
    {
        return ChecklistResult::Block {
            reason: "Protected target modification cannot be safe_execute".to_owned(),
        };
    }

    // Reversible/risk control: mutating steps cannot run without explicit risk class.
    if step.command.modifies_state && step.policy_class == PolicyClass::SafeExecute {
        return ChecklistResult::Block {
            reason: "Mutating step lacks approval/manual risk class".to_owned(),
        };
    }

    // Security controls should never slip through as safe execution.
    if policy::deny::is_manual_only(&step.command.command)
        && step.policy_class == PolicyClass::SafeExecute
    {
        return ChecklistResult::Block {
            reason: "Security-sensitive operation misclassified as safe_execute".to_owned(),
        };
    }

    // Hard refusal rules always win.
    if policy::deny::is_forbidden(&step.command.command) {
        return ChecklistResult::Block {
            reason: "Forbidden operation detected by hard-deny rules".to_owned(),
        };
    }

    ChecklistResult::Allow
}

#[cfg(test)]
mod tests {
    use crate::policy::checklist::{ChecklistResult, evaluate};
    use crate::skills::types::{CommandTemplate, PlanStep, PolicyClass};

    fn make_step(command: &str, modifies_state: bool, policy_class: PolicyClass) -> PlanStep {
        PlanStep {
            id: "step".to_owned(),
            command: CommandTemplate {
                summary: "summary".to_owned(),
                command: command.to_owned(),
                modifies_state,
            },
            policy_class,
            policy_note: "note".to_owned(),
        }
    }

    #[test]
    fn blocks_readonly_sudo() {
        let step = make_step("sudo cat /etc/os-release", false, PolicyClass::SafeExecute);
        let result = evaluate(&step);
        assert!(matches!(result, ChecklistResult::Block { .. }));
    }

    #[test]
    fn blocks_mutating_safe_execute() {
        let step = make_step("touch /tmp/x", true, PolicyClass::SafeExecute);
        let result = evaluate(&step);
        assert!(matches!(result, ChecklistResult::Block { .. }));
    }

    #[test]
    fn allows_approved_mutating_step() {
        let step = make_step(
            "sudo apt-get install -y nginx",
            true,
            PolicyClass::ApprovalRequired,
        );
        let result = evaluate(&step);
        assert_eq!(result, ChecklistResult::Allow);
    }

    #[test]
    fn allows_readonly_sudo_when_approval_required() {
        let step = make_step(
            "sudo journalctl -n 20",
            false,
            PolicyClass::ApprovalRequired,
        );
        let result = evaluate(&step);
        assert_eq!(result, ChecklistResult::Allow);
    }
}
