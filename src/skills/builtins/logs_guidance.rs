//! Built-in planner for operational log investigation.

use std::collections::BTreeMap;

use crate::policy::sanitize;
use crate::skills::types::{
    CommandTemplate, Plan, PlanStep, PolicyClass, SkillId, VerificationStep,
};

/// Build logs investigation plan.
#[must_use]
pub fn plan(request: &str, args: &BTreeMap<String, String>) -> Plan {
    let service = args
        .get("service")
        .and_then(|value| sanitize::service_name(value).ok());

    let mut steps = vec![PlanStep {
        id: "journal-recent-errors".to_owned(),
        command: CommandTemplate {
            summary: "Recent warning and error logs".to_owned(),
            command: "journalctl -p warning -n 100 --no-pager".to_owned(),
            modifies_state: false,
        },
        policy_class: PolicyClass::SafeExecute,
        policy_note: "Read-only inspection".to_owned(),
    }];

    if let Some(service) = &service {
        steps.push(PlanStep {
            id: "service-logs".to_owned(),
            command: CommandTemplate {
                summary: format!("Recent logs for service {service}"),
                command: format!("journalctl -u {service} -n 80 --no-pager"),
                modifies_state: false,
            },
            policy_class: PolicyClass::SafeExecute,
            policy_note: "Read-only inspection".to_owned(),
        });
    }

    Plan {
        skill_id: SkillId::LogsGuidance,
        request: request.to_owned(),
        steps,
        verification: vec![VerificationStep {
            summary: "Current boot time context".to_owned(),
            command: "uptime".to_owned(),
        }],
        manual_guidance: vec![
            "Start with recent warnings and errors before reading full logs.".to_owned(),
            "If a service is implicated, inspect its systemd status and unit-specific journal entries next."
                .to_owned(),
            "Avoid changing configuration or rotating logs until evidence is captured.".to_owned(),
        ],
        refusal_reason: None,
    }
}
