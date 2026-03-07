//! Post-execution summary helpers.

use crate::exec::runner::CommandResult;
use crate::skills::types::Plan;

/// Final outcome summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationSummary {
    /// Outcome label.
    pub outcome: String,
    /// Human-readable details.
    pub details: String,
}

/// Build summary from plan and step results.
#[must_use]
pub fn build(plan: &Plan, results: &[CommandResult]) -> VerificationSummary {
    let mut failed = 0i32;
    for result in results {
        if result.exit_code != 0 {
            failed += 1;
        }
    }

    if failed == 0 {
        VerificationSummary {
            outcome: "success".to_owned(),
            details: format!(
                "Skill {} completed with {} executed steps",
                plan.skill_id.as_str(),
                results.len()
            ),
        }
    } else {
        VerificationSummary {
            outcome: "partial_failure".to_owned(),
            details: format!(
                "Skill {} completed with {} failing steps out of {}",
                plan.skill_id.as_str(),
                failed,
                results.len()
            ),
        }
    }
}
