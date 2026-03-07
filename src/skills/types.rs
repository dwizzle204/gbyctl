//! Core skill and planning types.

use serde::{Deserialize, Serialize};

/// Identifier for a bounded skill.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SkillId {
    /// doctor
    Doctor,
    /// service_status
    ServiceStatus,
    /// disk_full_triage
    DiskFullTriage,
    /// inspect_storage
    InspectStorage,
    /// resize_root_plan
    ResizeRootPlan,
    /// install_package
    InstallPackage,
    /// troubleshoot_firewall
    TroubleshootFirewall,
    /// diagnose_reboot_or_kernel_issue
    DiagnoseRebootOrKernelIssue,
    /// maintenance_guidance
    MaintenanceGuidance,
    /// logs_guidance
    LogsGuidance,
    /// package_status
    PackageStatus,
}

impl SkillId {
    /// Returns the canonical wire name.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Doctor => "doctor",
            Self::ServiceStatus => "service_status",
            Self::DiskFullTriage => "disk_full_triage",
            Self::InspectStorage => "inspect_storage",
            Self::ResizeRootPlan => "resize_root_plan",
            Self::InstallPackage => "install_package",
            Self::TroubleshootFirewall => "troubleshoot_firewall",
            Self::DiagnoseRebootOrKernelIssue => "diagnose_reboot_or_kernel_issue",
            Self::MaintenanceGuidance => "maintenance_guidance",
            Self::LogsGuidance => "logs_guidance",
            Self::PackageStatus => "package_status",
        }
    }
}

/// Policy class for a step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyClass {
    /// Safe command execution.
    SafeExecute,
    /// Requires explicit operator consent.
    ApprovalRequired,
    /// Must be executed manually by operator.
    ManualOnly,
    /// Must never be operationalized.
    Forbidden,
}

/// Route confidence for intent resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentConfidence {
    /// Deterministic mapping.
    High,
    /// Fuzzy but acceptable mapping.
    Medium,
    /// Insufficient confidence.
    Low,
}

/// Output from intent routing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntentMatch {
    /// Selected skill id.
    pub skill_id: SkillId,
    /// Confidence level.
    pub confidence: IntentConfidence,
    /// Freeform extracted args.
    pub args: std::collections::BTreeMap<String, String>,
}

/// Clarification prompt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClarificationQuestion {
    /// Prompt shown to operator.
    pub prompt: String,
    /// Machine key for expected field.
    pub field: String,
    /// Optional bounded choices.
    pub choices: Vec<String>,
}

/// Shell command template as bounded operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandTemplate {
    /// Human summary.
    pub summary: String,
    /// Exact shell command.
    pub command: String,
    /// Whether this command mutates state.
    pub modifies_state: bool,
}

/// Verification step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationStep {
    /// Human summary.
    pub summary: String,
    /// Verification command.
    pub command: String,
}

/// A planned operation step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanStep {
    /// Stable step id.
    pub id: String,
    /// Command template.
    pub command: CommandTemplate,
    /// Policy class for this step.
    pub policy_class: PolicyClass,
    /// Why this policy was chosen.
    pub policy_note: String,
}

/// Complete bounded plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    /// Selected skill.
    pub skill_id: SkillId,
    /// Original request.
    pub request: String,
    /// Execution steps.
    pub steps: Vec<PlanStep>,
    /// Verification steps.
    pub verification: Vec<VerificationStep>,
    /// Optional manual-only guidance.
    pub manual_guidance: Vec<String>,
    /// Optional refusal reason.
    pub refusal_reason: Option<String>,
}

/// Skill metadata plus planner callback.
#[derive(Debug, Clone)]
pub struct Skill {
    /// Skill id.
    pub id: SkillId,
    /// Human description.
    pub description: String,
    /// Short intent aliases.
    pub intents: Vec<String>,
}
