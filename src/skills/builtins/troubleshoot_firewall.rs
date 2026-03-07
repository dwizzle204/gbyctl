//! Built-in planner for troubleshoot_firewall.

use std::collections::BTreeMap;

use crate::policy::sanitize;
use crate::skills::types::{
    CommandTemplate, Plan, PlanStep, PolicyClass, SkillId, VerificationStep,
};

/// Build firewall troubleshooting plan.
#[must_use]
pub fn plan(request: &str, args: &BTreeMap<String, String>) -> Plan {
    let port = args
        .get("port")
        .and_then(|value| sanitize::port(value).ok())
        .unwrap_or_else(|| "8080".to_owned());

    Plan {
        skill_id: SkillId::TroubleshootFirewall,
        request: request.to_owned(),
        steps: vec![
            PlanStep {
                id: "ufw-status".to_owned(),
                command: CommandTemplate {
                    summary: "Firewall state".to_owned(),
                    command: "ufw status verbose".to_owned(),
                    modifies_state: false,
                },
                policy_class: PolicyClass::SafeExecute,
                policy_note: "Read-only inspection".to_owned(),
            },
            PlanStep {
                id: "listener-check".to_owned(),
                command: CommandTemplate {
                    summary: format!("Listener check for tcp/{port}"),
                    command: "ss -ltnp".to_owned(),
                    modifies_state: false,
                },
                policy_class: PolicyClass::SafeExecute,
                policy_note: "Read-only inspection".to_owned(),
            },
            PlanStep {
                id: "open-port".to_owned(),
                command: CommandTemplate {
                    summary: format!("Open tcp/{port} in ufw"),
                    command: format!("sudo ufw allow {port}/tcp"),
                    modifies_state: true,
                },
                policy_class: PolicyClass::ApprovalRequired,
                policy_note: "Firewall rule changes require explicit approval".to_owned(),
            },
        ],
        verification: vec![VerificationStep {
            summary: "Confirm firewall rule".to_owned(),
            command: "ufw status".to_owned(),
        }],
        manual_guidance: Vec::new(),
        refusal_reason: None,
    }
}
