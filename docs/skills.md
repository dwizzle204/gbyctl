# Skill System

## Intent to skill mapping

Natural-language requests are classified into one of:

- `skill`
- `clarification`
- `refusal`
- `out_of_scope`

Only `skill` outcomes proceed to planning/execution.

## Built-in workflow families

1. `doctor`
2. `service_status`
3. `disk_full_triage`
4. `inspect_storage`
5. `resize_root_plan`
6. `install_package`
7. `troubleshoot_firewall`
8. `diagnose_reboot_or_kernel_issue`
9. `maintenance_guidance`
10. `logs_guidance`
11. `package_status`

These are intentionally broad operational workflows, not a long catalog of
application-specific scripts. Example asks like "install tomcat" or "install nginx"
should resolve into `install_package`; "why is my server slow" should resolve into
`doctor`; security/update/kernel best-practice asks should resolve into
`maintenance_guidance`; "show me recent logs" should resolve into `logs_guidance`;
package version/install/update asks should resolve into `package_status`.

## Non-goal reinforcement

`gbyctl` is not a coding assistant and not general chat.

Examples of out-of-scope asks:

- "write a python script"
- "refactor my code"

These should return `out_of_scope` or `refusal`, never command execution.

## Adding a new workflow family

1. Add `SkillId` variant in `src/skills/types.rs`.
2. Register metadata in `src/skills/builtins/mod.rs`.
3. Add planner in `src/skills/builtins/<skill>.rs`.
4. Extend deterministic router in `src/intent/router.rs`.
5. Ensure LLM classifier prompt/schema recognizes new skill.
6. Extend planner dispatch in `src/plan/planner.rs`.
7. Add tests for routing, policy, and checklist behavior.

Do not add a new workflow for every phrasing or package. Prefer expanding routing,
argument extraction, or clarification inside an existing workflow family first.

## Command safety rule

Intent detection never bypasses skill templates or policy gates.
