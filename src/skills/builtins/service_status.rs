//! Built-in planner for service_status.

use std::collections::BTreeMap;

use crate::policy::sanitize;
use crate::skills::types::{
    CommandTemplate, Plan, PlanStep, PolicyClass, SkillId, VerificationStep,
};

/// Build service-status plan.
#[must_use]
pub fn plan(request: &str, args: &BTreeMap<String, String>) -> Plan {
    let service = args
        .get("service")
        .and_then(|value| sanitize::service_name(value).ok())
        .unwrap_or_else(|| "ssh".to_owned());

    let command = format!("systemctl status {} --no-pager", service);

    Plan {
        skill_id: SkillId::ServiceStatus,
        request: request.to_owned(),
        steps: vec![PlanStep {
            id: "service-status".to_owned(),
            command: CommandTemplate {
                summary: format!("Inspect service: {service}"),
                command,
                modifies_state: false,
            },
            policy_class: PolicyClass::SafeExecute,
            policy_note: "Read-only inspection".to_owned(),
        }],
        verification: vec![VerificationStep {
            summary: "Recent logs for service".to_owned(),
            command: format!("journalctl -u {} -n 60 --no-pager", service),
        }],
        manual_guidance: Vec::new(),
        refusal_reason: None,
    }
}
