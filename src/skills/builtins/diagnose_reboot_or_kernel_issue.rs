//! Built-in planner for diagnose_reboot_or_kernel_issue.

use crate::skills::types::{
    CommandTemplate, Plan, PlanStep, PolicyClass, SkillId, VerificationStep,
};

/// Build reboot/kernel diagnostics plan.
#[must_use]
pub fn plan(request: &str) -> Plan {
    Plan {
        skill_id: SkillId::DiagnoseRebootOrKernelIssue,
        request: request.to_owned(),
        steps: vec![
            PlanStep {
                id: "last-reboots".to_owned(),
                command: CommandTemplate {
                    summary: "Recent reboot history".to_owned(),
                    command: "last -x reboot".to_owned(),
                    modifies_state: false,
                },
                policy_class: PolicyClass::SafeExecute,
                policy_note: "Read-only inspection".to_owned(),
            },
            PlanStep {
                id: "kernel-errors".to_owned(),
                command: CommandTemplate {
                    summary: "Kernel errors from previous boot".to_owned(),
                    command: "journalctl -k -b -1 -p err --no-pager".to_owned(),
                    modifies_state: false,
                },
                policy_class: PolicyClass::SafeExecute,
                policy_note: "Read-only inspection".to_owned(),
            },
        ],
        verification: vec![VerificationStep {
            summary: "Current boot id".to_owned(),
            command: "cat /proc/sys/kernel/random/boot_id".to_owned(),
        }],
        manual_guidance: Vec::new(),
        refusal_reason: None,
    }
}
