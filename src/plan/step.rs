//! Plan step utilities.

use anyhow::Result;

use crate::skills::types::{Plan, PolicyClass};

/// Enforce plan-level policy constraints.
pub fn enforce_policy(plan: &mut Plan) -> Result<()> {
    if plan.refusal_reason.is_some() {
        return Ok(());
    }

    if let Some(step) = plan
        .steps
        .iter()
        .find(|step| step.policy_class == PolicyClass::Forbidden)
    {
        let step_id = step.id.clone();
        plan.refusal_reason = Some(format!(
            "Refused by policy: generated forbidden step '{}'",
            step_id
        ));
        plan.steps.clear();
    }

    Ok(())
}
