//! Shared storage workflow planners.

use crate::skills::types::{
    CommandTemplate, Plan, PlanStep, PolicyClass, SkillId, VerificationStep,
};

/// Build storage layout inspection plan.
#[must_use]
pub fn layout_plan(request: &str) -> Plan {
    Plan {
        skill_id: SkillId::InspectStorage,
        request: request.to_owned(),
        steps: vec![
            PlanStep {
                id: "lsblk".to_owned(),
                command: CommandTemplate {
                    summary: "Block devices and mounts".to_owned(),
                    command: "lsblk -f".to_owned(),
                    modifies_state: false,
                },
                policy_class: PolicyClass::SafeExecute,
                policy_note: "Read-only inspection".to_owned(),
            },
            PlanStep {
                id: "findmnt-root".to_owned(),
                command: CommandTemplate {
                    summary: "Root mount details".to_owned(),
                    command: "findmnt /".to_owned(),
                    modifies_state: false,
                },
                policy_class: PolicyClass::SafeExecute,
                policy_note: "Read-only inspection".to_owned(),
            },
        ],
        verification: vec![VerificationStep {
            summary: "Disk usage snapshot".to_owned(),
            command: "df -h".to_owned(),
        }],
        manual_guidance: Vec::new(),
        refusal_reason: None,
    }
}

/// Build storage pressure triage plan.
#[must_use]
pub fn disk_pressure_plan(request: &str) -> Plan {
    Plan {
        skill_id: SkillId::DiskFullTriage,
        request: request.to_owned(),
        steps: vec![
            PlanStep {
                id: "df".to_owned(),
                command: CommandTemplate {
                    summary: "Filesystem usage".to_owned(),
                    command: "df -h".to_owned(),
                    modifies_state: false,
                },
                policy_class: PolicyClass::SafeExecute,
                policy_note: "Read-only inspection".to_owned(),
            },
            PlanStep {
                id: "du-var".to_owned(),
                command: CommandTemplate {
                    summary: "Largest paths under /var".to_owned(),
                    command: "du -xhd1 /var".to_owned(),
                    modifies_state: false,
                },
                policy_class: PolicyClass::SafeExecute,
                policy_note: "Read-only inspection".to_owned(),
            },
            PlanStep {
                id: "storage-layout".to_owned(),
                command: CommandTemplate {
                    summary: "Mounted filesystem layout".to_owned(),
                    command: "findmnt /".to_owned(),
                    modifies_state: false,
                },
                policy_class: PolicyClass::SafeExecute,
                policy_note: "Read-only inspection".to_owned(),
            },
        ],
        verification: vec![VerificationStep {
            summary: "Re-check filesystem usage".to_owned(),
            command: "df -h".to_owned(),
        }],
        manual_guidance: vec![
            "For cleanup, prefer package cache and rotated logs first.".to_owned(),
            "Do not remove unknown files under /var/lib manually.".to_owned(),
        ],
        refusal_reason: None,
    }
}
