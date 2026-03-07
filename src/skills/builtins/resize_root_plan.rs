//! Built-in planner for resize_root_plan.

use crate::skills::types::{
    CommandTemplate, Plan, PlanStep, PolicyClass, SkillId, VerificationStep,
};

/// Build resize-root plan.
#[must_use]
pub fn plan(request: &str) -> Plan {
    Plan {
        skill_id: SkillId::ResizeRootPlan,
        request: request.to_owned(),
        steps: vec![
            PlanStep {
                id: "inspect-root-device".to_owned(),
                command: CommandTemplate {
                    summary: "Identify root block device".to_owned(),
                    command: "findmnt -no SOURCE /".to_owned(),
                    modifies_state: false,
                },
                policy_class: PolicyClass::SafeExecute,
                policy_note: "Read-only inspection".to_owned(),
            },
            PlanStep {
                id: "proposed-growpart".to_owned(),
                command: CommandTemplate {
                    summary: "Proposed root partition expansion command".to_owned(),
                    command: "sudo growpart /dev/sda 3".to_owned(),
                    modifies_state: true,
                },
                policy_class: PolicyClass::ManualOnly,
                policy_note: "Root partition changes are manual-only".to_owned(),
            },
        ],
        verification: vec![VerificationStep {
            summary: "Check root size after manual action".to_owned(),
            command: "df -h /".to_owned(),
        }],
        manual_guidance: vec![
            "Run growth commands manually in maintenance windows.".to_owned(),
            "Validate backups and rollback path before editing root partition.".to_owned(),
        ],
        refusal_reason: None,
    }
}
