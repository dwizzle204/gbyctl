//! Built-in planner for package installation workflows.

use std::collections::BTreeMap;

use crate::policy::sanitize;
use crate::skills::types::{
    CommandTemplate, Plan, PlanStep, PolicyClass, SkillId, VerificationStep,
};

/// Build package-install plan.
#[must_use]
pub fn plan(request: &str, args: &BTreeMap<String, String>) -> Plan {
    let package = args
        .get("package")
        .and_then(|value| sanitize::package_name(value).ok())
        .unwrap_or_else(|| "tomcat10".to_owned());

    Plan {
        skill_id: SkillId::InstallPackage,
        request: request.to_owned(),
        steps: vec![
            PlanStep {
                id: "apt-update".to_owned(),
                command: CommandTemplate {
                    summary: "Refresh apt cache".to_owned(),
                    command: "sudo apt-get update".to_owned(),
                    modifies_state: true,
                },
                policy_class: PolicyClass::ApprovalRequired,
                policy_note: "Package changes require explicit approval".to_owned(),
            },
            PlanStep {
                id: "apt-install".to_owned(),
                command: CommandTemplate {
                    summary: format!("Install package {package}"),
                    command: format!("sudo apt-get install -y {package}"),
                    modifies_state: true,
                },
                policy_class: PolicyClass::ApprovalRequired,
                policy_note: "Package changes require explicit approval".to_owned(),
            },
        ],
        verification: vec![VerificationStep {
            summary: format!("Verify package metadata for {package}"),
            command: format!("apt-cache policy {package}"),
        }],
        manual_guidance: vec![
            "Confirm the package name is correct before approving install.".to_owned(),
            "Prefer Ubuntu repository packages over ad-hoc installers.".to_owned(),
        ],
        refusal_reason: None,
    }
}
