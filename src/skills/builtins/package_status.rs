//! Built-in planner for package status checks.

use std::collections::BTreeMap;

use crate::policy::sanitize;
use crate::skills::types::{
    CommandTemplate, Plan, PlanStep, PolicyClass, SkillId, VerificationStep,
};

/// Build package status plan.
#[must_use]
pub fn plan(request: &str, args: &BTreeMap<String, String>) -> Plan {
    let package = args
        .get("package")
        .and_then(|value| sanitize::package_name(value).ok())
        .unwrap_or_else(|| "nginx".to_owned());

    Plan {
        skill_id: SkillId::PackageStatus,
        request: request.to_owned(),
        steps: vec![
            PlanStep {
                id: "package-policy".to_owned(),
                command: CommandTemplate {
                    summary: format!("Candidate and installed version for {package}"),
                    command: format!("apt-cache policy {package}"),
                    modifies_state: false,
                },
                policy_class: PolicyClass::SafeExecute,
                policy_note: "Read-only inspection".to_owned(),
            },
            PlanStep {
                id: "package-installed".to_owned(),
                command: CommandTemplate {
                    summary: format!("Installed package record for {package}"),
                    command: format!("dpkg-query -W {package}"),
                    modifies_state: false,
                },
                policy_class: PolicyClass::SafeExecute,
                policy_note: "Read-only inspection".to_owned(),
            },
        ],
        verification: vec![VerificationStep {
            summary: "Pending update list".to_owned(),
            command: "apt list --upgradable".to_owned(),
        }],
        manual_guidance: vec![
            "If the package is not installed, use the install workflow instead of ad-hoc commands."
                .to_owned(),
            "Review candidate versus installed version before approving upgrades.".to_owned(),
        ],
        refusal_reason: None,
    }
}
