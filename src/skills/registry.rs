//! Skill discovery and lookup.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::skills::builtins;
use crate::skills::types::{Skill, SkillId};

/// In-memory registry of supported skills.
#[derive(Debug, Default)]
pub struct SkillRegistry {
    skills: BTreeMap<SkillId, Skill>,
}

#[derive(Debug, Deserialize)]
struct SkillYaml {
    intent: IntentYaml,
}

#[derive(Debug, Deserialize)]
struct IntentYaml {
    skill_id: String,
}

impl SkillRegistry {
    /// Builds a registry from built-ins and optional YAML metadata.
    pub fn load(skills_dir: &Path) -> Result<Self> {
        let mut registry = Self::default();

        for skill in builtins::catalog() {
            let _old = registry.skills.insert(skill.id.clone(), skill);
        }

        if skills_dir.exists() {
            for entry in fs::read_dir(skills_dir)
                .with_context(|| format!("failed to read skills dir: {}", skills_dir.display()))?
            {
                let entry = entry.context("failed to read skill entry")?;
                let path = entry.path();
                if path.extension().and_then(std::ffi::OsStr::to_str) != Some("yaml") {
                    continue;
                }
                let raw = fs::read_to_string(&path)
                    .with_context(|| format!("failed to read skill yaml: {}", path.display()))?;
                let parsed: SkillYaml = serde_yaml::from_str(&raw)
                    .with_context(|| format!("invalid yaml: {}", path.display()))?;
                if let Some(skill_id) = parse_skill_id(&parsed.intent.skill_id)
                    && let Some(existing) = registry.skills.get_mut(&skill_id)
                {
                    existing.intents.push(parsed.intent.skill_id.clone());
                }
            }
        }

        Ok(registry)
    }

    /// Lookup by id.
    #[must_use]
    pub fn get(&self, id: &SkillId) -> Option<&Skill> {
        self.skills.get(id)
    }

    /// Iterate all skills.
    pub fn all(&self) -> impl Iterator<Item = &Skill> {
        self.skills.values()
    }
}

/// Parse canonical skill id names.
#[must_use]
pub fn parse_skill_id(input: &str) -> Option<SkillId> {
    match input {
        "doctor" => Some(SkillId::Doctor),
        "service_status" => Some(SkillId::ServiceStatus),
        "disk_full_triage" => Some(SkillId::DiskFullTriage),
        "inspect_storage" => Some(SkillId::InspectStorage),
        "resize_root_plan" => Some(SkillId::ResizeRootPlan),
        "install_package" | "install_tomcat" => Some(SkillId::InstallPackage),
        "troubleshoot_firewall" => Some(SkillId::TroubleshootFirewall),
        "diagnose_reboot_or_kernel_issue" => Some(SkillId::DiagnoseRebootOrKernelIssue),
        "maintenance_guidance" => Some(SkillId::MaintenanceGuidance),
        "logs_guidance" => Some(SkillId::LogsGuidance),
        "package_status" => Some(SkillId::PackageStatus),
        _ => None,
    }
}
