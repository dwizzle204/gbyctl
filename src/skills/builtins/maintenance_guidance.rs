//! Built-in planner for maintenance and hardening guidance.

use crate::skills::types::{
    CommandTemplate, Plan, PlanStep, PolicyClass, SkillId, VerificationStep,
};

/// Build maintenance guidance plan.
#[must_use]
pub fn plan(request: &str) -> Plan {
    Plan {
        skill_id: SkillId::MaintenanceGuidance,
        request: request.to_owned(),
        steps: vec![
            PlanStep {
                id: "os-release".to_owned(),
                command: CommandTemplate {
                    summary: "Ubuntu release and support baseline".to_owned(),
                    command: "cat /etc/os-release".to_owned(),
                    modifies_state: false,
                },
                policy_class: PolicyClass::SafeExecute,
                policy_note: "Read-only inspection".to_owned(),
            },
            PlanStep {
                id: "kernel".to_owned(),
                command: CommandTemplate {
                    summary: "Running kernel version".to_owned(),
                    command: "uname -r".to_owned(),
                    modifies_state: false,
                },
                policy_class: PolicyClass::SafeExecute,
                policy_note: "Read-only inspection".to_owned(),
            },
            PlanStep {
                id: "upgradable-packages".to_owned(),
                command: CommandTemplate {
                    summary: "Pending package updates".to_owned(),
                    command: "apt list --upgradable".to_owned(),
                    modifies_state: false,
                },
                policy_class: PolicyClass::SafeExecute,
                policy_note: "Read-only inspection".to_owned(),
            },
        ],
        verification: vec![VerificationStep {
            summary: "Confirm current uptime before scheduling reboots".to_owned(),
            command: "uptime".to_owned(),
        }],
        manual_guidance: vec![
            "Apply security updates routinely and review pending package changes before approval."
                .to_owned(),
            "Treat kernel updates as reboot-planned changes and verify maintenance windows first."
                .to_owned(),
            "Prefer least privilege, keep SSH and firewall controls enabled, and avoid broad access exceptions."
                .to_owned(),
            "Before major updates, confirm backups, rollback options, and service restart impact."
                .to_owned(),
        ],
        refusal_reason: None,
    }
}
