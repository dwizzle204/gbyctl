//! Built-in skill catalog and planners.

pub mod diagnose_reboot_or_kernel_issue;
pub mod doctor;
pub mod install_package;
pub mod logs_guidance;
pub mod maintenance_guidance;
pub mod package_status;
pub mod resize_root_plan;
pub mod service_status;
pub mod storage;
pub mod troubleshoot_firewall;

use crate::skills::types::{Skill, SkillId};

/// Returns metadata for built-in skills.
#[must_use]
pub fn catalog() -> Vec<Skill> {
    vec![
        Skill {
            id: SkillId::Doctor,
            description: "Quick Ubuntu host health overview".to_owned(),
            intents: vec![
                "doctor".to_owned(),
                "health".to_owned(),
                "check system".to_owned(),
            ],
        },
        Skill {
            id: SkillId::ServiceStatus,
            description: "Inspect a systemd service".to_owned(),
            intents: vec![
                "service".to_owned(),
                "status".to_owned(),
                "won't start".to_owned(),
            ],
        },
        Skill {
            id: SkillId::DiskFullTriage,
            description: "Storage pressure triage and safe cleanup options".to_owned(),
            intents: vec!["disk full".to_owned(), "no space".to_owned()],
        },
        Skill {
            id: SkillId::InspectStorage,
            description: "Inspect storage layout, block devices, and mounts".to_owned(),
            intents: vec![
                "inspect storage".to_owned(),
                "lsblk".to_owned(),
                "mount".to_owned(),
            ],
        },
        Skill {
            id: SkillId::ResizeRootPlan,
            description: "Plan root filesystem resize".to_owned(),
            intents: vec!["resize root".to_owned(), "make root disk bigger".to_owned()],
        },
        Skill {
            id: SkillId::InstallPackage,
            description: "Install an Ubuntu package via apt after explicit approval".to_owned(),
            intents: vec![
                "install package".to_owned(),
                "install tomcat".to_owned(),
                "install nginx".to_owned(),
                "install docker".to_owned(),
            ],
        },
        Skill {
            id: SkillId::TroubleshootFirewall,
            description: "Troubleshoot local firewall and listeners".to_owned(),
            intents: vec!["firewall".to_owned(), "port".to_owned(), "ufw".to_owned()],
        },
        Skill {
            id: SkillId::DiagnoseRebootOrKernelIssue,
            description: "Inspect reboot and kernel clues".to_owned(),
            intents: vec!["reboot".to_owned(), "kernel".to_owned(), "crash".to_owned()],
        },
        Skill {
            id: SkillId::MaintenanceGuidance,
            description: "Review security, updates, and kernel maintenance best practices"
                .to_owned(),
            intents: vec![
                "security best practices".to_owned(),
                "system updates".to_owned(),
                "kernel maintenance".to_owned(),
            ],
        },
        Skill {
            id: SkillId::LogsGuidance,
            description: "Inspect recent system and service logs for operational clues".to_owned(),
            intents: vec![
                "logs".to_owned(),
                "what happened".to_owned(),
                "recent errors".to_owned(),
            ],
        },
        Skill {
            id: SkillId::PackageStatus,
            description: "Check whether a package is installed, its version, and update status"
                .to_owned(),
            intents: vec![
                "package status".to_owned(),
                "is package installed".to_owned(),
                "package version".to_owned(),
            ],
        },
    ]
}
