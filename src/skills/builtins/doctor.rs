//! Built-in planner for doctor.

use crate::skills::types::{
    CommandTemplate, Plan, PlanStep, PolicyClass, SkillId, VerificationStep,
};

/// Build doctor plan.
#[must_use]
pub fn plan(request: &str) -> Plan {
    let steps = vec![
        PlanStep {
            id: "os-release".to_owned(),
            command: CommandTemplate {
                summary: "Ubuntu release".to_owned(),
                command: "cat /etc/os-release".to_owned(),
                modifies_state: false,
            },
            policy_class: PolicyClass::SafeExecute,
            policy_note: "Read-only inspection".to_owned(),
        },
        PlanStep {
            id: "uptime".to_owned(),
            command: CommandTemplate {
                summary: "Uptime and load".to_owned(),
                command: "uptime".to_owned(),
                modifies_state: false,
            },
            policy_class: PolicyClass::SafeExecute,
            policy_note: "Read-only inspection".to_owned(),
        },
        PlanStep {
            id: "memory".to_owned(),
            command: CommandTemplate {
                summary: "Memory usage".to_owned(),
                command: "free -h".to_owned(),
                modifies_state: false,
            },
            policy_class: PolicyClass::SafeExecute,
            policy_note: "Read-only inspection".to_owned(),
        },
        PlanStep {
            id: "disk".to_owned(),
            command: CommandTemplate {
                summary: "Filesystem usage".to_owned(),
                command: "df -h".to_owned(),
                modifies_state: false,
            },
            policy_class: PolicyClass::SafeExecute,
            policy_note: "Read-only inspection".to_owned(),
        },
        PlanStep {
            id: "failed-services".to_owned(),
            command: CommandTemplate {
                summary: "Failed services".to_owned(),
                command: "systemctl --failed --no-pager".to_owned(),
                modifies_state: false,
            },
            policy_class: PolicyClass::SafeExecute,
            policy_note: "Read-only inspection".to_owned(),
        },
        PlanStep {
            id: "running-services".to_owned(),
            command: CommandTemplate {
                summary: "Running services".to_owned(),
                command: "systemctl list-units --type=service --state=running --no-pager"
                    .to_owned(),
                modifies_state: false,
            },
            policy_class: PolicyClass::SafeExecute,
            policy_note: "Read-only inspection".to_owned(),
        },
    ];

    Plan {
        skill_id: SkillId::Doctor,
        request: request.to_owned(),
        steps,
        verification: vec![VerificationStep {
            summary: "Host still reachable".to_owned(),
            command: "true".to_owned(),
        }],
        manual_guidance: Vec::new(),
        refusal_reason: None,
    }
}
