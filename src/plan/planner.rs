//! Planner implementation.

use anyhow::{Context, Result};

use crate::plan::step::enforce_policy;
use crate::policy;
use crate::skills::builtins;
use crate::skills::types::{IntentMatch, Plan, SkillId};

/// Build bounded plan from routed intent.
pub fn build(request: &str, intent: &IntentMatch) -> Result<Plan> {
    let mut plan = match intent.skill_id {
        SkillId::Doctor => builtins::doctor::plan(request),
        SkillId::ServiceStatus => builtins::service_status::plan(request, &intent.args),
        SkillId::DiskFullTriage => builtins::storage::disk_pressure_plan(request),
        SkillId::InspectStorage => builtins::storage::layout_plan(request),
        SkillId::ResizeRootPlan => builtins::resize_root_plan::plan(request),
        SkillId::InstallPackage => builtins::install_package::plan(request, &intent.args),
        SkillId::TroubleshootFirewall => {
            builtins::troubleshoot_firewall::plan(request, &intent.args)
        }
        SkillId::DiagnoseRebootOrKernelIssue => {
            builtins::diagnose_reboot_or_kernel_issue::plan(request)
        }
        SkillId::MaintenanceGuidance => builtins::maintenance_guidance::plan(request),
        SkillId::LogsGuidance => builtins::logs_guidance::plan(request, &intent.args),
        SkillId::PackageStatus => builtins::package_status::plan(request, &intent.args),
    };

    for step in &mut plan.steps {
        let (class, note) = policy::classifier::classify(&step.command.command);
        step.policy_class = class;
        step.policy_note = note;
    }

    enforce_policy(&mut plan).context("failed policy enforcement on plan")?;

    Ok(plan)
}
