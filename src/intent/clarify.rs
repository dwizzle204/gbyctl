//! Clarification helpers.

use crate::skills::types::{ClarificationQuestion, IntentMatch, SkillId};

/// Returns any required clarification for a routed intent.
#[must_use]
pub fn required_question(intent: &IntentMatch) -> Option<ClarificationQuestion> {
    match intent.skill_id {
        SkillId::ServiceStatus => {
            if intent.args.contains_key("service") {
                None
            } else {
                Some(ClarificationQuestion {
                    prompt: "Which service name should I inspect?".to_owned(),
                    field: "service".to_owned(),
                    choices: Vec::new(),
                })
            }
        }
        SkillId::TroubleshootFirewall => {
            if intent.args.contains_key("port") {
                None
            } else {
                Some(ClarificationQuestion {
                    prompt: "Which port should be checked?".to_owned(),
                    field: "port".to_owned(),
                    choices: vec!["80".to_owned(), "443".to_owned(), "8080".to_owned()],
                })
            }
        }
        SkillId::InstallPackage => {
            if intent.args.contains_key("package") {
                None
            } else {
                Some(ClarificationQuestion {
                    prompt: "Which Ubuntu package should be installed?".to_owned(),
                    field: "package".to_owned(),
                    choices: vec![
                        "tomcat10".to_owned(),
                        "nginx".to_owned(),
                        "docker.io".to_owned(),
                    ],
                })
            }
        }
        SkillId::PackageStatus => {
            if intent.args.contains_key("package") {
                None
            } else {
                Some(ClarificationQuestion {
                    prompt: "Which Ubuntu package should be checked?".to_owned(),
                    field: "package".to_owned(),
                    choices: vec![
                        "nginx".to_owned(),
                        "docker.io".to_owned(),
                        "tomcat10".to_owned(),
                    ],
                })
            }
        }
        SkillId::Doctor
        | SkillId::DiskFullTriage
        | SkillId::InspectStorage
        | SkillId::ResizeRootPlan
        | SkillId::DiagnoseRebootOrKernelIssue
        | SkillId::MaintenanceGuidance
        | SkillId::LogsGuidance => None,
    }
}
